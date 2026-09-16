use alux_http::{
    HttpMethod, HttpSelectorAlg, PathSyntaxAlg, RouteAlg, RoutePath, SelectorAlg, compose_path, describe_path,
};
use poem::endpoint::BoxEndpoint;
use poem::http::Method;
use poem::{Endpoint, EndpointExt, Response, Route, RouteMethod};
use std::collections::BTreeMap;

/// Spells route parameters the way Poem's router reads them.
struct PoemPath;

impl PathSyntaxAlg for PoemPath {
    fn param(&self, name: &str) -> String {
        format!(":{name}")
    }

    fn tail(&self, name: &str) -> String {
        format!("*{name}")
    }
}

/// Interprets a request method as the one Poem routes on.
fn poem_method(method: HttpMethod) -> Method {
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
enum PoemSelectorPart {
    Method(HttpMethod),
    Path(RoutePath),
    Prefix(RoutePath),
}

/// Carries route-selection meaning before it is interpreted by Poem.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PoemSelector {
    parts: Vec<PoemSelectorPart>,
}

impl PoemSelector {
    /// Returns the composed absolute path this selector matches.
    ///
    /// The description states the path in the spelling every interpretation shares, so that two
    /// interpretations of one program describe one surface. Poem's router is handed its own
    /// spelling instead.
    pub fn path(&self) -> String {
        describe_path(self.paths())
    }

    /// Returns the composed path in the spelling Poem's router reads.
    pub(crate) fn poem_path(&self) -> String {
        compose_path(self.paths(), &PoemPath)
    }

    fn paths(&self) -> impl Iterator<Item = &RoutePath> {
        self.parts.iter().filter_map(|part| match part {
            PoemSelectorPart::Path(path) | PoemSelectorPart::Prefix(path) => Some(path),
            PoemSelectorPart::Method(_) => None,
        })
    }

    /// Returns the selected method and path, using `*` when no method is selected.
    pub fn label(&self) -> String {
        let method = self.method().map_or("*", HttpMethod::label);
        format!("{method} {}", self.path())
    }

    pub(crate) fn method(&self) -> Option<HttpMethod> {
        self.parts.iter().rev().find_map(|part| match part {
            PoemSelectorPart::Method(method) => Some(*method),
            PoemSelectorPart::Path(_) | PoemSelectorPart::Prefix(_) => None,
        })
    }
}

/// Erases a native Poem endpoint for route composition.
pub struct PoemEndpoint(pub(crate) BoxEndpoint<'static, Response>);

impl PoemEndpoint {
    /// Erases a Poem endpoint while normalizing its output to `Response`.
    pub fn new<E>(endpoint: E) -> Self
    where
        E: Endpoint + 'static,
    {
        Self(endpoint.map_to_response().boxed())
    }
}

struct PoemRouteEntry {
    selector: PoemSelector,
    endpoint: PoemEndpoint,
}

/// Carries a composable collection of native Poem endpoints.
#[derive(Default)]
pub struct PoemRoute {
    entries: Vec<PoemRouteEntry>,
}

impl PoemRoute {
    /// Returns each composed selector as `METHOD /path`, in declaration order.
    pub fn labels(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.label()).collect()
    }

    /// Returns each composed route path, in declaration order.
    pub fn paths(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.path()).collect()
    }

    /// Materializes the composed meaning as a native Poem route.
    pub fn into_poem(self) -> Route {
        let mut paths = BTreeMap::<String, Vec<(Option<HttpMethod>, PoemEndpoint)>>::new();
        for entry in self.entries {
            paths.entry(entry.selector.poem_path()).or_default().push((entry.selector.method(), entry.endpoint));
        }

        paths.into_iter().fold(Route::new(), |route, (path, entries)| {
            if entries.iter().all(|(method, _)| method.is_some()) {
                let endpoint = entries.into_iter().fold(RouteMethod::new(), |route, (method, endpoint)| match method {
                    Some(method) => route.method(poem_method(method), endpoint.0),
                    None => unreachable!("method presence was checked"),
                });
                route.at(path, endpoint)
            } else {
                entries.into_iter().fold(route, |route, (_, endpoint)| route.at(path.clone(), endpoint.0))
            }
        })
    }
}

/// Interprets categorical route composition as native Poem routing.
#[derive(Debug, Default)]
pub struct PoemRouteImpl;

impl SelectorAlg for PoemRouteImpl {
    type Selector = PoemSelector;

    fn identity(&self) -> PoemSelector {
        PoemSelector::default()
    }

    fn compose(&self, mut first: PoemSelector, second: PoemSelector) -> PoemSelector {
        first.parts.extend(second.parts);
        first
    }
}

impl RouteAlg for PoemRouteImpl {
    type Route = PoemRoute;
    type Selector = PoemSelector;
    type Endpoint = PoemEndpoint;

    fn initial(&self) -> PoemRoute {
        PoemRoute::default()
    }

    fn coproduct(&self, mut left: PoemRoute, right: PoemRoute) -> PoemRoute {
        left.entries.extend(right.entries);
        left
    }

    fn precompose(&self, selector: PoemSelector, mut route: PoemRoute) -> PoemRoute {
        for entry in &mut route.entries {
            entry.selector = self.compose(selector.clone(), core::mem::take(&mut entry.selector));
        }
        route
    }

    fn lift(&self, endpoint: PoemEndpoint) -> PoemRoute {
        PoemRoute { entries: vec![PoemRouteEntry { selector: self.identity(), endpoint }] }
    }
}

impl HttpSelectorAlg for PoemRouteImpl {
    type Selector = PoemSelector;

    fn http_method(&self, method: HttpMethod) -> PoemSelector {
        PoemSelector { parts: vec![PoemSelectorPart::Method(method)] }
    }

    fn http_path(&self, path: &RoutePath) -> PoemSelector {
        PoemSelector { parts: vec![PoemSelectorPart::Path(path.clone())] }
    }

    fn http_prefix(&self, prefix: &RoutePath) -> PoemSelector {
        PoemSelector { parts: vec![PoemSelectorPart::Prefix(prefix.clone())] }
    }
}

#[cfg(test)]
mod tests {
    use super::{PoemEndpoint, PoemRouteImpl, poem_method};
    use alux_http::{HttpMethod, HttpSelectorAlg, RouteAlg, RouteAlgExt, RoutePath, SelectorAlg};
    use poem::endpoint::make_sync;
    use poem::web::Path;
    use poem::{Endpoint, Request, get, handler};

    #[handler]
    fn item(Path(id): Path<u32>) -> String {
        id.to_string()
    }

    #[tokio::test]
    async fn nesting_precomposes_a_prefix_and_keeps_the_endpoint_reachable() {
        let alg = PoemRouteImpl;
        let health = alg.routes().at("/health", PoemEndpoint::new(get(make_sync(|_| "ok"))));
        let route = alg.routes().nest("/api", health).into_route();

        assert_eq!(route.paths(), ["/api/health"]);
        assert_eq!(route.labels(), ["* /api/health"]);

        let request = Request::builder().uri_str("/api/health").finish();
        let mut response = route.into_poem().call(request).await.unwrap();

        assert!(response.status().is_success());
        assert_eq!(response.take_body().into_string().await.unwrap(), "ok");
    }

    #[tokio::test]
    async fn routes_a_parameter_written_in_another_router_s_spelling() {
        let alg = PoemRouteImpl;
        let selector = alg.compose(alg.http_method(HttpMethod::Get), alg.http_path(&RoutePath::parse("/item/{id}")));
        let route = alg.precompose(selector, alg.lift(PoemEndpoint::new(get(item))));

        // The surface is described in the shared spelling and routed in Poem's.
        assert_eq!(route.labels(), ["GET /item/{id}"]);

        let request = Request::builder().uri_str("/item/7").finish();
        let mut response = route.into_poem().call(request).await.unwrap();

        assert!(response.status().is_success());
        assert_eq!(response.take_body().into_string().await.unwrap(), "7");
    }

    #[tokio::test]
    async fn routes_every_request_method_the_specification_names() {
        let alg = PoemRouteImpl;
        let route = HttpMethod::ALL.iter().fold(alg.initial(), |route, method| {
            let selector = alg.compose(alg.http_method(*method), alg.http_path(&RoutePath::parse("/ping")));
            let endpoint = alg.lift(PoemEndpoint::new(make_sync(|_| "ok")));

            alg.coproduct(route, alg.precompose(selector, endpoint))
        });

        assert_eq!(route.labels().len(), HttpMethod::ALL.len());

        // One path answering on every method is one Poem `RouteMethod`, so each must dispatch.
        let poem = route.into_poem();
        for method in HttpMethod::ALL {
            let request = Request::builder().method(poem_method(*method)).uri_str("/ping").finish();
            let response = poem.call(request).await.unwrap();

            assert!(response.status().is_success(), "`{}` did not route", method.label());
        }
    }
}
