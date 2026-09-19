use crate::input::{
    PoemBodyInput, PoemCookieInput, PoemFormInput, PoemHeaderInput, PoemInputsAlg, PoemMultipartInput, PoemPathInput,
    PoemQueryInput, PoemRequestInput,
};
use crate::route::{PoemEndpoint, PoemRoute, PoemRouteImpl, PoemSelector};
use alux_ext::{ApplyAlg, HandlerContextAlg, OperationAlg};
use alux_http::{
    HandlerAlg, HandlerEndpointAlg, HttpInputAlg, HttpMethod, HttpSelectorAlg, OutputAlg, OutputKindAlg, RouteAlg,
    RoutePath, SelectorAlg,
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
    type Form<I> = PoemFormInput<I>;
    type Multipart<I> = PoemMultipartInput<I>;
    // A body taken as it arrived is whatever Poem reads a whole request into, such as `Vec<u8>`.
    type RawBody<I> = PoemRequestInput<I>;
    type Header<I> = PoemHeaderInput<I>;
    type Cookie<I> = PoemCookieInput<I>;
    type Auth<I> = PoemHeaderInput<I>;
    type Context<I> = PoemRequestInput<I>;
}

impl<Context, Inputs, Args, Transform, Answering, Answered, Output>
    HandlerEndpointAlg<Arc<Context>, Inputs, Args, Transform, Output> for PoemHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
    Inputs: PoemInputsAlg<Args>,
    Args: Send + 'static,
    Output: Send + 'static,
    Transform: OutputKindAlg<Self, Output, Transform = Answering>,
    Answering: OutputAlg<Output, Output = Answered>,
    Answered: IntoResponse,
{
    fn finish_handler<Handler>(&self, handler: Handler) -> <Self as HandlerAlg>::Endpoint
    where
        Handler: OperationAlg + ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
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
                Ok::<Response, poem::Error>(Answering::output(output).into_response())
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

    fn http_method(&self, method: HttpMethod) -> Self::Selector {
        HttpSelectorAlg::http_method(&PoemRouteImpl, method)
    }

    fn http_path(&self, path: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_path(&PoemRouteImpl, path)
    }

    fn http_prefix(&self, prefix: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_prefix(&PoemRouteImpl, prefix)
    }
}
