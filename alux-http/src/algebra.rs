use alux_ext::{ApplyAlg, ext};
use trait_set::trait_set;

/// Describes the capability to build typed handler endpoints.
pub trait HandlerAlg {
    /// The interpreter's homogeneous endpoint representation.
    type Endpoint;
}

/// Compiles a typed handler declaration supported by an interpreter.
pub trait HandlerEndpointAlg<Context, Inputs, Args, Transform, Output> {
    /// Interprets a typed first-order handler as a concrete endpoint.
    ///
    /// `Inputs`, `Args`, `Transform`, and `Output` remain type-level evidence;
    /// only the handler value needs to be supplied at runtime.
    fn finish_handler<Handler>(&self, handler: Handler) -> Self::Endpoint
    where
        Self: HandlerAlg,
        Handler: ApplyAlg<Context, Args, Output = Output> + Send + Sync + 'static;
}

/// Interprets a named, defunctionalized HTTP program with `Compiler`.
pub trait HttpProgramAlg<Compiler> {
    /// The route representation produced by `Compiler`.
    type Route;

    /// Compiles the program through the supplied interpreter.
    fn compile_http(self, compiler: &Compiler) -> Self::Route;
}

/// Describes categorical composition of route selectors.
pub trait SelectorAlg {
    /// The interpreter's selector representation.
    type Selector;

    /// Returns the selector identity.
    fn identity(&self) -> Self::Selector;

    /// Composes two selectors from left to right.
    fn compose(&self, first: Self::Selector, second: Self::Selector) -> Self::Selector;
}

/// Describes categorical construction and composition of routes.
pub trait RouteAlg {
    /// The interpreter's composed route representation.
    type Route;
    /// The selector applied to a route or endpoint.
    type Selector;
    /// The homogeneous endpoint representation lifted into routes.
    type Endpoint;

    /// Returns the initial route.
    fn initial(&self) -> Self::Route;

    /// Forms the coproduct of two routes.
    fn coproduct(&self, left: Self::Route, right: Self::Route) -> Self::Route;

    /// Precomposes a route with a selector.
    fn precompose(&self, selector: Self::Selector, route: Self::Route) -> Self::Route;

    /// Lifts an endpoint into a route.
    fn lift(&self, endpoint: Self::Endpoint) -> Self::Route;
}

/// Describes the HTTP input roles chosen by an interpreter.
pub trait HttpInputAlg {
    /// The interpreter's path extractor for `Input`.
    type Path<Input>;
    /// The interpreter's query extractor for `Input`.
    type Query<Input>;
    /// The interpreter's request-body extractor for `Input`.
    type Body<Input>;
    /// The interpreter's header extractor for `Input`.
    type Header<Input>;
    /// The interpreter's authentication extractor for `Input`.
    type Auth<Input>;
    /// The interpreter's endpoint-context extractor for `Input`.
    type Context<Input>;
}

/// Describes HTTP selectors independently of route composition.
pub trait HttpSelectorAlg {
    /// The interpreter's HTTP selector representation.
    type Selector;

    /// Interprets the GET method selector.
    fn http_get(&self) -> Self::Selector;

    /// Interprets the POST method selector.
    fn http_post(&self) -> Self::Selector;

    /// Interprets an exact path selector.
    fn http_path(&self, path: &str) -> Self::Selector;

    /// Interprets a path-prefix selector.
    fn http_prefix(&self, prefix: &str) -> Self::Selector;
}

trait_set! {
    /// Combines the capabilities required to interpret HTTP route composition.
    ///
    /// The alias states that HTTP selectors, route composition, and selector composition are
    /// witnessed by one interpreter using a single selector representation.
    pub trait HttpRouteAlg = SelectorAlg
        + HttpSelectorAlg<Selector = <Self as SelectorAlg>::Selector>
        + RouteAlg<Selector = <Self as SelectorAlg>::Selector>;

    /// Combines the capabilities required to interpret a typed HTTP API.
    ///
    /// The alias states that endpoints built by the handler capability are the endpoints lifted
    /// into composed routes.
    pub trait HttpApiAlg =
        HandlerAlg + HttpInputAlg + HttpRouteAlg + RouteAlg<Endpoint = <Self as HandlerAlg>::Endpoint>;
}

/// Carries a fluent route composition over an interpreter.
pub struct Routes<'a, Alg>
where
    Alg: RouteAlg,
{
    alg: &'a Alg,
    route: Alg::Route,
}

impl<Alg> Routes<'_, Alg>
where
    Alg: RouteAlg,
{
    /// Returns the interpreted route value.
    pub fn into_route(self) -> Alg::Route {
        self.route
    }

    /// Forms a coproduct with another route composition.
    #[must_use]
    pub fn coproduct(self, other: Self) -> Self {
        let route = self.alg.coproduct(self.route, other.route);
        Self { alg: self.alg, route }
    }

    /// Merges another route composition into this one.
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        self.coproduct(other)
    }
}

impl<Alg> Routes<'_, Alg>
where
    Alg: HttpRouteAlg,
{
    fn append(self, route: Alg::Route) -> Self {
        let route = self.alg.coproduct(self.route, route);
        Self { alg: self.alg, route }
    }

    /// Adds an endpoint at an exact path.
    #[must_use]
    pub fn at(self, path: &str, endpoint: Alg::Endpoint) -> Self {
        let route = self.alg.precompose(self.alg.http_path(path), self.alg.lift(endpoint));
        self.append(route)
    }

    /// Nests another route composition under a path prefix.
    #[must_use]
    pub fn nest(self, prefix: &str, nested: Self) -> Self {
        let route = self.alg.precompose(self.alg.http_prefix(prefix), nested.route);
        self.append(route)
    }
}

/// Provides fluent route composition on any `RouteAlg`.
#[ext(name = RouteAlgExt, supertraits = RouteAlg + Sized)]
pub impl<This> This
where
    This: RouteAlg,
{
    /// Starts with the initial route.
    fn routes(&self) -> Routes<'_, Self> {
        Routes { alg: self, route: self.initial() }
    }

    /// Wraps an already interpreted route for further fluent composition.
    fn route(&self, route: This::Route) -> Routes<'_, Self> {
        Routes { alg: self, route }
    }
}

/// Compiles defunctionalized HTTP programs with an interpreter.
#[ext(name = HttpProgramExt)]
pub impl<This> This {
    /// Compiles a named HTTP program with this interpreter.
    fn compile_http<Program>(&self, program: Program) -> Program::Route
    where
        Program: HttpProgramAlg<This>,
    {
        program.compile_http(self)
    }
}
