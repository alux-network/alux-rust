//! Reaches one endpoint: it reads the arguments its roles state, applies the operation, and answers.

use crate::input::{
    WarpBodyInput, WarpCookieInput, WarpFormInput, WarpHeadInput, WarpHeaderInput, WarpInputsAlg, WarpMultipartInput,
    WarpPathInput, WarpQueryInput, WarpRawBodyInput,
};
use crate::route::{WarpEndpoint, WarpRoute, WarpRouteImpl, WarpSelector};
use alux_ext::{ApplyAlg, HandlerContextAlg, OperationAlg};
use alux_http::{
    HandlerAlg, HandlerEndpointAlg, HttpErrorAlg, HttpInputAlg, HttpMethod, HttpSelectorAlg, OutputAlg, OutputKindAlg,
    RouteAlg, RoutePath, SelectorAlg,
};
use std::sync::Arc;
use warp::http::header;
use warp::reply::Response;

/// Interprets typed HTTP programs as an executable warp filter.
pub struct WarpHandlerImpl<Context> {
    context: Arc<Context>,
}

impl<Context> WarpHandlerImpl<Context> {
    /// Creates an interpreter owning a newly shared context.
    pub fn new(context: Context) -> Self {
        Self { context: Arc::new(context) }
    }

    /// Creates an interpreter from an existing shared context.
    pub fn from_shared(context: Arc<Context>) -> Self {
        Self { context }
    }
}

impl<Context> HandlerContextAlg<Context> for WarpHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
{
    type Handle = Arc<Context>;
}

impl<Context> HandlerAlg for WarpHandlerImpl<Context> {
    type Endpoint = WarpEndpoint;
}

impl<Context> HttpInputAlg for WarpHandlerImpl<Context> {
    type Path<I> = WarpPathInput<I>;
    type Query<I> = WarpQueryInput<I>;
    type Body<I> = WarpBodyInput<I>;
    type Form<I> = WarpFormInput<I>;
    type Multipart<I> = WarpMultipartInput<I>;
    type RawBody<I> = WarpRawBodyInput<I>;
    // warp gathers a header, an authentication value, and an endpoint context from the headers.
    type Header<I> = WarpHeaderInput<I>;
    type Cookie<I> = WarpCookieInput<I>;
    type Auth<I> = WarpHeaderInput<I>;
    type Context<I> = WarpHeadInput<I>;
}

impl<Context, Inputs, Args, Transform, Answering, Output>
    HandlerEndpointAlg<Arc<Context>, Inputs, Args, Transform, Output> for WarpHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
    Inputs: WarpInputsAlg<Args> + Send + 'static,
    Args: Send + 'static,
    Output: Send + 'static,
    Transform: OutputKindAlg<Self, Output, Transform = Answering> + Send + 'static,
    Answering: OutputAlg<Output, Output = Response>,
{
    fn finish_handler<Handler>(&self, handler: Handler) -> <Self as HandlerAlg>::Endpoint
    where
        Handler: OperationAlg + ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
    {
        let context = Arc::clone(&self.context);
        let handler = Arc::new(handler);

        WarpEndpoint::new(move |request| {
            let context = Arc::clone(&context);
            let handler = Arc::clone(&handler);
            Box::pin(async move {
                let inputs = match Inputs::extract(&request).await {
                    Ok(inputs) => inputs,
                    Err(error) => return unreadable(&error),
                };
                let output = handler.apply(context, inputs).await;

                Answering::output(output)
            })
        })
    }
}

/// Answers with what reading a request meant when it could not be read.
fn unreadable(error: &impl HttpErrorAlg) -> Response {
    let message = error.http_message();

    warp::http::Response::builder()
        .status(crate::warp_status(error.http_status()))
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(message.into_bytes().into())
        .unwrap_or_else(|_| warp::http::Response::new(Vec::new().into()))
}

impl<Context> SelectorAlg for WarpHandlerImpl<Context> {
    type Selector = WarpSelector;

    fn identity(&self) -> Self::Selector {
        SelectorAlg::identity(&WarpRouteImpl)
    }

    fn compose(&self, first: Self::Selector, second: Self::Selector) -> Self::Selector {
        SelectorAlg::compose(&WarpRouteImpl, first, second)
    }
}

impl<Context> RouteAlg for WarpHandlerImpl<Context> {
    type Route = WarpRoute;
    type Selector = WarpSelector;
    type Endpoint = WarpEndpoint;

    fn initial(&self) -> Self::Route {
        RouteAlg::initial(&WarpRouteImpl)
    }

    fn coproduct(&self, left: Self::Route, right: Self::Route) -> Self::Route {
        RouteAlg::coproduct(&WarpRouteImpl, left, right)
    }

    fn precompose(&self, selector: Self::Selector, route: Self::Route) -> Self::Route {
        RouteAlg::precompose(&WarpRouteImpl, selector, route)
    }

    fn lift(&self, endpoint: Self::Endpoint) -> Self::Route {
        RouteAlg::lift(&WarpRouteImpl, endpoint)
    }
}

impl<Context> HttpSelectorAlg for WarpHandlerImpl<Context> {
    type Selector = WarpSelector;

    fn http_method(&self, method: HttpMethod) -> Self::Selector {
        HttpSelectorAlg::http_method(&WarpRouteImpl, method)
    }

    fn http_path(&self, path: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_path(&WarpRouteImpl, path)
    }

    fn http_prefix(&self, prefix: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_prefix(&WarpRouteImpl, prefix)
    }
}
