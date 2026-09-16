//! Reaches one endpoint: it reads the arguments its roles state, applies the operation, and answers.

use crate::RocketAnswer;
use crate::input::{
    RocketBodyInput, RocketCookieInput, RocketFormInput, RocketHeadInput, RocketHeaderInput, RocketInputsAlg,
    RocketMultipartInput, RocketPathInput, RocketQueryInput, RocketRawBodyInput,
};
use crate::route::{RocketEndpoint, RocketRoute, RocketRouteImpl, RocketSelector};
use alux_ext::{ApplyAlg, HandlerContextAlg, OperationAlg};
use alux_http::{
    HandlerAlg, HandlerEndpointAlg, HttpErrorAlg, HttpInputAlg, HttpMethod, HttpSelectorAlg, OutputAlg, OutputKindAlg,
    RouteAlg, RoutePath, SelectorAlg,
};
use std::sync::Arc;

/// Interprets typed HTTP programs as executable Rocket routes.
pub struct RocketHandlerImpl<Context> {
    context: Arc<Context>,
}

impl<Context> RocketHandlerImpl<Context> {
    /// Creates an interpreter owning a newly shared context.
    pub fn new(context: Context) -> Self {
        Self { context: Arc::new(context) }
    }

    /// Creates an interpreter from an existing shared context.
    pub fn from_shared(context: Arc<Context>) -> Self {
        Self { context }
    }
}

impl<Context> HandlerContextAlg<Context> for RocketHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
{
    type Handle = Arc<Context>;
}

impl<Context> HandlerAlg for RocketHandlerImpl<Context> {
    type Endpoint = RocketEndpoint;
}

impl<Context> HttpInputAlg for RocketHandlerImpl<Context> {
    type Path<I> = RocketPathInput<I>;
    type Query<I> = RocketQueryInput<I>;
    type Body<I> = RocketBodyInput<I>;
    type Form<I> = RocketFormInput<I>;
    type Multipart<I> = RocketMultipartInput<I>;
    type RawBody<I> = RocketRawBodyInput<I>;
    // Rocket reads a header, an authentication value, and an endpoint context from the headers.
    type Header<I> = RocketHeaderInput<I>;
    type Cookie<I> = RocketCookieInput<I>;
    type Auth<I> = RocketHeaderInput<I>;
    type Context<I> = RocketHeadInput<I>;
}

impl<Context, Inputs, Args, Transform, Output> HandlerEndpointAlg<Arc<Context>, Inputs, Args, Transform, Output>
    for RocketHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
    Inputs: RocketInputsAlg<Args> + Send + 'static,
    Args: Send + 'static,
    Output: Send + 'static,
    Transform: OutputKindAlg<Self, Output> + Send + 'static,
    <Transform as OutputKindAlg<Self, Output>>::Transform: OutputAlg<Output, Output = RocketAnswer>,
{
    fn finish_handler<Handler>(&self, handler: Handler) -> <Self as HandlerAlg>::Endpoint
    where
        Handler: OperationAlg + ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
    {
        let context = Arc::clone(&self.context);
        let handler = Arc::new(handler);

        RocketEndpoint::new(move |request| {
            let context = Arc::clone(&context);
            let handler = Arc::clone(&handler);
            Box::pin(async move {
                let inputs = match Inputs::extract(&request).await {
                    Ok(inputs) => inputs,
                    Err(error) => return unreadable(&error),
                };
                let output = handler.apply(context, inputs).await;

                <Transform as OutputKindAlg<Self, Output>>::Transform::output(output)
            })
        })
    }
}

/// Answers with what reading a request meant when it could not be read.
fn unreadable(error: &impl HttpErrorAlg) -> RocketAnswer {
    RocketAnswer::content(error.http_status(), "text/plain; charset=utf-8", error.http_message())
}

impl<Context> SelectorAlg for RocketHandlerImpl<Context> {
    type Selector = RocketSelector;

    fn identity(&self) -> Self::Selector {
        SelectorAlg::identity(&RocketRouteImpl)
    }

    fn compose(&self, first: Self::Selector, second: Self::Selector) -> Self::Selector {
        SelectorAlg::compose(&RocketRouteImpl, first, second)
    }
}

impl<Context> RouteAlg for RocketHandlerImpl<Context> {
    type Route = RocketRoute;
    type Selector = RocketSelector;
    type Endpoint = RocketEndpoint;

    fn initial(&self) -> Self::Route {
        RouteAlg::initial(&RocketRouteImpl)
    }

    fn coproduct(&self, left: Self::Route, right: Self::Route) -> Self::Route {
        RouteAlg::coproduct(&RocketRouteImpl, left, right)
    }

    fn precompose(&self, selector: Self::Selector, route: Self::Route) -> Self::Route {
        RouteAlg::precompose(&RocketRouteImpl, selector, route)
    }

    fn lift(&self, endpoint: Self::Endpoint) -> Self::Route {
        RouteAlg::lift(&RocketRouteImpl, endpoint)
    }
}

impl<Context> HttpSelectorAlg for RocketHandlerImpl<Context> {
    type Selector = RocketSelector;

    fn http_method(&self, method: HttpMethod) -> Self::Selector {
        HttpSelectorAlg::http_method(&RocketRouteImpl, method)
    }

    fn http_path(&self, path: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_path(&RocketRouteImpl, path)
    }

    fn http_prefix(&self, prefix: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_prefix(&RocketRouteImpl, prefix)
    }
}
