//! Reads one endpoint as the call a caller writes.

use crate::input::{
    TsBodyInput, TsCookieInput, TsFormInput, TsHeaderInput, TsInputsAlg, TsMultipartInput, TsPathInput, TsQueryInput,
    TsRawBodyInput, TsUnstatedInput,
};
use crate::output::TsOutputAlg;
use crate::route::{TsCall, TsHttpModule, TsRouteImpl, TsSelector};
use alux_ext::{ApplyAlg, HandlerContextAlg, OperationAlg};
use alux_http::{
    HandlerAlg, HandlerEndpointAlg, HttpInputAlg, HttpMethod, HttpSelectorAlg, OutputKindAlg, RouteAlg, RoutePath,
    SelectorAlg,
};
use alux_shape::{Spelling, words_of};
use alux_shape_typescript::TsShape;
use std::sync::Arc;

/// The package that interprets a program: what turns one into calls, installed once.
///
/// A generated module imports `endpoint` from here rather than restating how a request is made, so a
/// surface and what makes a surface's requests are upgraded separately.
pub const RUNTIME_PACKAGE: &str = "@alux-network/api";

/// Interprets typed HTTP programs as a TypeScript client module.
///
/// Nothing is applied and no request is made. Each endpoint is read for what a caller states and
/// what they receive, which is the same program an executing interpretation answers.
#[derive(Debug, Clone, Copy)]
pub struct TsHttpClient {
    shapes: TsShape,
    members: Spelling,
}

impl TsHttpClient {
    /// Emits a client whose member and parameter names are spelled this way.
    pub fn new(members: Spelling) -> Self {
        Self { shapes: TsShape::new(members), members }
    }
}

impl<Context> HandlerContextAlg<Context> for TsHttpClient
where
    Context: Send + Sync + 'static,
{
    // A client applies nothing, so the handle it names is only what the program's obligation asks
    // for: an owned reference to the domain the operations read.
    type Handle = Arc<Context>;
}

impl HandlerAlg for TsHttpClient {
    type Endpoint = TsCall;
}

impl HttpInputAlg for TsHttpClient {
    type Path<I> = TsPathInput<I>;
    type Query<I> = TsQueryInput<I>;
    type Body<I> = TsBodyInput<I>;
    type Form<I> = TsFormInput<I>;
    type Multipart<I> = TsMultipartInput<I>;
    type RawBody<I> = TsRawBodyInput<I>;
    type Header<I> = TsHeaderInput<I>;
    type Cookie<I> = TsCookieInput<I>;
    // A caller states an authentication value in a header; an endpoint context it never states.
    type Auth<I> = TsHeaderInput<I>;
    type Context<I> = TsUnstatedInput<I>;
}

impl<Handle, Inputs, Args, Transform, Output> HandlerEndpointAlg<Handle, Inputs, Args, Transform, Output>
    for TsHttpClient
where
    Inputs: TsInputsAlg,
    Transform: OutputKindAlg<Self, Output>,
    <Transform as OutputKindAlg<Self, Output>>::Transform: TsOutputAlg<Output>,
{
    fn finish_handler<Handler>(&self, _handler: Handler) -> TsCall
    where
        Handler: OperationAlg + ApplyAlg<Handle, Args, Output = Output> + Send + Sync + 'static,
    {
        // An argument name arrives as it was authored, so its words are read out of it and spelled
        // the way this surface spells names.
        let named = |name: &&'static str| self.members.spell(&words_of(name));
        let parameters = Handler::ARG_NAMES.iter().map(named).zip(Inputs::describe(&self.shapes)).collect();

        TsCall {
            name: self.members.spell(&words_of(Handler::NAME)),
            doc: Handler::DOC,
            parameters,
            answer: <Transform as OutputKindAlg<Self, Output>>::Transform::answer(&self.shapes),
        }
    }
}

impl SelectorAlg for TsHttpClient {
    type Selector = TsSelector;

    fn identity(&self) -> Self::Selector {
        SelectorAlg::identity(&TsRouteImpl)
    }

    fn compose(&self, first: Self::Selector, second: Self::Selector) -> Self::Selector {
        SelectorAlg::compose(&TsRouteImpl, first, second)
    }
}

impl RouteAlg for TsHttpClient {
    type Route = TsHttpModule;
    type Selector = TsSelector;
    type Endpoint = TsCall;

    fn initial(&self) -> Self::Route {
        RouteAlg::initial(&TsRouteImpl)
    }

    fn coproduct(&self, left: Self::Route, right: Self::Route) -> Self::Route {
        RouteAlg::coproduct(&TsRouteImpl, left, right)
    }

    fn precompose(&self, selector: Self::Selector, route: Self::Route) -> Self::Route {
        RouteAlg::precompose(&TsRouteImpl, selector, route)
    }

    fn lift(&self, endpoint: Self::Endpoint) -> Self::Route {
        RouteAlg::lift(&TsRouteImpl, endpoint)
    }
}

impl HttpSelectorAlg for TsHttpClient {
    type Selector = TsSelector;

    fn http_method(&self, method: HttpMethod) -> Self::Selector {
        HttpSelectorAlg::http_method(&TsRouteImpl, method)
    }

    fn http_path(&self, path: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_path(&TsRouteImpl, path)
    }

    fn http_prefix(&self, prefix: &RoutePath) -> Self::Selector {
        HttpSelectorAlg::http_prefix(&TsRouteImpl, prefix)
    }
}
