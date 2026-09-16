//! Composes selectors, and builds the warp filter they state.

use crate::WarpRequest;
use alux_http::{HttpMethod, HttpSelectorAlg, PathSegment, RouteAlg, RoutePath, SelectorAlg, describe_path};
use bytes::Bytes;
use core::future::Future;
use core::pin::Pin;
use std::sync::Arc;
use warp::Filter;
use warp::filters::BoxedFilter;
use warp::http::Method;
use warp::reply::Response;

/// The answer one reached endpoint produces.
pub type Answer = Pin<Box<dyn Future<Output = Response> + Send>>;

/// Interprets a request method as the one warp matches on.
fn warp_method(method: HttpMethod) -> Method {
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
enum WarpSelectorPart {
    Method(HttpMethod),
    Path(RoutePath),
    Prefix(RoutePath),
}

/// Carries route-selection meaning before it becomes the filters warp matches with.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WarpSelector {
    parts: Vec<WarpSelectorPart>,
}

impl WarpSelector {
    /// Returns the composed absolute path this selector matches.
    ///
    /// warp matches by composing filters rather than by parsing a path, so a spelling is needed only
    /// to describe the surface.
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
            WarpSelectorPart::Path(path) | WarpSelectorPart::Prefix(path) => Some(path),
            WarpSelectorPart::Method(_) => None,
        })
    }

    fn segments(&self) -> Vec<PathSegment> {
        self.paths().flat_map(|path| path.segments().iter().cloned()).collect()
    }

    fn method(&self) -> Option<HttpMethod> {
        self.parts.iter().rev().find_map(|part| match part {
            WarpSelectorPart::Method(method) => Some(*method),
            WarpSelectorPart::Path(_) | WarpSelectorPart::Prefix(_) => None,
        })
    }

    /// Builds the filter matching this selector, gathering what its path binds.
    fn filter(&self) -> BoxedFilter<(Vec<String>,)> {
        let segments = self.segments();
        let mut bound = warp::any().map(Vec::<String>::new).boxed();
        let mut open = false;
        for segment in &segments {
            bound = match segment {
                PathSegment::Literal(value) => bound.and(warp::path(value.clone())).boxed(),
                PathSegment::Param(_) => bound
                    .and(warp::path::param::<String>())
                    .map(|mut bound: Vec<String>, value: String| {
                        bound.push(value);
                        bound
                    })
                    .boxed(),
                PathSegment::Tail(_) => {
                    open = true;
                    bound
                        .and(warp::path::tail())
                        .map(|mut bound: Vec<String>, tail: warp::path::Tail| {
                            bound.push(tail.as_str().to_owned());
                            bound
                        })
                        .boxed()
                }
            };
        }

        // A tail already matches everything that is left, so only a closed path states an end.
        let bound = if open { bound } else { bound.and(warp::path::end()).boxed() };

        match self.method() {
            Some(method) => {
                let expected = warp_method(method);
                bound
                    .and(warp::method())
                    .and_then(move |bound: Vec<String>, method: Method| {
                        let expected = expected.clone();
                        async move { if method == expected { Ok(bound) } else { Err(warp::reject()) } }
                    })
                    .boxed()
            }
            None => bound,
        }
    }
}

/// Erases what one endpoint does with a request warp's filters matched.
#[derive(Clone)]
pub struct WarpEndpoint(Arc<dyn Fn(WarpRequest) -> Answer + Send + Sync>);

impl WarpEndpoint {
    /// States an endpoint as what it answers, given what the filters gathered.
    pub fn new<Reach>(reach: Reach) -> Self
    where
        Reach: Fn(WarpRequest) -> Answer + Send + Sync + 'static,
    {
        Self(Arc::new(reach))
    }
}

#[derive(Clone)]
struct WarpRouteEntry {
    selector: WarpSelector,
    endpoint: WarpEndpoint,
}

/// Carries a composable collection of warp endpoints.
#[derive(Clone, Default)]
pub struct WarpRoute {
    entries: Vec<WarpRouteEntry>,
}

impl WarpRoute {
    /// Returns each composed selector as `METHOD /path`, in declaration order.
    pub fn labels(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.label()).collect()
    }

    /// Returns each composed route path, in declaration order.
    pub fn paths(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.path()).collect()
    }

    /// Materializes the composed meaning as a native warp filter.
    ///
    /// A coproduct of routes is `or`, which is the whole of what composing a surface means here.
    pub fn into_warp(self) -> BoxedFilter<(Response,)> {
        let nothing = warp::any().and_then(|| async { Err::<Response, _>(warp::reject::not_found()) }).boxed();

        self.entries.into_iter().fold(nothing, |answered, entry| {
            let endpoint = entry.endpoint.clone();
            let reached = entry
                .selector
                .filter()
                .and(warp::query::raw().or(warp::any().map(String::new)).unify())
                .and(warp::header::headers_cloned())
                .and(warp::body::bytes())
                .and_then(move |captures, query, headers, body: Bytes| {
                    let endpoint = endpoint.clone();
                    async move {
                        let request = WarpRequest { captures, query, headers, body: body.to_vec() };

                        Ok::<_, warp::Rejection>((endpoint.0)(request).await)
                    }
                })
                .boxed();

            answered.or(reached).unify().boxed()
        })
    }
}

/// Interprets categorical route composition as native warp filter composition.
#[derive(Debug, Default)]
pub struct WarpRouteImpl;

impl SelectorAlg for WarpRouteImpl {
    type Selector = WarpSelector;

    fn identity(&self) -> WarpSelector {
        WarpSelector::default()
    }

    fn compose(&self, mut first: WarpSelector, second: WarpSelector) -> WarpSelector {
        first.parts.extend(second.parts);
        first
    }
}

impl RouteAlg for WarpRouteImpl {
    type Route = WarpRoute;
    type Selector = WarpSelector;
    type Endpoint = WarpEndpoint;

    fn initial(&self) -> WarpRoute {
        WarpRoute::default()
    }

    fn coproduct(&self, mut left: WarpRoute, right: WarpRoute) -> WarpRoute {
        left.entries.extend(right.entries);
        left
    }

    fn precompose(&self, selector: WarpSelector, mut route: WarpRoute) -> WarpRoute {
        for entry in &mut route.entries {
            entry.selector = self.compose(selector.clone(), core::mem::take(&mut entry.selector));
        }
        route
    }

    fn lift(&self, endpoint: WarpEndpoint) -> WarpRoute {
        WarpRoute { entries: vec![WarpRouteEntry { selector: self.identity(), endpoint }] }
    }
}

impl HttpSelectorAlg for WarpRouteImpl {
    type Selector = WarpSelector;

    fn http_method(&self, method: HttpMethod) -> WarpSelector {
        WarpSelector { parts: vec![WarpSelectorPart::Method(method)] }
    }

    fn http_path(&self, path: &RoutePath) -> WarpSelector {
        WarpSelector { parts: vec![WarpSelectorPart::Path(path.clone())] }
    }

    fn http_prefix(&self, prefix: &RoutePath) -> WarpSelector {
        WarpSelector { parts: vec![WarpSelectorPart::Prefix(prefix.clone())] }
    }
}
