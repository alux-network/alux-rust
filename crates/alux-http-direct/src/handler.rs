//! Reaches one endpoint: it reads the arguments its roles state, applies the operation, and answers.

use crate::input::DirectInputsAlg;
use crate::input::{
    DirectBodyInput, DirectCookieInput, DirectFormInput, DirectHeadInput, DirectHeaderInput, DirectMultipartInput,
    DirectPathInput, DirectQueryInput, DirectRawBodyInput,
};
use crate::route::{DirectEndpoint, DirectRoute, DirectRouteImpl, DirectSelector};
use crate::{DirectRequest, DirectResponse};
use alux_ext::{ApplyAlg, HandlerContextAlg, OperationAlg};
use alux_http::{
    HandlerAlg, HandlerEndpointAlg, HttpInputAlg, HttpMethod, HttpSelectorAlg, OutputAlg, OutputKindAlg, RouteAlg,
    RoutePath, SelectorAlg,
};
use std::sync::Arc;

/// Interprets typed HTTP programs as a surface that answers requests itself.
pub struct DirectHandlerImpl<Context> {
    context: Arc<Context>,
}

impl<Context> DirectHandlerImpl<Context> {
    /// Creates an interpreter owning a newly shared context.
    pub fn new(context: Context) -> Self {
        Self { context: Arc::new(context) }
    }

    /// Creates an interpreter from an existing shared context.
    pub fn from_shared(context: Arc<Context>) -> Self {
        Self { context }
    }
}

impl<Context> HandlerContextAlg<Context> for DirectHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
{
    type Handle = Arc<Context>;
}

impl<Context> HandlerAlg for DirectHandlerImpl<Context> {
    type Endpoint = DirectEndpoint;
}

impl<Context> HttpInputAlg for DirectHandlerImpl<Context> {
    type Path<I> = DirectPathInput<I>;
    type Query<I> = DirectQueryInput<I>;
    type Body<I> = DirectBodyInput<I>;
    type Form<I> = DirectFormInput<I>;
    type Multipart<I> = DirectMultipartInput<I>;
    type RawBody<I> = DirectRawBodyInput<I>;
    // A header, an authentication value, and an endpoint context are all read from what arrived
    // before the body.
    type Header<I> = DirectHeaderInput<I>;
    type Cookie<I> = DirectCookieInput<I>;
    type Auth<I> = DirectHeaderInput<I>;
    type Context<I> = DirectHeadInput<I>;
}

impl<Context, Inputs, Args, Transform, Answering, Output>
    HandlerEndpointAlg<Arc<Context>, Inputs, Args, Transform, Output> for DirectHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
    Inputs: DirectInputsAlg<Args> + Send + 'static,
    Args: Send + 'static,
    Output: Send + 'static,
    Transform: OutputKindAlg<Self, Output, Transform = Answering> + Send + 'static,
    Answering: OutputAlg<Output, Output = DirectResponse>,
{
    fn finish_handler<Handler>(&self, handler: Handler) -> <Self as HandlerAlg>::Endpoint
    where
        Handler: OperationAlg + ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
    {
        let context = Arc::clone(&self.context);
        let handler = Arc::new(handler);
        DirectEndpoint::new(move |request: DirectRequest, captures: Vec<String>| {
            let context = Arc::clone(&context);
            let handler = Arc::clone(&handler);
            Box::pin(async move {
                let inputs = match Inputs::extract(&request, &captures).await {
                    Ok(inputs) => inputs,
                    Err(error) => return error.into(),
                };
                let output = handler.apply(context, inputs).await;

                Answering::output(output)
            })
        })
    }
}

impl<Context> SelectorAlg for DirectHandlerImpl<Context> {
    type Selector = DirectSelector;

    fn identity(&self) -> Self::Selector {
        SelectorAlg::identity(&DirectRouteImpl)
    }

    fn compose(&self, first: Self::Selector, second: Self::Selector) -> Self::Selector {
        SelectorAlg::compose(&DirectRouteImpl, first, second)
    }
}

impl<Context> RouteAlg for DirectHandlerImpl<Context> {
    type Route = DirectRoute;
    type Selector = DirectSelector;
    type Endpoint = DirectEndpoint;

    fn initial(&self) -> Self::Route {
        RouteAlg::initial(&DirectRouteImpl)
    }

    fn coproduct(&self, left: Self::Route, right: Self::Route) -> Self::Route {
        RouteAlg::coproduct(&DirectRouteImpl, left, right)
    }

    fn precompose(&self, selector: Self::Selector, route: Self::Route) -> Self::Route {
        RouteAlg::precompose(&DirectRouteImpl, selector, route)
    }

    fn lift(&self, endpoint: Self::Endpoint) -> Self::Route {
        RouteAlg::lift(&DirectRouteImpl, endpoint)
    }
}

impl<Context> HttpSelectorAlg for DirectHandlerImpl<Context> {
    type Selector = DirectSelector;

    fn http_method(&self, method: HttpMethod) -> Self::Selector {
        HttpSelectorAlg::http_method(&DirectRouteImpl, method)
    }

    fn http_path(&self, path: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_path(&DirectRouteImpl, path)
    }

    fn http_prefix(&self, prefix: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_prefix(&DirectRouteImpl, prefix)
    }
}
