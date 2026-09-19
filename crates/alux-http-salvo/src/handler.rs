//! Reaches one endpoint: it reads the arguments its roles state, applies the operation, and answers.

use crate::input::{
    SalvoBodyInput, SalvoCookieInput, SalvoFormInput, SalvoHeadInput, SalvoHeaderInput, SalvoInputsAlg,
    SalvoMultipartInput, SalvoPathInput, SalvoQueryInput, SalvoRawBodyInput,
};
use crate::route::{SalvoEndpoint, SalvoReachAlg, SalvoRoute, SalvoRouteImpl, SalvoSelector};
use alux_ext::{ApplyAlg, HandlerContextAlg, OperationAlg};
use alux_http::{
    HandlerAlg, HandlerEndpointAlg, HttpInputAlg, HttpMethod, HttpSelectorAlg, OutputAlg, OutputKindAlg, RouteAlg,
    RoutePath, SelectorAlg,
};
use core::marker::PhantomData;
use salvo::{Request, Response, Scribe};
use std::sync::Arc;

/// Interprets typed HTTP programs as executable Salvo routes.
pub struct SalvoHandlerImpl<Context> {
    context: Arc<Context>,
}

impl<Context> SalvoHandlerImpl<Context> {
    /// Creates an interpreter owning a newly shared context.
    pub fn new(context: Context) -> Self {
        Self { context: Arc::new(context) }
    }

    /// Creates an interpreter from an existing shared context.
    pub fn from_shared(context: Arc<Context>) -> Self {
        Self { context }
    }
}

impl<Context> HandlerContextAlg<Context> for SalvoHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
{
    type Handle = Arc<Context>;
}

impl<Context> HandlerAlg for SalvoHandlerImpl<Context> {
    type Endpoint = SalvoEndpoint;
}

impl<Context> HttpInputAlg for SalvoHandlerImpl<Context> {
    type Path<I> = SalvoPathInput<I>;
    type Query<I> = SalvoQueryInput<I>;
    type Body<I> = SalvoBodyInput<I>;
    type Form<I> = SalvoFormInput<I>;
    type Multipart<I> = SalvoMultipartInput<I>;
    type RawBody<I> = SalvoRawBodyInput<I>;
    // Salvo reads a header, an authentication value, and an endpoint context from the request head.
    type Header<I> = SalvoHeaderInput<I>;
    type Cookie<I> = SalvoCookieInput<I>;
    type Auth<I> = SalvoHeaderInput<I>;
    type Context<I> = SalvoHeadInput<I>;
}

/// What one endpoint does with a request: read its arguments, apply, and answer.
struct Reaching<Context, Handler, Inputs, Args, Transform> {
    context: Arc<Context>,
    handler: Arc<Handler>,
    marker: PhantomData<fn(Inputs, Args, Transform)>,
}

impl<Context, Handler, Inputs, Args, Transform, Output> SalvoReachAlg
    for Reaching<Context, Handler, Inputs, Args, Transform>
where
    Context: Send + Sync + 'static,
    Handler: ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
    Inputs: SalvoInputsAlg<Args> + Send + Sync + 'static,
    Args: Send + 'static,
    Output: Send + 'static,
    Transform: OutputAlg<Output, Output = Response> + Send + Sync + 'static,
{
    fn reach<'a>(
        &'a self,
        request: &'a mut Request,
    ) -> core::pin::Pin<Box<dyn core::future::Future<Output = Response> + Send + 'a>> {
        Box::pin(async move {
            let inputs = match Inputs::extract(request).await {
                Ok(inputs) => inputs,
                Err(rejection) => {
                    let mut answer = Response::new();
                    rejection.render(&mut answer);

                    return answer;
                }
            };
            let output = self.handler.apply(Arc::clone(&self.context), inputs).await;

            Transform::output(output)
        })
    }
}

impl<Context, Inputs, Args, Transform, Answering, Output>
    HandlerEndpointAlg<Arc<Context>, Inputs, Args, Transform, Output> for SalvoHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
    Inputs: SalvoInputsAlg<Args> + Send + Sync + 'static,
    Args: Send + 'static,
    Output: Send + 'static,
    Transform: OutputKindAlg<Self, Output, Transform = Answering>,
    Answering: OutputAlg<Output, Output = Response> + Send + Sync + 'static,
{
    fn finish_handler<Handler>(&self, handler: Handler) -> <Self as HandlerAlg>::Endpoint
    where
        Handler: OperationAlg + ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
    {
        SalvoEndpoint::new(Reaching::<Context, Handler, Inputs, Args, Answering> {
            context: Arc::clone(&self.context),
            handler: Arc::new(handler),
            marker: PhantomData,
        })
    }
}

impl<Context> SelectorAlg for SalvoHandlerImpl<Context> {
    type Selector = SalvoSelector;

    fn identity(&self) -> Self::Selector {
        SelectorAlg::identity(&SalvoRouteImpl)
    }

    fn compose(&self, first: Self::Selector, second: Self::Selector) -> Self::Selector {
        SelectorAlg::compose(&SalvoRouteImpl, first, second)
    }
}

impl<Context> RouteAlg for SalvoHandlerImpl<Context> {
    type Route = SalvoRoute;
    type Selector = SalvoSelector;
    type Endpoint = SalvoEndpoint;

    fn initial(&self) -> Self::Route {
        RouteAlg::initial(&SalvoRouteImpl)
    }

    fn coproduct(&self, left: Self::Route, right: Self::Route) -> Self::Route {
        RouteAlg::coproduct(&SalvoRouteImpl, left, right)
    }

    fn precompose(&self, selector: Self::Selector, route: Self::Route) -> Self::Route {
        RouteAlg::precompose(&SalvoRouteImpl, selector, route)
    }

    fn lift(&self, endpoint: Self::Endpoint) -> Self::Route {
        RouteAlg::lift(&SalvoRouteImpl, endpoint)
    }
}

impl<Context> HttpSelectorAlg for SalvoHandlerImpl<Context> {
    type Selector = SalvoSelector;

    fn http_method(&self, method: HttpMethod) -> Self::Selector {
        HttpSelectorAlg::http_method(&SalvoRouteImpl, method)
    }

    fn http_path(&self, path: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_path(&SalvoRouteImpl, path)
    }

    fn http_prefix(&self, prefix: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_prefix(&SalvoRouteImpl, prefix)
    }
}
