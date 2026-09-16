//! Composes selectors, and builds the Salvo router they state.

use alux_http::{
    HttpMethod, HttpSelectorAlg, PathSyntaxAlg, RouteAlg, RoutePath, SelectorAlg, compose_path, describe_path,
};
use core::future::Future;
use core::pin::Pin;
use salvo::http::Method;
use salvo::routing::filters::MethodFilter;
use salvo::{Depot, FlowCtrl, Handler, Request, Response, Router, async_trait};
use std::sync::Arc;

/// Spells route parameters the way Salvo's router reads them.
struct SalvoPath;

impl PathSyntaxAlg for SalvoPath {
    fn param(&self, name: &str) -> String {
        format!("{{{name}}}")
    }

    fn tail(&self, name: &str) -> String {
        format!("{{**{name}}}")
    }
}

/// Interprets a request method as the one Salvo routes on.
fn salvo_method(method: HttpMethod) -> Method {
    match method {
        HttpMethod::Get => Method::GET,
        HttpMethod::Post => Method::POST,
        HttpMethod::Put => Method::PUT,
        HttpMethod::Patch => Method::PATCH,
        HttpMethod::Delete => Method::DELETE,
        HttpMethod::Head => Method::HEAD,
        HttpMethod::Options => Method::OPTIONS,
        HttpMethod::Trace => Method::TRACE,
        HttpMethod::Connect => Method::CONNECT,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SalvoSelectorPart {
    Method(HttpMethod),
    Path(RoutePath),
    Prefix(RoutePath),
}

/// Carries route-selection meaning before it is interpreted by Salvo.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SalvoSelector {
    parts: Vec<SalvoSelectorPart>,
}

impl SalvoSelector {
    /// Returns the composed absolute path this selector matches.
    pub fn path(&self) -> String {
        describe_path(self.paths())
    }

    /// Returns the composed path in the spelling Salvo's router reads.
    pub(crate) fn salvo_path(&self) -> String {
        compose_path(self.paths(), &SalvoPath)
    }

    fn paths(&self) -> impl Iterator<Item = &RoutePath> {
        self.parts.iter().filter_map(|part| match part {
            SalvoSelectorPart::Path(path) | SalvoSelectorPart::Prefix(path) => Some(path),
            SalvoSelectorPart::Method(_) => None,
        })
    }

    /// Returns the selected method and path, using `*` when no method is selected.
    pub fn label(&self) -> String {
        let method = self.method().map_or("*", HttpMethod::label);
        format!("{method} {}", self.path())
    }

    pub(crate) fn method(&self) -> Option<HttpMethod> {
        self.parts.iter().rev().find_map(|part| match part {
            SalvoSelectorPart::Method(method) => Some(*method),
            SalvoSelectorPart::Path(_) | SalvoSelectorPart::Prefix(_) => None,
        })
    }
}

/// The answer one reached endpoint produces.
type Answer<'a> = Pin<Box<dyn Future<Output = Response> + Send + 'a>>;

/// Answers a request Salvo routed here.
pub(crate) trait SalvoReachAlg: Send + Sync + 'static {
    fn reach<'a>(&'a self, request: &'a mut Request) -> Answer<'a>;
}

/// Erases what one endpoint does with a request Salvo routed to it.
#[derive(Clone)]
pub struct SalvoEndpoint(Arc<dyn SalvoReachAlg>);

impl SalvoEndpoint {
    pub(crate) fn new(reach: impl SalvoReachAlg) -> Self {
        Self(Arc::new(reach))
    }
}

#[async_trait]
impl Handler for SalvoEndpoint {
    async fn handle(&self, request: &mut Request, _depot: &mut Depot, response: &mut Response, _ctrl: &mut FlowCtrl) {
        *response = self.0.reach(request).await;
    }
}

#[derive(Clone)]
struct SalvoRouteEntry {
    selector: SalvoSelector,
    endpoint: SalvoEndpoint,
}

/// Carries a composable collection of Salvo endpoints.
#[derive(Clone, Default)]
pub struct SalvoRoute {
    entries: Vec<SalvoRouteEntry>,
}

impl SalvoRoute {
    /// Returns each composed selector as `METHOD /path`, in declaration order.
    pub fn labels(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.label()).collect()
    }

    /// Returns each composed route path, in declaration order.
    pub fn paths(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.path()).collect()
    }

    /// Materializes the composed meaning as a native Salvo router.
    ///
    /// Salvo routes by pushing one router per endpoint, so a coproduct of endpoints is a router
    /// holding each of them and a method selector is one more filter on the way in.
    pub fn into_salvo(self) -> Router {
        self.entries.into_iter().fold(Router::new(), |router, entry| {
            let reached = Router::with_path(entry.selector.salvo_path());
            let reached = match entry.selector.method() {
                Some(method) => reached.filter(MethodFilter(salvo_method(method))),
                None => reached,
            };

            router.push(reached.goal(entry.endpoint))
        })
    }
}

/// Interprets categorical route composition as native Salvo routing.
#[derive(Debug, Default)]
pub struct SalvoRouteImpl;

impl SelectorAlg for SalvoRouteImpl {
    type Selector = SalvoSelector;

    fn identity(&self) -> SalvoSelector {
        SalvoSelector::default()
    }

    fn compose(&self, mut first: SalvoSelector, second: SalvoSelector) -> SalvoSelector {
        first.parts.extend(second.parts);
        first
    }
}

impl RouteAlg for SalvoRouteImpl {
    type Route = SalvoRoute;
    type Selector = SalvoSelector;
    type Endpoint = SalvoEndpoint;

    fn initial(&self) -> SalvoRoute {
        SalvoRoute::default()
    }

    fn coproduct(&self, mut left: SalvoRoute, right: SalvoRoute) -> SalvoRoute {
        left.entries.extend(right.entries);
        left
    }

    fn precompose(&self, selector: SalvoSelector, mut route: SalvoRoute) -> SalvoRoute {
        for entry in &mut route.entries {
            entry.selector = self.compose(selector.clone(), core::mem::take(&mut entry.selector));
        }
        route
    }

    fn lift(&self, endpoint: SalvoEndpoint) -> SalvoRoute {
        SalvoRoute { entries: vec![SalvoRouteEntry { selector: self.identity(), endpoint }] }
    }
}

impl HttpSelectorAlg for SalvoRouteImpl {
    type Selector = SalvoSelector;

    fn http_method(&self, method: HttpMethod) -> SalvoSelector {
        SalvoSelector { parts: vec![SalvoSelectorPart::Method(method)] }
    }

    fn http_path(&self, path: &RoutePath) -> SalvoSelector {
        SalvoSelector { parts: vec![SalvoSelectorPart::Path(path.clone())] }
    }

    fn http_prefix(&self, prefix: &RoutePath) -> SalvoSelector {
        SalvoSelector { parts: vec![SalvoSelectorPart::Prefix(prefix.clone())] }
    }
}
