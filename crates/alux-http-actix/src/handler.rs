//! Reaches one endpoint: it reads the arguments its roles state, applies the operation, and answers.

use crate::input::{
    ActixBodyInput, ActixCookieInput, ActixFormInput, ActixHeaderInput, ActixInputsAlg, ActixMultipartInput,
    ActixPathInput, ActixQueryInput, ActixRequestInput,
};
use crate::route::{ActixEndpoint, ActixRoute, ActixRouteImpl, ActixSelector};
use actix_web::web::Payload;
use actix_web::{HttpRequest, HttpResponse, Route};
use alux_ext::{ApplyAlg, HandlerContextAlg, OperationAlg};
use alux_http::{
    HandlerAlg, HandlerEndpointAlg, HttpInputAlg, HttpMethod, HttpSelectorAlg, OutputAlg, OutputKindAlg, RouteAlg,
    RoutePath, SelectorAlg,
};
use std::sync::Arc;

/// Interprets typed HTTP programs as executable actix-web routes.
///
/// Shared state reaches a handler as the semantic context the program already threads, not as
/// `actix_web::Data`, because a framework composing domain code is the inversion this design exists
/// to avoid.
pub struct ActixHandlerImpl<Context> {
    context: Arc<Context>,
}

impl<Context> ActixHandlerImpl<Context> {
    /// Creates an interpreter owning a newly shared context.
    pub fn new(context: Context) -> Self {
        Self { context: Arc::new(context) }
    }

    /// Creates an interpreter from an existing shared context.
    pub fn from_shared(context: Arc<Context>) -> Self {
        Self { context }
    }
}

impl<Context> HandlerContextAlg<Context> for ActixHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
{
    type Handle = Arc<Context>;
}

impl<Context> HandlerAlg for ActixHandlerImpl<Context> {
    type Endpoint = ActixEndpoint;
}

impl<Context> HttpInputAlg for ActixHandlerImpl<Context> {
    type Path<I> = ActixPathInput<I>;
    type Query<I> = ActixQueryInput<I>;
    type Body<I> = ActixBodyInput<I>;
    type Form<I> = ActixFormInput<I>;
    type Multipart<I> = ActixMultipartInput<I>;
    // actix-web reads an unread body, a header, an authentication value, and an endpoint context
    // through its own request extractor.
    type RawBody<I> = ActixRequestInput<I>;
    type Header<I> = ActixHeaderInput<I>;
    type Cookie<I> = ActixCookieInput<I>;
    type Auth<I> = ActixHeaderInput<I>;
    type Context<I> = ActixRequestInput<I>;
}

impl<Context, Inputs, Args, Transform, Answering, Output>
    HandlerEndpointAlg<Arc<Context>, Inputs, Args, Transform, Output> for ActixHandlerImpl<Context>
where
    Context: Send + Sync + 'static,
    Inputs: ActixInputsAlg<Args> + 'static,
    Args: 'static,
    Output: 'static,
    Transform: OutputKindAlg<Self, Output, Transform = Answering> + 'static,
    Answering: OutputAlg<Output, Output = HttpResponse>,
{
    fn finish_handler<Handler>(&self, handler: Handler) -> <Self as HandlerAlg>::Endpoint
    where
        Handler: OperationAlg + ApplyAlg<Arc<Context>, Args, Output = Output> + Send + Sync + 'static,
    {
        let context = Arc::clone(&self.context);
        let handler = Arc::new(handler);

        // A route is made again for every worker, so what is held is the making of one.
        ActixEndpoint::new(move || {
            let context = Arc::clone(&context);
            let handler = Arc::clone(&handler);

            Route::new().to(move |request: HttpRequest, payload: Payload| {
                let context = Arc::clone(&context);
                let handler = Arc::clone(&handler);
                async move {
                    // What a handler is given is the whole payload; what reads it is the roles.
                    let mut payload = payload.into_inner();
                    let inputs = match Inputs::extract(&request, &mut payload).await {
                        Ok(inputs) => inputs,
                        Err(rejection) => return rejection.error_response(),
                    };
                    let output = handler.apply(context, inputs).await;

                    Answering::output(output)
                }
            })
        })
    }
}

impl<Context> SelectorAlg for ActixHandlerImpl<Context> {
    type Selector = ActixSelector;

    fn identity(&self) -> Self::Selector {
        SelectorAlg::identity(&ActixRouteImpl)
    }

    fn compose(&self, first: Self::Selector, second: Self::Selector) -> Self::Selector {
        SelectorAlg::compose(&ActixRouteImpl, first, second)
    }
}

impl<Context> RouteAlg for ActixHandlerImpl<Context> {
    type Route = ActixRoute;
    type Selector = ActixSelector;
    type Endpoint = ActixEndpoint;

    fn initial(&self) -> Self::Route {
        RouteAlg::initial(&ActixRouteImpl)
    }

    fn coproduct(&self, left: Self::Route, right: Self::Route) -> Self::Route {
        RouteAlg::coproduct(&ActixRouteImpl, left, right)
    }

    fn precompose(&self, selector: Self::Selector, route: Self::Route) -> Self::Route {
        RouteAlg::precompose(&ActixRouteImpl, selector, route)
    }

    fn lift(&self, endpoint: Self::Endpoint) -> Self::Route {
        RouteAlg::lift(&ActixRouteImpl, endpoint)
    }
}

impl<Context> HttpSelectorAlg for ActixHandlerImpl<Context> {
    type Selector = ActixSelector;

    fn http_method(&self, method: HttpMethod) -> Self::Selector {
        HttpSelectorAlg::http_method(&ActixRouteImpl, method)
    }

    fn http_path(&self, path: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_path(&ActixRouteImpl, path)
    }

    fn http_prefix(&self, prefix: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_prefix(&ActixRouteImpl, prefix)
    }
}
