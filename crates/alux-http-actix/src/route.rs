//! Composes selectors, and registers what they reach with actix-web.

use actix_web::http::Method;
use actix_web::web::ServiceConfig;
use actix_web::{HttpResponse, Route};
use alux_http::{
    HttpMethod, HttpSelectorAlg, PathSyntaxAlg, RouteAlg, RoutePath, SelectorAlg, compose_path, describe_path,
};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Spells route parameters the way actix-web's router reads them.
struct ActixPath;

impl PathSyntaxAlg for ActixPath {
    fn param(&self, name: &str) -> String {
        format!("{{{name}}}")
    }

    fn tail(&self, name: &str) -> String {
        format!("{{{name}:.*}}")
    }
}

/// Interprets a request method as the one actix-web routes on.
fn actix_method(method: HttpMethod) -> Method {
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
enum ActixSelectorPart {
    Method(HttpMethod),
    Path(RoutePath),
    Prefix(RoutePath),
}

/// Carries route-selection meaning before it is registered with actix-web.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ActixSelector {
    parts: Vec<ActixSelectorPart>,
}

impl ActixSelector {
    /// Returns the composed absolute path this selector matches.
    pub fn path(&self) -> String {
        describe_path(self.paths())
    }

    /// Returns the composed path in the spelling actix-web's router reads.
    pub(crate) fn actix_path(&self) -> String {
        compose_path(self.paths(), &ActixPath)
    }

    fn paths(&self) -> impl Iterator<Item = &RoutePath> {
        self.parts.iter().filter_map(|part| match part {
            ActixSelectorPart::Path(path) | ActixSelectorPart::Prefix(path) => Some(path),
            ActixSelectorPart::Method(_) => None,
        })
    }

    /// Returns the selected method and path, using `*` when no method is selected.
    pub fn label(&self) -> String {
        let method = self.method().map_or("*", HttpMethod::label);
        format!("{method} {}", self.path())
    }

    pub(crate) fn method(&self) -> Option<HttpMethod> {
        self.parts.iter().rev().find_map(|part| match part {
            ActixSelectorPart::Method(method) => Some(*method),
            ActixSelectorPart::Path(_) | ActixSelectorPart::Prefix(_) => None,
        })
    }
}

/// States an endpoint as what makes one, because actix-web builds its routes once per worker.
///
/// An `actix_web::Route` is neither cloneable nor reusable, and a service is configured again for
/// every worker thread, so what an interpretation can hold is the making of one rather than one.
#[derive(Clone)]
pub struct ActixEndpoint(Arc<dyn Fn() -> Route + Send + Sync>);

impl ActixEndpoint {
    /// States an endpoint as the route it makes, each time one is asked for.
    pub fn new<Make>(make: Make) -> Self
    where
        Make: Fn() -> Route + Send + Sync + 'static,
    {
        Self(Arc::new(make))
    }
}

#[derive(Clone)]
struct ActixRouteEntry {
    selector: ActixSelector,
    endpoint: ActixEndpoint,
}

/// Carries a composable collection of actix-web routes, as what makes them.
#[derive(Clone, Default)]
pub struct ActixRoute {
    entries: Vec<ActixRouteEntry>,
}

impl ActixRoute {
    /// Returns each composed selector as `METHOD /path`, in declaration order.
    pub fn labels(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.label()).collect()
    }

    /// Returns each composed route path, in declaration order.
    pub fn paths(&self) -> Vec<String> {
        self.entries.iter().map(|entry| entry.selector.path()).collect()
    }

    /// Configures a service with every route this composition states.
    ///
    /// The result configures a service as many times as a server asks it to, which is what an
    /// `actix_web::HttpServer` does once per worker.
    pub fn into_actix(self) -> impl Fn(&mut ServiceConfig) + Clone + Send + 'static {
        let mut paths = BTreeMap::<String, Vec<(Option<HttpMethod>, ActixEndpoint)>>::new();
        for entry in self.entries {
            paths.entry(entry.selector.actix_path()).or_default().push((entry.selector.method(), entry.endpoint));
        }

        move |service| {
            for (path, entries) in &paths {
                for (method, endpoint) in entries {
                    let route = (endpoint.0)();
                    let route = match method {
                        Some(method) => route.method(actix_method(*method)),
                        None => route,
                    };
                    service.route(path, route);
                }
            }
        }
    }
}

/// Answers whatever nothing else did, which is how a surface states that it did not.
pub fn not_found() -> HttpResponse {
    HttpResponse::NotFound().finish()
}

/// Interprets categorical route composition as native actix-web routing.
#[derive(Debug, Default)]
pub struct ActixRouteImpl;

impl SelectorAlg for ActixRouteImpl {
    type Selector = ActixSelector;

    fn identity(&self) -> ActixSelector {
        ActixSelector::default()
    }

    fn compose(&self, mut first: ActixSelector, second: ActixSelector) -> ActixSelector {
        first.parts.extend(second.parts);
        first
    }
}

impl RouteAlg for ActixRouteImpl {
    type Route = ActixRoute;
    type Selector = ActixSelector;
    type Endpoint = ActixEndpoint;

    fn initial(&self) -> ActixRoute {
        ActixRoute::default()
    }

    fn coproduct(&self, mut left: ActixRoute, right: ActixRoute) -> ActixRoute {
        left.entries.extend(right.entries);
        left
    }

    fn precompose(&self, selector: ActixSelector, mut route: ActixRoute) -> ActixRoute {
        for entry in &mut route.entries {
            entry.selector = self.compose(selector.clone(), core::mem::take(&mut entry.selector));
        }
        route
    }

    fn lift(&self, endpoint: ActixEndpoint) -> ActixRoute {
        ActixRoute { entries: vec![ActixRouteEntry { selector: self.identity(), endpoint }] }
    }
}

impl HttpSelectorAlg for ActixRouteImpl {
    type Selector = ActixSelector;

    fn http_method(&self, method: HttpMethod) -> ActixSelector {
        ActixSelector { parts: vec![ActixSelectorPart::Method(method)] }
    }

    fn http_path(&self, path: &RoutePath) -> ActixSelector {
        ActixSelector { parts: vec![ActixSelectorPart::Path(path.clone())] }
    }

    fn http_prefix(&self, prefix: &RoutePath) -> ActixSelector {
        ActixSelector { parts: vec![ActixSelectorPart::Prefix(prefix.clone())] }
    }
}
