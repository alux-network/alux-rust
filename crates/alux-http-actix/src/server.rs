use crate::ActixRoute;
use actix_web::dev::ServerHandle;
use actix_web::{App, HttpServer};
use alux_http::{HttpServerAlg, HttpServerSetup};
use tokio::task::JoinHandle;

type ActixServerError = Box<dyn std::error::Error + Send + Sync>;

/// Bounds how long the workers drain connections before they stop anyway, in seconds.
///
/// Actix Web waits 30 seconds by default, which is long enough to look like a close that never
/// returns. This is the same bound the other interpretations state.
const DRAIN: u64 = 5;

/// Serves an actix-web route at the address its setup names.
#[derive(Debug, Default)]
pub struct ActixServer;

/// Carries an open actix-web server that is accepting requests.
pub struct ActixOpen {
    handle: ServerHandle,
    task: JoinHandle<Result<(), ActixServerError>>,
}

impl Drop for ActixOpen {
    fn drop(&mut self) {
        // `stop` sends its command before the future it returns is awaited, so the workers holding
        // the listener are reached even though a drop cannot wait for them.
        let stopping = self.handle.stop(true);
        drop(stopping);
        self.task.abort();
    }
}

impl HttpServerAlg for ActixServer {
    type Program = ActixRoute;
    type Open = ActixOpen;
    type Error = ActixServerError;

    async fn open(&mut self, setup: HttpServerSetup<Self::Program>) -> Result<Self::Open, Self::Error> {
        let (bind, route) = setup.into_parts();
        let configure = route.into_actix();
        // Actix accepts from its own per-worker runtimes and wants a std listener. Binding here
        // makes a taken address fail like it does in the other crates. Binding does not block.
        let listener = std::net::TcpListener::bind(bind.address())?;
        let server = HttpServer::new(move || App::new().configure(configure.clone()))
            .shutdown_timeout(DRAIN)
            // Whoever opened this server decides when it closes. Left on, actix installs
            // process-wide handlers and answers Ctrl-C and SIGTERM itself, which stops an
            // application from ever seeing them.
            .disable_signals()
            .listen(listener)?
            .run();
        let handle = server.handle();
        let task = actix_web::rt::spawn(async move { server.await.map_err(Into::into) });

        Ok(ActixOpen { handle, task })
    }

    async fn close(&mut self, open: &mut Self::Open) -> Result<(), Self::Error> {
        // Aborting the task does not reach actix's workers, which hold the listener, so ask the
        // server to stop instead. Graceful, so connections already accepted are answered first.
        open.handle.stop(true).await;
        if !open.task.is_finished() {
            let _ = (&mut open.task).await;
        }

        Ok(())
    }

    // Nothing here can tell the address being released from the drain being over, because both
    // happen inside the framework's own shutdown. So ending is closing, which satisfies both.
    async fn end(&mut self, open: &mut Self::Open) -> Result<(), Self::Error> {
        self.close(open).await
    }
}
