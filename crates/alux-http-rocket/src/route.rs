//! Composes selectors, and builds the Rocket routes they state.

use crate::{RocketAnswer, RocketBody, RocketRequest};
use alux_http::{
    HttpMethod, HttpSelectorAlg, PathSegment, PathSyntaxAlg, RouteAlg, RoutePath, SelectorAlg, compose_path,
    describe_path,
};
use bytes::Bytes;
use core::future::Future;
use core::pin::Pin;
use futures::TryStreamExt;
use rocket::data::Capped;
use rocket::data::{ByteUnit, Data};
use rocket::http::{HeaderMap, Method, Status};
use rocket::response::Response;
use rocket::route::{Handler, Outcome, Route};
use rocket::{Build, Request, Rocket, async_trait};
use std::io::Cursor;
use std::sync::Arc;
use tokio_util::io::StreamReader;

/// Spells route parameters the way Rocket's router reads them.
struct RocketPath;

impl PathSyntaxAlg for RocketPath {
    fn param(&self, name: &str) -> String {
        format!("<{name}>")
    }

    fn tail(&self, name: &str) -> String {
        format!("<{name}..>")
    }
}

/// Interprets a request method as the one Rocket routes on.
fn rocket_method(method: HttpMethod) -> Method {
    match method {
        HttpMethod::Get => Method::Get,
        HttpMethod::Post => Method::Post,
        HttpMethod::Put => Method::Put,
        HttpMethod::Patch => Method::Patch,
        HttpMethod::Delete => Method::Delete,
        HttpMethod::Head => Method::Head,
        HttpMethod::Options => Method::Options,
        HttpMethod::Trace => Method::Trace,
        HttpMethod::Connect => Method::Connect,
    }
}

/// The answer one reached endpoint produces.
pub type Answer = Pin<Box<dyn Future<Output = RocketAnswer> + Send>>;

#[derive(Debug, Clone, PartialEq, Eq)]
enum RocketSelectorPart {
    Method(HttpMethod),
    Path(RoutePath),
    Prefix(RoutePath),
}

/// Carries route-selection meaning before it is mounted on Rocket.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RocketSelector {
    parts: Vec<RocketSelectorPart>,
}

impl RocketSelector {
    /// Returns the composed absolute path this selector matches.
    pub fn path(&self) -> String {
        describe_path(self.paths())
    }

    /// Returns the composed path in the spelling Rocket's router reads.
    pub(crate) fn rocket_path(&self) -> String {
        compose_path(self.paths(), &RocketPath)
    }

    fn paths(&self) -> impl Iterator<Item = &RoutePath> {
        self.parts.iter().filter_map(|part| match part {
            RocketSelectorPart::Path(path) | RocketSelectorPart::Prefix(path) => Some(path),
            RocketSelectorPart::Method(_) => None,
        })
    }

    fn segments(&self) -> Vec<PathSegment> {
        self.paths().flat_map(|path| path.segments().iter().cloned()).collect()
    }

    /// Returns the selected method and path, using `*` when no method is selected.
    pub fn label(&self) -> String {
        let method = self.method().map_or("*", HttpMethod::label);
        format!("{method} {}", self.path())
    }

    pub(crate) fn method(&self) -> Option<HttpMethod> {
        self.parts.iter().rev().find_map(|part| match part {
            RocketSelectorPart::Method(method) => Some(*method),
            RocketSelectorPart::Path(_) | RocketSelectorPart::Prefix(_) => None,
        })
    }

    /// Returns what the path bound, read out of the segments Rocket routed.
    fn captures(&self, request: &Request<'_>) -> Vec<String> {
        let mut captured = Vec::new();
        for (index, segment) in self.segments().iter().enumerate() {
            match segment {
                PathSegment::Literal(_) => {}
                PathSegment::Param(_) => {
                    captured.push(request.routed_segment(index).unwrap_or_default().to_owned());
                }
                PathSegment::Tail(_) => {
                    let tail = request.routed_segments(index..).collect::<Vec<_>>().join("/");
                    captured.push(tail);
                }
            }
        }

        captured
    }
}

/// Erases what one endpoint does with a request Rocket routed to it.
#[derive(Clone)]
pub struct RocketEndpoint(Arc<dyn Fn(RocketRequest) -> Answer + Send + Sync>);

impl RocketEndpoint {
    /// States an endpoint as what it answers, given what Rocket routed.
    pub fn new<Reach>(reach: Reach) -> Self
    where
        Reach: Fn(RocketRequest) -> Answer + Send + Sync + 'static,
    {
        Self(Arc::new(reach))
    }
}

#[derive(Clone)]
struct RocketRouteEntry {
    selector: RocketSelector,
    endpoint: RocketEndpoint,
}

/// Carries a composable collection of Rocket endpoints.
#[derive(Clone, Default)]
pub struct RocketRoute {
    entries: Vec<RocketRouteEntry>,
}

impl RocketRoute {
    /// Returns each composed selector as `METHOD /path`, in declaration order.
    pub fn labels(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.label()).collect()
    }

    /// Returns each composed route path, in declaration order.
    pub fn paths(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.path()).collect()
    }

    /// Materializes the composed meaning as native Rocket routes.
    ///
    /// Rocket states a method on every route, so an endpoint composed without one states nothing it
    /// can mount.
    pub fn into_rocket(self) -> Vec<Route> {
        self.entries
            .into_iter()
            .filter_map(|entry| {
                let method = rocket_method(entry.selector.method()?);
                let path = entry.selector.rocket_path();
                let reaching = Reaching { selector: entry.selector, endpoint: entry.endpoint };

                Some(Route::new(method, &path, reaching))
            })
            .collect()
    }

    /// Mounts every route this composition states on a Rocket instance.
    pub fn mount(self, rocket: Rocket<Build>) -> Rocket<Build> {
        rocket.mount("/", self.into_rocket())
    }
}

/// What one endpoint does with a request Rocket routed to it.
#[derive(Clone)]
struct Reaching {
    selector: RocketSelector,
    endpoint: RocketEndpoint,
}

#[async_trait]
impl Handler for Reaching {
    async fn handle<'r>(&self, request: &'r Request<'_>, data: Data<'r>) -> Outcome<'r> {
        let captures = self.selector.captures(request);
        let query = request.uri().query().map(|query| query.as_str().to_owned()).unwrap_or_default();
        let mut headers = HeaderMap::new();
        for header in request.headers().iter() {
            headers.add_raw(header.name().to_string(), header.value().to_string());
        }
        let body = data.open(ByteUnit::Mebibyte(8)).into_bytes().await.map(Capped::into_inner).unwrap_or_default();
        let answered = (self.endpoint.0)(RocketRequest { captures, query, headers, body }).await;

        Outcome::Success(respond(answered))
    }
}

/// Builds the response Rocket answers with from what an endpoint stated.
fn respond<'r>(answered: RocketAnswer) -> Response<'r> {
    let mut response = Response::build();
    response.status(Status::new(answered.status.code()));
    for (name, value) in answered.headers {
        response.raw_header(name, value);
    }
    match answered.body {
        RocketBody::Stated(body) => response.sized_body(body.len(), Cursor::new(body)),
        // Rocket reads a body it streams, so what produces the chunks is read as one.
        RocketBody::Produced(chunks) => response.streamed_body(StreamReader::new(chunks.map_ok(Bytes::from))),
    };

    response.finalize()
}

/// Interprets categorical route composition as native Rocket routing.
#[derive(Debug, Default)]
pub struct RocketRouteImpl;

impl SelectorAlg for RocketRouteImpl {
    type Selector = RocketSelector;

    fn identity(&self) -> RocketSelector {
        RocketSelector::default()
    }

    fn compose(&self, mut first: RocketSelector, second: RocketSelector) -> RocketSelector {
        first.parts.extend(second.parts);
        first
    }
}

impl RouteAlg for RocketRouteImpl {
    type Route = RocketRoute;
    type Selector = RocketSelector;
    type Endpoint = RocketEndpoint;

    fn initial(&self) -> RocketRoute {
        RocketRoute::default()
    }

    fn coproduct(&self, mut left: RocketRoute, right: RocketRoute) -> RocketRoute {
        left.entries.extend(right.entries);
        left
    }

    fn precompose(&self, selector: RocketSelector, mut route: RocketRoute) -> RocketRoute {
        for entry in &mut route.entries {
            entry.selector = self.compose(selector.clone(), core::mem::take(&mut entry.selector));
        }
        route
    }

    fn lift(&self, endpoint: RocketEndpoint) -> RocketRoute {
        RocketRoute { entries: vec![RocketRouteEntry { selector: self.identity(), endpoint }] }
    }
}

impl HttpSelectorAlg for RocketRouteImpl {
    type Selector = RocketSelector;

    fn http_method(&self, method: HttpMethod) -> RocketSelector {
        RocketSelector { parts: vec![RocketSelectorPart::Method(method)] }
    }

    fn http_path(&self, path: &RoutePath) -> RocketSelector {
        RocketSelector { parts: vec![RocketSelectorPart::Path(path.clone())] }
    }

    fn http_prefix(&self, prefix: &RoutePath) -> RocketSelector {
        RocketSelector { parts: vec![RocketSelectorPart::Prefix(prefix.clone())] }
    }
}
