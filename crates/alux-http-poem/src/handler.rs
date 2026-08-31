use crate::input::{PoemBodyInput, PoemHeaderInput, PoemInputsAlg, PoemPathInput, PoemQueryInput, PoemRequestInput};
use crate::output::{PoemFileOutput, PoemJsonOutput};
use crate::route::{PoemEndpoint, PoemRoute, PoemRouteImpl, PoemSelector};
use alux_ext::{ApplyAlg, HandlerContextAlg};
use alux_http::{
    FileOutAlg, HandlerAlg, HandlerEndpointAlg, HttpInputAlg, HttpSelectorAlg, JsonOutAlg, OutputAlg, OutputKindAlg,
    RouteAlg, SelectorAlg,
};
use poem::endpoint::make;
use poem::{IntoResponse, Request, Response};
use std::sync::Arc;

/// Interprets typed HTTP programs as executable Poem routes.
pub struct PoemHandlerImpl<Context> {
    context: Arc<Context>,
}

impl<Context> PoemHandlerImpl<Context> {
    /// Creates an interpreter owning a newly shared context.
    pub fn new(context: Context) -> Self {
        Self { context: Arc::new(context) }
    }

    /// Creates an interpreter from an existing shared context.
    pub fn from_shared(context: Arc<Context>) -> Self {
        Self { context }
    }
}

impl<Context> HandlerContextAlg<Context> for PoemHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
{
    type Handle = Arc<Context>;
}

impl<Context> HandlerAlg for PoemHandlerImpl<Context> {
    type Endpoint = PoemEndpoint;
}

impl<Context> HttpInputAlg for PoemHandlerImpl<Context> {
    type Path<I> = PoemPathInput<I>;
    type Query<I> = PoemQueryInput<I>;
    type Body<I> = PoemBodyInput<I>;
    type Header<I> = PoemHeaderInput<I>;
    type Auth<I> = PoemRequestInput<I>;
    type Context<I> = PoemRequestInput<I>;
}

impl<Context> JsonOutAlg for PoemHandlerImpl<Context> {
    type Json<From> = PoemJsonOutput;
}

impl<Context> FileOutAlg for PoemHandlerImpl<Context> {
    type File<From> = PoemFileOutput;
}

impl<Context, Inputs, Args, Transform, Output> HandlerEndpointAlg<Arc<Context>, Inputs, Args, Transform, Output>
    for PoemHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
    Inputs: PoemInputsAlg<Args>,
    Args: Send + 'static,
    Output: Send + 'static,
    Transform: OutputKindAlg<Self, Output>,
    <<Transform as OutputKindAlg<Self, Output>>::Transform as OutputAlg<Output>>::Output: IntoResponse,
{
    fn finish_handler<H>(&self, handler: H) -> <Self as HandlerAlg>::Endpoint
    where
        H: ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
    {
        let context = Arc::clone(&self.context);
        let handler = Arc::new(handler);
        PoemEndpoint::new(make(move |request: Request| {
            let context = Arc::clone(&context);
            let handler = Arc::clone(&handler);
            async move {
                let (request, mut body) = request.split();
                let inputs = Inputs::extract(&request, &mut body).await?;
                let output = handler.apply(context, inputs).await;
                Ok::<Response, poem::Error>(
                    <Transform as OutputKindAlg<Self, Output>>::Transform::output(output).into_response(),
                )
            }
        }))
    }
}

impl<Context> SelectorAlg for PoemHandlerImpl<Context> {
    type Selector = PoemSelector;

    fn identity(&self) -> Self::Selector {
        SelectorAlg::identity(&PoemRouteImpl)
    }

    fn compose(&self, first: Self::Selector, second: Self::Selector) -> Self::Selector {
        SelectorAlg::compose(&PoemRouteImpl, first, second)
    }
}

impl<Context> RouteAlg for PoemHandlerImpl<Context> {
    type Route = PoemRoute;
    type Selector = PoemSelector;
    type Endpoint = PoemEndpoint;

    fn initial(&self) -> Self::Route {
        RouteAlg::initial(&PoemRouteImpl)
    }

    fn coproduct(&self, left: Self::Route, right: Self::Route) -> Self::Route {
        RouteAlg::coproduct(&PoemRouteImpl, left, right)
    }

    fn precompose(&self, selector: Self::Selector, route: Self::Route) -> Self::Route {
        RouteAlg::precompose(&PoemRouteImpl, selector, route)
    }

    fn lift(&self, endpoint: Self::Endpoint) -> Self::Route {
        RouteAlg::lift(&PoemRouteImpl, endpoint)
    }
}

impl<Context> HttpSelectorAlg for PoemHandlerImpl<Context> {
    type Selector = PoemSelector;

    fn http_get(&self) -> Self::Selector {
        HttpSelectorAlg::http_get(&PoemRouteImpl)
    }

    fn http_post(&self) -> Self::Selector {
        HttpSelectorAlg::http_post(&PoemRouteImpl)
    }

    fn http_path(&self, path: &str) -> Self::Selector {
        HttpSelectorAlg::http_path(&PoemRouteImpl, path)
    }

    fn http_prefix(&self, prefix: &str) -> Self::Selector {
        HttpSelectorAlg::http_prefix(&PoemRouteImpl, prefix)
    }
}
