use alux_http::{HttpSelectorAlg, RouteAlg, SelectorAlg, append_path};
use poem::endpoint::BoxEndpoint;
use poem::{Endpoint, EndpointExt, Response, Route, RouteMethod};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PoemMethod {
    Get,
    Post,
}

impl PoemMethod {
    fn label(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PoemSelectorPart {
    Method(PoemMethod),
    Path(String),
    Prefix(String),
}

/// Carries route-selection meaning before it is interpreted by Poem.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PoemSelector {
    parts: Vec<PoemSelectorPart>,
}

impl PoemSelector {
    /// Returns the composed absolute path this selector matches.
    pub fn path(&self) -> String {
        let mut path = String::new();
        for part in &self.parts {
            if let PoemSelectorPart::Path(value) | PoemSelectorPart::Prefix(value) = part {
                append_path(&mut path, value);
            }
        }
        if path.is_empty() {
            path.push('/');
        }
        path
    }

    /// Returns the selected method and path, using `*` when no method is selected.
    pub fn label(&self) -> String {
        let method = self.method().map_or("*", PoemMethod::label);
        format!("{method} {}", self.path())
    }

    pub(crate) fn method(&self) -> Option<PoemMethod> {
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
        let mut paths = BTreeMap::<String, Vec<(Option<PoemMethod>, PoemEndpoint)>>::new();
        for entry in self.entries {
            paths.entry(entry.selector.path()).or_default().push((entry.selector.method(), entry.endpoint));
        }

        paths.into_iter().fold(Route::new(), |route, (path, entries)| {
            if entries.iter().all(|(method, _)| method.is_some()) {
                let endpoint = entries.into_iter().fold(RouteMethod::new(), |route, (method, endpoint)| match method {
                    Some(PoemMethod::Get) => route.get(endpoint.0),
                    Some(PoemMethod::Post) => route.post(endpoint.0),
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

    fn http_get(&self) -> PoemSelector {
        PoemSelector { parts: vec![PoemSelectorPart::Method(PoemMethod::Get)] }
    }

    fn http_post(&self) -> PoemSelector {
        PoemSelector { parts: vec![PoemSelectorPart::Method(PoemMethod::Post)] }
    }

    fn http_path(&self, path: &str) -> PoemSelector {
        PoemSelector { parts: vec![PoemSelectorPart::Path(path.into())] }
    }

    fn http_prefix(&self, prefix: &str) -> PoemSelector {
        PoemSelector { parts: vec![PoemSelectorPart::Prefix(prefix.into())] }
    }
}

#[cfg(test)]
mod tests {
    use super::{PoemEndpoint, PoemRouteImpl};
    use alux_http::RouteAlgExt;
    use poem::endpoint::make_sync;
    use poem::{Endpoint, Request, get};

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
}
