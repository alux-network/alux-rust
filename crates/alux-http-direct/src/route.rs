//! Routes a request against the paths a program states, with no router but the program's own.

use crate::{DirectError, DirectRequest, DirectResponse};
use alux_http::{HttpMethod, HttpSelectorAlg, PathSegment, RouteAlg, RoutePath, SelectorAlg, describe_path};
use core::future::Future;
use core::pin::Pin;
use std::sync::Arc;

/// The answer one reached endpoint produces.
pub type Answer = Pin<Box<dyn Future<Output = DirectResponse> + Send>>;

type Reached = Arc<dyn Fn(DirectRequest, Vec<String>) -> Answer + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum DirectSelectorPart {
    Method(HttpMethod),
    Path(RoutePath),
    Prefix(RoutePath),
}

/// Carries route-selection meaning, which for this interpretation is all the routing there is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DirectSelector {
    parts: Vec<DirectSelectorPart>,
}

impl DirectSelector {
    /// Returns the composed absolute path this selector matches.
    ///
    /// There is no router to spell a path for, so the description is the only rendering needed.
    pub fn path(&self) -> String {
        describe_path(self.paths())
    }

    /// Returns the selected method and path, using `*` when no method is selected.
    pub fn label(&self) -> String {
        let method = self.method().map_or("*", HttpMethod::label);
        format!("{method} {}", self.path())
    }

    fn paths(&self) -> impl Iterator<Item = &RoutePath> {
        self.parts.iter().filter_map(|part| match part {
            DirectSelectorPart::Path(path) | DirectSelectorPart::Prefix(path) => Some(path),
            DirectSelectorPart::Method(_) => None,
        })
    }

    fn segments(&self) -> Vec<&PathSegment> {
        self.paths().flat_map(RoutePath::segments).collect()
    }

    fn method(&self) -> Option<HttpMethod> {
        self.parts.iter().rev().find_map(|part| match part {
            DirectSelectorPart::Method(method) => Some(*method),
            DirectSelectorPart::Path(_) | DirectSelectorPart::Prefix(_) => None,
        })
    }

    /// Returns what the path bound, or nothing where this selector does not match it.
    ///
    /// A literal matches itself, a parameter binds one segment, and a tail binds every segment that
    /// is left. Anything the request states beyond what the selector reads is not a match.
    fn captures(&self, path: &str) -> Option<Vec<String>> {
        let mut asked = path.split('/').filter(|segment| !segment.is_empty()).peekable();
        let mut captured = Vec::new();
        for segment in self.segments() {
            match segment {
                PathSegment::Literal(value) => {
                    if asked.next()? != value {
                        return None;
                    }
                }
                PathSegment::Param(_) => captured.push(asked.next()?.to_owned()),
                PathSegment::Tail(_) => {
                    captured.push(asked.by_ref().collect::<Vec<_>>().join("/"));

                    return Some(captured);
                }
            }
        }

        asked.next().is_none().then_some(captured)
    }
}

/// Erases what one endpoint does with a request it was reached by.
#[derive(Clone)]
pub struct DirectEndpoint(Reached);

impl DirectEndpoint {
    /// States an endpoint as what it answers, given a request and what its path bound.
    pub fn new<Reach>(reach: Reach) -> Self
    where
        Reach: Fn(DirectRequest, Vec<String>) -> Answer + Send + Sync + 'static,
    {
        Self(Arc::new(reach))
    }
}

#[derive(Clone)]
struct DirectRouteEntry {
    selector: DirectSelector,
    endpoint: DirectEndpoint,
}

/// Carries a composable collection of endpoints and the selectors that reach them.
#[derive(Clone, Default)]
pub struct DirectRoute {
    entries: Vec<DirectRouteEntry>,
}

impl DirectRoute {
    /// Returns each composed selector as `METHOD /path`, in declaration order.
    pub fn labels(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.label()).collect()
    }

    /// Returns each composed route path, in declaration order.
    pub fn paths(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.path()).collect()
    }

    /// Answers a request by reaching the endpoint its method and path select.
    ///
    /// Declaration order is the order candidates are read in, so a surface answers with the first
    /// endpoint that states it. A path that is declared under other methods is answered as such,
    /// rather than as nothing being there.
    pub async fn answer(&self, request: DirectRequest) -> DirectResponse {
        let mut allowed = false;
        for entry in &self.entries {
            let Some(captures) = entry.selector.captures(request.path()) else { continue };
            allowed = true;
            if entry.selector.method().is_none_or(|method| Some(method) == request.method()) {
                return (entry.endpoint.0)(request, captures).await;
            }
        }

        let error = if allowed {
            DirectError::method_not_allowed(request.path())
        } else {
            DirectError::not_found(request.path())
        };

        error.into()
    }
}

/// Interprets categorical route composition as the only routing this interpretation needs.
#[derive(Debug, Default)]
pub struct DirectRouteImpl;

impl SelectorAlg for DirectRouteImpl {
    type Selector = DirectSelector;

    fn identity(&self) -> DirectSelector {
        DirectSelector::default()
    }

    fn compose(&self, mut first: DirectSelector, second: DirectSelector) -> DirectSelector {
        first.parts.extend(second.parts);
        first
    }
}

impl RouteAlg for DirectRouteImpl {
    type Route = DirectRoute;
    type Selector = DirectSelector;
    type Endpoint = DirectEndpoint;

    fn initial(&self) -> DirectRoute {
        DirectRoute::default()
    }

    fn coproduct(&self, mut left: DirectRoute, right: DirectRoute) -> DirectRoute {
        left.entries.extend(right.entries);
        left
    }

    fn precompose(&self, selector: DirectSelector, mut route: DirectRoute) -> DirectRoute {
        for entry in &mut route.entries {
            entry.selector = self.compose(selector.clone(), core::mem::take(&mut entry.selector));
        }
        route
    }

    fn lift(&self, endpoint: DirectEndpoint) -> DirectRoute {
        DirectRoute { entries: vec![DirectRouteEntry { selector: self.identity(), endpoint }] }
    }
}

impl HttpSelectorAlg for DirectRouteImpl {
    type Selector = DirectSelector;

    fn http_method(&self, method: HttpMethod) -> DirectSelector {
        DirectSelector { parts: vec![DirectSelectorPart::Method(method)] }
    }

    fn http_path(&self, path: &RoutePath) -> DirectSelector {
        DirectSelector { parts: vec![DirectSelectorPart::Path(path.clone())] }
    }

    fn http_prefix(&self, prefix: &RoutePath) -> DirectSelector {
        DirectSelector { parts: vec![DirectSelectorPart::Prefix(prefix.clone())] }
    }
}
