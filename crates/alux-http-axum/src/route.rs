use alux_http::{
    HttpMethod, HttpSelectorAlg, PathSyntaxAlg, RouteAlg, RoutePath, SelectorAlg, compose_path, describe_path,
};
use axum::Router;
use axum::extract::Request;
use axum::response::Response;
use axum::routing::{MethodFilter, MethodRouter};
use core::convert::Infallible;
use std::collections::BTreeMap;
use tower::Service;
use tower::util::BoxCloneSyncService;

/// Spells route parameters the way axum's router reads them.
///
/// axum reads the spelling a described surface is already stated in, so this interpretation renders
/// a path for its router exactly as it describes one.
struct AxumPath;

impl PathSyntaxAlg for AxumPath {
    fn param(&self, name: &str) -> String {
        format!("{{{name}}}")
    }

    fn tail(&self, name: &str) -> String {
        format!("{{*{name}}}")
    }
}

/// Interprets a request method as the filter axum routes on.
fn axum_method(method: HttpMethod) -> MethodFilter {
    match method {
        HttpMethod::Get => MethodFilter::GET,
        HttpMethod::Post => MethodFilter::POST,
        HttpMethod::Put => MethodFilter::PUT,
        HttpMethod::Patch => MethodFilter::PATCH,
        HttpMethod::Delete => MethodFilter::DELETE,
        HttpMethod::Head => MethodFilter::HEAD,
        HttpMethod::Options => MethodFilter::OPTIONS,
        HttpMethod::Trace => MethodFilter::TRACE,
        HttpMethod::Connect => MethodFilter::CONNECT,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AxumSelectorPart {
    Method(HttpMethod),
    Path(RoutePath),
    Prefix(RoutePath),
}

/// Carries route-selection meaning before it is interpreted by axum.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AxumSelector {
    parts: Vec<AxumSelectorPart>,
}

impl AxumSelector {
    /// Returns the composed absolute path this selector matches.
    pub fn path(&self) -> String {
        describe_path(self.paths())
    }

    /// Returns the composed path in the spelling axum's router reads.
    pub(crate) fn axum_path(&self) -> String {
        compose_path(self.paths(), &AxumPath)
    }

    fn paths(&self) -> impl Iterator<Item = &RoutePath> {
        self.parts.iter().filter_map(|part| match part {
            AxumSelectorPart::Path(path) | AxumSelectorPart::Prefix(path) => Some(path),
            AxumSelectorPart::Method(_) => None,
        })
    }

    /// Returns the selected method and path, using `*` when no method is selected.
    pub fn label(&self) -> String {
        let method = self.method().map_or("*", HttpMethod::label);
        format!("{method} {}", self.path())
    }

    pub(crate) fn method(&self) -> Option<HttpMethod> {
        self.parts.iter().rev().find_map(|part| match part {
            AxumSelectorPart::Method(method) => Some(*method),
            AxumSelectorPart::Path(_) | AxumSelectorPart::Prefix(_) => None,
        })
    }
}

/// Erases a native axum service for route composition.
pub struct AxumEndpoint(BoxCloneSyncService<Request, Response, Infallible>);

impl AxumEndpoint {
    /// Erases a service answering a request, which is what axum routes to.
    pub fn new<Endpoint>(endpoint: Endpoint) -> Self
    where
        Endpoint: Service<Request, Response = Response, Error = Infallible> + Clone + Send + Sync + 'static,
        Endpoint::Future: Send + 'static,
    {
        Self(BoxCloneSyncService::new(endpoint))
    }
}

struct AxumRouteEntry {
    selector: AxumSelector,
    endpoint: AxumEndpoint,
}

/// Carries a composable collection of native axum services.
#[derive(Default)]
pub struct AxumRoute {
    entries: Vec<AxumRouteEntry>,
}

impl AxumRoute {
    /// Returns each composed selector as `METHOD /path`, in declaration order.
    pub fn labels(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.label()).collect()
    }

    /// Returns each composed route path, in declaration order.
    pub fn paths(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.path()).collect()
    }

    /// Materializes the composed meaning as a native axum router.
    ///
    /// axum states one method router per path, so the endpoints composed at one path are folded
    /// together. An endpoint composed without a method answers whatever the others do not.
    pub fn into_axum(self) -> Router {
        let mut paths = BTreeMap::<String, Vec<(Option<HttpMethod>, AxumEndpoint)>>::new();
        for entry in self.entries {
            paths.entry(entry.selector.axum_path()).or_default().push((entry.selector.method(), entry.endpoint));
        }

        paths.into_iter().fold(Router::new(), |router, (path, entries)| {
            let methods = entries.into_iter().fold(MethodRouter::new(), |methods, (method, endpoint)| match method {
                Some(method) => methods.on_service(axum_method(method), endpoint.0),
                None => methods.fallback_service(endpoint.0),
            });

            router.route(&path, methods)
        })
    }
}

/// Interprets categorical route composition as native axum routing.
#[derive(Debug, Default)]
pub struct AxumRouteImpl;

impl SelectorAlg for AxumRouteImpl {
    type Selector = AxumSelector;

    fn identity(&self) -> AxumSelector {
        AxumSelector::default()
    }

    fn compose(&self, mut first: AxumSelector, second: AxumSelector) -> AxumSelector {
        first.parts.extend(second.parts);
        first
    }
}

impl RouteAlg for AxumRouteImpl {
    type Route = AxumRoute;
    type Selector = AxumSelector;
    type Endpoint = AxumEndpoint;

    fn initial(&self) -> AxumRoute {
        AxumRoute::default()
    }

    fn coproduct(&self, mut left: AxumRoute, right: AxumRoute) -> AxumRoute {
        left.entries.extend(right.entries);
        left
    }

    fn precompose(&self, selector: AxumSelector, mut route: AxumRoute) -> AxumRoute {
        for entry in &mut route.entries {
            entry.selector = self.compose(selector.clone(), core::mem::take(&mut entry.selector));
        }
        route
    }

    fn lift(&self, endpoint: AxumEndpoint) -> AxumRoute {
        AxumRoute { entries: vec![AxumRouteEntry { selector: self.identity(), endpoint }] }
    }
}

impl HttpSelectorAlg for AxumRouteImpl {
    type Selector = AxumSelector;

    fn http_method(&self, method: HttpMethod) -> AxumSelector {
        AxumSelector { parts: vec![AxumSelectorPart::Method(method)] }
    }

    fn http_path(&self, path: &RoutePath) -> AxumSelector {
        AxumSelector { parts: vec![AxumSelectorPart::Path(path.clone())] }
    }

    fn http_prefix(&self, prefix: &RoutePath) -> AxumSelector {
        AxumSelector { parts: vec![AxumSelectorPart::Prefix(prefix.clone())] }
    }
}

#[cfg(test)]
mod tests {
    use super::{AxumEndpoint, AxumRouteImpl};
    use alux_http::{HttpMethod, HttpSelectorAlg, RouteAlg, RouteAlgExt, RoutePath, SelectorAlg};
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::Method;
    use axum::response::{IntoResponse, Response};
    use core::convert::Infallible;
    use tower::{ServiceExt, service_fn};

    fn answering() -> AxumEndpoint {
        AxumEndpoint::new(service_fn(|_: Request| async move { Ok::<Response, Infallible>("ok".into_response()) }))
    }

    fn requested(method: Method, path: &str) -> Request {
        Request::builder().method(method).uri(path).body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn nesting_precomposes_a_prefix_and_keeps_the_endpoint_reachable() {
        let alg = AxumRouteImpl;
        let health = alg.routes().at("/health", answering());
        let route = alg.routes().nest("/api", health).into_route();

        assert_eq!(route.paths(), ["/api/health"]);
        assert_eq!(route.labels(), ["* /api/health"]);

        let response = route.into_axum().oneshot(requested(Method::GET, "/api/health")).await.unwrap();

        assert!(response.status().is_success());
    }

    #[tokio::test]
    async fn routes_every_request_method_the_specification_names() {
        let alg = AxumRouteImpl;
        let route = HttpMethod::ALL.iter().fold(alg.initial(), |route, method| {
            let selector = alg.compose(alg.http_method(*method), alg.http_path(&RoutePath::parse("/ping")));

            alg.coproduct(route, alg.precompose(selector, alg.lift(answering())))
        });

        assert_eq!(route.labels().len(), HttpMethod::ALL.len());

        // One path answering on every method is one axum `MethodRouter`, so each must dispatch.
        let router = route.into_axum();
        for method in HttpMethod::ALL {
            let sent = Method::from_bytes(method.label().as_bytes()).unwrap();
            let response = router.clone().oneshot(requested(sent, "/ping")).await.unwrap();

            assert!(response.status().is_success(), "`{}` did not route", method.label());
        }
    }
}
