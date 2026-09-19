use crate::input::{
    AxumBodyInput, AxumCookieInput, AxumFormInput, AxumHeaderInput, AxumInputsAlg, AxumMultipartInput, AxumPartsInput,
    AxumPathInput, AxumQueryInput, AxumRawBodyInput,
};
use crate::route::{AxumEndpoint, AxumRoute, AxumRouteImpl, AxumSelector};
use alux_ext::{ApplyAlg, HandlerContextAlg, OperationAlg};
use alux_http::{
    HandlerAlg, HandlerEndpointAlg, HttpInputAlg, HttpMethod, HttpSelectorAlg, OutputAlg, OutputKindAlg, RouteAlg,
    RoutePath, SelectorAlg,
};
use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use core::convert::Infallible;
use std::sync::Arc;
use tower::service_fn;

/// Interprets typed HTTP programs as executable axum routes.
pub struct AxumHandlerImpl<Context> {
    context: Arc<Context>,
}

impl<Context> AxumHandlerImpl<Context> {
    /// Creates an interpreter owning a newly shared context.
    pub fn new(context: Context) -> Self {
        Self { context: Arc::new(context) }
    }

    /// Creates an interpreter from an existing shared context.
    pub fn from_shared(context: Arc<Context>) -> Self {
        Self { context }
    }
}

impl<Context> HandlerContextAlg<Context> for AxumHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
{
    type Handle = Arc<Context>;
}

impl<Context> HandlerAlg for AxumHandlerImpl<Context> {
    type Endpoint = AxumEndpoint;
}

impl<Context> HttpInputAlg for AxumHandlerImpl<Context> {
    type Path<I> = AxumPathInput<I>;
    type Query<I> = AxumQueryInput<I>;
    type Body<I> = AxumBodyInput<I>;
    type Form<I> = AxumFormInput<I>;
    type Multipart<I> = AxumMultipartInput<I>;
    type RawBody<I> = AxumRawBodyInput<I>;
    // axum reads a header, an authentication value, and an endpoint context from the request head.
    type Header<I> = AxumHeaderInput<I>;
    type Cookie<I> = AxumCookieInput<I>;
    type Auth<I> = AxumHeaderInput<I>;
    type Context<I> = AxumPartsInput<I>;
}

impl<Context, Inputs, Args, Transform, Answering, Answered, Output>
    HandlerEndpointAlg<Arc<Context>, Inputs, Args, Transform, Output> for AxumHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
    Inputs: AxumInputsAlg<Args> + Send + 'static,
    Args: Send + 'static,
    Output: Send + 'static,
    Transform: OutputKindAlg<Self, Output, Transform = Answering> + Send + 'static,
    Answering: OutputAlg<Output, Output = Answered>,
    Answered: IntoResponse,
{
    fn finish_handler<Handler>(&self, handler: Handler) -> <Self as HandlerAlg>::Endpoint
    where
        Handler: OperationAlg + ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
    {
        let context = Arc::clone(&self.context);
        let handler = Arc::new(handler);
        AxumEndpoint::new(service_fn(move |request: Request| {
            let context = Arc::clone(&context);
            let handler = Arc::clone(&handler);
            async move {
                let (mut parts, body) = request.into_parts();
                let mut body = Some(body);
                let inputs = match Inputs::extract(&mut parts, &mut body).await {
                    Ok(inputs) => inputs,
                    Err(rejection) => return Ok::<Response, Infallible>(rejection),
                };
                let output = handler.apply(context, inputs).await;

                Ok(Answering::output(output).into_response())
            }
        }))
    }
}

impl<Context> SelectorAlg for AxumHandlerImpl<Context> {
    type Selector = AxumSelector;

    fn identity(&self) -> Self::Selector {
        SelectorAlg::identity(&AxumRouteImpl)
    }

    fn compose(&self, first: Self::Selector, second: Self::Selector) -> Self::Selector {
        SelectorAlg::compose(&AxumRouteImpl, first, second)
    }
}

impl<Context> RouteAlg for AxumHandlerImpl<Context> {
    type Route = AxumRoute;
    type Selector = AxumSelector;
    type Endpoint = AxumEndpoint;

    fn initial(&self) -> Self::Route {
        RouteAlg::initial(&AxumRouteImpl)
    }

    fn coproduct(&self, left: Self::Route, right: Self::Route) -> Self::Route {
        RouteAlg::coproduct(&AxumRouteImpl, left, right)
    }

    fn precompose(&self, selector: Self::Selector, route: Self::Route) -> Self::Route {
        RouteAlg::precompose(&AxumRouteImpl, selector, route)
    }

    fn lift(&self, endpoint: Self::Endpoint) -> Self::Route {
        RouteAlg::lift(&AxumRouteImpl, endpoint)
    }
}

impl<Context> HttpSelectorAlg for AxumHandlerImpl<Context> {
    type Selector = AxumSelector;

    fn http_method(&self, method: HttpMethod) -> Self::Selector {
        HttpSelectorAlg::http_method(&AxumRouteImpl, method)
    }

    fn http_path(&self, path: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_path(&AxumRouteImpl, path)
    }

    fn http_prefix(&self, prefix: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_prefix(&AxumRouteImpl, prefix)
    }
}
