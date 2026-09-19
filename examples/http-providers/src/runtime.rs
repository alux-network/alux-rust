use crate::{ExampleApiExt, ExampleAppAlg, Provider};
use alux_ext::ext;
use alux_http::{
    HttpBind, HttpProgramExt, HttpServerAlg, HttpServerCommand, HttpServerEvent, HttpServerExt, HttpServerSetup,
};
use alux_http_actix::{ActixHandlerImpl, ActixServer};
use alux_http_axum::{AxumHandlerImpl, AxumServer};
use alux_http_direct::DirectHandlerImpl;
use alux_http_hyper::{HyperRoute, HyperServer};
use alux_http_openapi::OpenApiHandlerImpl;
use alux_http_poem::{PoemHandlerImpl, PoemServer};
use alux_http_rocket::{RocketHandlerImpl, RocketServer};
use alux_http_salvo::{SalvoHandlerImpl, SalvoServer};
use alux_http_text::TextHandlerImpl;
use alux_http_typescript::TsHttpClient;
use alux_http_warp::{WarpHandlerImpl, WarpServer};
use alux_sdk::case_mapping;
use alux_shape::Spelling;
use core::future::{Future, pending};
use derive_new::new as New;
use futures::{StreamExt, TryStreamExt, stream};
use std::error::Error;
use std::{io, pin::Pin};
use tokio::sync::{mpsc, oneshot};
use tokio::task::LocalSet;

/// Carries an error returned while opening or serving the example.
pub type BoxError = Box<dyn Error + Send + Sync>;

/// Serves the shared address, as whichever provider currently holds it.
///
/// Each running provider gets its own value naming itself, so answering needs no shared state and
/// there is no window where the server handling a request cannot say which one it is.
#[derive(Clone, New)]
struct FrontDoorApp {
    provider: Provider,
    switches: Switches,
}

/// Asks the dynamic front door to serve a different provider.
#[derive(Clone, New)]
struct Switches(mpsc::UnboundedSender<SwitchRequest>);

/// Requests replacement of the dynamic provider.
struct SwitchRequest {
    provider: Provider,
    complete: oneshot::Sender<()>,
}

/// Names the transition one provider made.
type ProviderEvent = (Provider, HttpServerEvent<BoxError>);

/// Carries every transition one provider will make, in the order it makes them.
type ProviderEvents = Pin<Box<dyn futures::Stream<Item = ProviderEvent>>>;

/// The dynamic provider and the lifecycle reporting what it does.
struct DynamicProvider {
    close: Option<oneshot::Sender<()>>,
    events: ProviderEvents,
}

impl ExampleAppAlg for FrontDoorApp {
    async fn provider(&self) -> String {
        provider::to(self.provider).0.to_owned()
    }

    // The front door cannot switch itself: the server that would answer is the one being closed,
    // so the caller is told where to ask instead.
    async fn switch(&self, _provider: Provider) -> String {
        self.switches.address()
    }

    async fn openapi(&self) -> String {
        self.openapi_document()
    }

    async fn typescript(&self) -> String {
        self.typescript_client()
    }
}

impl Switches {
    fn address(&self) -> String {
        format!("http://127.0.0.1:{}/api", provider::to(Provider::Dynamic).1)
    }

    /// Asks for a provider and waits until it is the one serving.
    ///
    /// Answers whether it is. A switch that did not take effect either failed to serve the address
    /// or was superseded by a later one, and the caller is the one that can tell which.
    async fn switch_provider(&self, provider: Provider) -> bool {
        let (complete, switched) = oneshot::channel();

        self.0.send(SwitchRequest { provider, complete }).is_ok() && switched.await.is_ok()
    }
}

/// Interprets the example's provider setup as the dynamic HTTP server.
#[derive(New)]
struct ProviderServer {
    switches: Switches,
}

impl HttpServerAlg for ProviderServer {
    type Program = SwitchRequest;
    type Open = DynamicProvider;
    type Error = BoxError;

    async fn open(&mut self, setup: HttpServerSetup<Self::Program>) -> Result<Self::Open, Self::Error> {
        let (bind, SwitchRequest { provider, complete }) = setup.into_parts();
        let (close, closing) = oneshot::channel();
        let app = FrontDoorApp::new(provider, self.switches.clone());
        let mut events = app.run_provider(provider, bind, async move {
            let _ = closing.await;
        });
        // Wait for the provider to report it is serving. Probing the address instead would not
        // distinguish this server from the one just closed.
        transition(&mut events).await?;
        // The open that serves a caller is the one that answers it. An open cancelled by a later
        // one drops this sender, which tells its caller the switch was superseded.
        let _ = complete.send(());

        Ok(DynamicProvider { close: Some(close), events })
    }

    async fn close(&mut self, open: &mut Self::Open) -> Result<(), Self::Error> {
        // Dropping the signal asks the provider to close; its next transition reports that the
        // address is free.
        drop(open.close.take());
        transition(&mut open.events).await
    }

    // The inner provider states one transition for the whole close, so there is nothing further to
    // wait for here.
    async fn end(&mut self, open: &mut Self::Open) -> Result<(), Self::Error> {
        self.close(open).await
    }
}

/// Waits for the provider's next transition and reports it.
async fn transition(events: &mut ProviderEvents) -> Result<(), BoxError> {
    let Some((provider, event)) = events.next().await else {
        return Err(io::Error::other("the provider stated no transition").into());
    };
    report(provider, &event);
    match event {
        HttpServerEvent::Failed { error, .. } => Err(error),
        _ => Ok(()),
    }
}

/// Serves one provider at its own address, and switches the shared one when asked.
///
/// The same surface as [`FrontDoorApp`], answered differently in one place: this is not the server
/// a switch replaces, so it can wait for the replacement rather than having to decline.
#[derive(Clone, New)]
struct SwitchingApp {
    provider: Provider,
    switches: Switches,
}

impl ExampleAppAlg for SwitchingApp {
    async fn provider(&self) -> String {
        provider::to(self.provider).0.to_owned()
    }

    async fn switch(&self, provider: Provider) -> String {
        // The address is where to ask either way: a switch that was superseded means another
        // provider is serving there, not that nothing is.
        let _ = self.switches.switch_provider(provider).await;

        self.switches.address()
    }

    async fn openapi(&self) -> String {
        self.openapi_document()
    }

    async fn typescript(&self) -> String {
        self.typescript_client()
    }
}

/// Reads the shared declaration as text: its routes, their inputs and what they answer with.
///
/// The interpretation that describes a program rather than serving it, so this needs no application
/// value and no address. It is the same declaration every provider below compiles.
fn text_spec() -> String {
    let api = TextHandlerImpl;

    api.compile_http(api.example_api::<SwitchingApp>()).lines().join("\n")
}

/// Reads the shared API as the documents that describe it.
///
/// Both are the same declaration compiled by another interpreter, so neither reads anything the
/// application holds: what a caller has to name is the context the surface is declared against,
/// and that is the value the document is asked of.
#[ext(name = ExampleDocumentExt)]
impl<This> This
where
    This: ExampleAppAlg + Send + Sync + 'static,
{
    /// Returns the OpenAPI document for this API.
    fn openapi_document(&self) -> String {
        let api = OpenApiHandlerImpl::<This>::new();
        let route = api.compile_http(api.example_api::<This>());
        serde_json::to_string_pretty(&api.document("HTTP providers", "0.1.0", &route))
            .expect("OpenAPI document serializes")
    }

    /// Returns the TypeScript client for this API.
    fn typescript_client(&self) -> String {
        let api = TsHttpClient::new(Spelling::LowerCamel);
        api.compile_http(api.example_api::<This>()).render()
    }
}

// Ordered by how much of the accept loop each framework leaves under a caller's control, which is
// the order the README compares them in and the order their ports run in.
case_mapping! {
    Provider, (&'static str, u16),
        Dynamic <=> ("dynamic", 3000),
        Hyper   <=> ("hyper", 3001),
        Axum    <=> ("axum", 3002),
        Warp    <=> ("warp", 3003),
        Poem    <=> ("poem", 3004),
        Salvo   <=> ("salvo", 3005),
        Actix   <=> ("actix", 3006),
        Rocket  <=> ("rocket", 3007),
}

fn provider_bind(provider: Provider) -> HttpBind {
    let port = provider::to(provider).1;
    HttpBind::new(std::net::SocketAddr::from(([127, 0, 0, 1], port)))
}

/// Runs every provider until `stop` resolves.
pub async fn run_until<Stop>(stop: Stop) -> Result<(), BoxError>
where
    Stop: Future<Output = ()>,
{
    // The one declaration every provider below compiles, read by the interpretation that only
    // describes it. Nothing here serves: it is the same value, said in words.
    println!("{}", text_spec());

    LocalSet::new()
        .run_until(async {
            let (requests, switch_requests) = mpsc::unbounded_channel();
            let switches = Switches::new(requests);
            tokio::task::spawn_local(switch_loop(switches.clone(), switch_requests));
            // Nothing else is serving yet, so a switch that did not take effect could only have
            // failed, and an example without its front door is not worth running.
            if !switches.switch_provider(Provider::Hyper).await {
                let bind = provider_bind(Provider::Dynamic).address();
                return Err(io::Error::other(format!("nothing could be served at {bind}")).into());
            }

            let hyper = SwitchingApp::new(Provider::Hyper, switches.clone());
            let axum = SwitchingApp::new(Provider::Axum, switches.clone());
            let warp = SwitchingApp::new(Provider::Warp, switches.clone());
            let poem = SwitchingApp::new(Provider::Poem, switches.clone());
            let salvo = SwitchingApp::new(Provider::Salvo, switches.clone());
            let actix = SwitchingApp::new(Provider::Actix, switches.clone());
            let rocket = SwitchingApp::new(Provider::Rocket, switches);

            // A fixed provider is never asked to close, so it serves until the example stops.
            stream::select_all([
                hyper.run_hyper(provider_bind(Provider::Hyper), pending()).boxed_local(),
                axum.run_axum(provider_bind(Provider::Axum), pending()).boxed_local(),
                warp.run_warp(provider_bind(Provider::Warp), pending()).boxed_local(),
                poem.run_poem(provider_bind(Provider::Poem), pending()).boxed_local(),
                salvo.run_salvo(provider_bind(Provider::Salvo), pending()).boxed_local(),
                actix.run_actix(provider_bind(Provider::Actix), pending()).boxed_local(),
                rocket.run_rocket(provider_bind(Provider::Rocket), pending()).boxed_local(),
            ])
            .take_until(stop)
            .map(Ok)
            .try_for_each(|(provider, event)| async move {
                report(provider, &event);
                match event {
                    // A provider that cannot serve its port leaves the example half up, which is
                    // less use than it not running at all.
                    HttpServerEvent::Failed { error, .. } => Err(error),
                    _ => Ok(()),
                }
            })
            .await
        })
        .await
}

async fn switch_loop(switches: Switches, mut requests: mpsc::UnboundedReceiver<SwitchRequest>) {
    let (command_sender, command_receiver) = mpsc::unbounded_channel();
    tokio::task::spawn_local(async move {
        while let Some(request) = requests.recv().await {
            if request.provider == Provider::Dynamic {
                let _ = request.complete.send(());
            } else {
                // The request travels as the program, so the start that serves a caller is the one
                // that answers it.
                let setup = HttpServerSetup::new(provider_bind(Provider::Dynamic), request);
                if command_sender.send(HttpServerCommand::Open(setup)).is_err() {
                    return;
                }
            }
        }
    });

    let commands = stream::unfold(command_receiver, |mut commands| async {
        let command = commands.recv().await?;
        Some((command, commands))
    });
    let mut events = Box::pin(ProviderServer::new(switches).lifecycle(Box::pin(commands)));

    // The provider taking or closing the address has already stated it, so these are drained.
    while events.next().await.is_some() {}
}

/// Reports what one provider did, colored like a diff: taking an address is green, closing it is
/// red. Nothing is printed before the provider reports it, so no server is announced that never
/// bound.
fn report(provider: Provider, event: &HttpServerEvent<BoxError>) {
    let name = provider::to(provider).0;
    match event {
        HttpServerEvent::Opened(bind) => println!("{ADDED}[{name}] serving http://{}/api{PLAIN}", bind.address()),
        HttpServerEvent::Replaced { closed, opened } => {
            println!("{REMOVED}[{name}] closed {}{PLAIN}", closed.address());
            println!("{ADDED}[{name}] serving http://{}/api{PLAIN}", opened.address());
        }
        HttpServerEvent::Closed(bind) => println!("{REMOVED}[{name}] closed {}{PLAIN}", bind.address()),
        HttpServerEvent::Failed { bind, error } => {
            eprintln!("{REMOVED}[{name}] {} failed: {error}{PLAIN}", bind.address());
        }
    }
}

/// Colors an address taken.
const ADDED: &str = "\x1b[32m";

/// Colors an address closed, and a transition that did not happen.
const REMOVED: &str = "\x1b[31m";

/// Ends a colored line.
const PLAIN: &str = "\x1b[0m";

#[ext(name = FrontDoorAppServerExt)]
impl<This> This
where
    This: Clone + ExampleAppAlg + Send + Sync + 'static,
{
    /// Serves the shared API directly through hyper.
    fn run_hyper<Close>(&self, bind: HttpBind, close: Close) -> impl futures::Stream<Item = ProviderEvent> + 'static
    where
        Close: Future<Output = ()> + 'static,
    {
        let api = DirectHandlerImpl::new(self.clone());
        let route = HyperRoute::new(api.compile_http(api.example_api::<This>()));
        serve(HyperServer, Provider::Hyper, bind, route, close)
    }

    /// Serves the shared API through axum.
    fn run_axum<Close>(&self, bind: HttpBind, close: Close) -> impl futures::Stream<Item = ProviderEvent> + 'static
    where
        Close: Future<Output = ()> + 'static,
    {
        let api = AxumHandlerImpl::new(self.clone());
        let route = api.compile_http(api.example_api::<This>());
        serve(AxumServer, Provider::Axum, bind, route, close)
    }

    /// Serves the shared API through warp.
    fn run_warp<Close>(&self, bind: HttpBind, close: Close) -> impl futures::Stream<Item = ProviderEvent> + 'static
    where
        Close: Future<Output = ()> + 'static,
    {
        let api = WarpHandlerImpl::new(self.clone());
        let route = api.compile_http(api.example_api::<This>());
        serve(WarpServer, Provider::Warp, bind, route, close)
    }

    /// Serves the shared API through Poem.
    fn run_poem<Close>(&self, bind: HttpBind, close: Close) -> impl futures::Stream<Item = ProviderEvent> + 'static
    where
        Close: Future<Output = ()> + 'static,
    {
        let api = PoemHandlerImpl::new(self.clone());
        let route = api.compile_http(api.example_api::<This>());
        serve(PoemServer, Provider::Poem, bind, route, close)
    }

    /// Serves the shared API through Salvo.
    fn run_salvo<Close>(&self, bind: HttpBind, close: Close) -> impl futures::Stream<Item = ProviderEvent> + 'static
    where
        Close: Future<Output = ()> + 'static,
    {
        let api = SalvoHandlerImpl::new(self.clone());
        let route = api.compile_http(api.example_api::<This>());
        serve(SalvoServer, Provider::Salvo, bind, route, close)
    }

    /// Serves the shared API through Actix Web.
    fn run_actix<Close>(&self, bind: HttpBind, close: Close) -> impl futures::Stream<Item = ProviderEvent> + 'static
    where
        Close: Future<Output = ()> + 'static,
    {
        let api = ActixHandlerImpl::new(self.clone());
        let route = api.compile_http(api.example_api::<This>());
        serve(ActixServer, Provider::Actix, bind, route, close)
    }

    /// Serves the shared API through Rocket.
    fn run_rocket<Close>(&self, bind: HttpBind, close: Close) -> impl futures::Stream<Item = ProviderEvent> + 'static
    where
        Close: Future<Output = ()> + 'static,
    {
        let api = RocketHandlerImpl::new(self.clone());
        let route = api.compile_http(api.example_api::<This>());
        serve(RocketServer, Provider::Rocket, bind, route, close)
    }

    /// Serves the shared API through the selected provider.
    fn run_provider<Close>(&self, provider: Provider, bind: HttpBind, close: Close) -> ProviderEvents
    where
        Close: Future<Output = ()> + 'static,
    {
        match provider {
            Provider::Dynamic => stream::empty().boxed_local(),
            Provider::Hyper => self.run_hyper(bind, close).boxed_local(),
            Provider::Axum => self.run_axum(bind, close).boxed_local(),
            Provider::Warp => self.run_warp(bind, close).boxed_local(),
            Provider::Poem => self.run_poem(bind, close).boxed_local(),
            Provider::Salvo => self.run_salvo(bind, close).boxed_local(),
            Provider::Actix => self.run_actix(bind, close).boxed_local(),
            Provider::Rocket => self.run_rocket(bind, close).boxed_local(),
        }
    }
}

/// Serves one surface until `close` resolves, reporting each transition.
///
/// Commands are a stream, so asking a server to close its address is one more command rather than
/// reaching for the task serving it.
fn serve<Server, Surface, Close>(
    server: Server,
    provider: Provider,
    bind: HttpBind,
    surface: Surface,
    close: Close,
) -> impl futures::Stream<Item = ProviderEvent>
where
    Server: HttpServerAlg<Program = Surface, Error = BoxError>,
    Close: Future<Output = ()> + 'static,
{
    let commands =
        stream::iter([HttpServerCommand::Open(HttpServerSetup::new(bind, surface))]).chain(stream::once(async move {
            close.await;
            HttpServerCommand::Close
        }));

    server.lifecycle(Box::pin(commands)).map(move |event| (provider, event))
}
