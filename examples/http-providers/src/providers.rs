//! The cases a benchmark states of every provider this example serves with.
//!
//! A case is a subject and what measures it, so what differs between providers is a value rather
//! than a type. What each bench measures is stated by the bench itself, in `benches/`.
//!
//! A round of any of these opens a server, puts a connection on it, and closes it, so what it
//! costs is a bind, an accept and a shutdown. Whether that is CPU-bound or IO-bound is the case's,
//! not the provider's: a composable `close` returns as soon as the listener is dropped, while a
//! closed one blocks until the framework's drain expires.

use crate::measured;
use alux_bench::{BenchRounds, BenchSentRoutine};
use alux_http::{HttpProgramExt, HttpServerAlg};
use alux_http_actix::{ActixHandlerImpl, ActixRoute, ActixServer};
use alux_http_axum::{AxumHandlerImpl, AxumRoute, AxumServer};
use alux_http_conformance::{Closing, Handover, LifecycleApiExt, LifecycleError, MeasureLifecycleExt, Shop};
use alux_http_direct::DirectHandlerImpl;
use alux_http_hyper::{HyperRoute, HyperServer};
use alux_http_poem::{PoemHandlerImpl, PoemRoute, PoemServer};
use alux_http_rocket::{RocketHandlerImpl, RocketRoute, RocketServer};
use alux_http_salvo::{SalvoHandlerImpl, SalvoRoute, SalvoServer};
use alux_http_warp::{WarpHandlerImpl, WarpRoute, WarpServer};

/// What one case of one provider measures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Measures {
    /// Closing a server with one request in flight, and ending it where the case states that.
    Closing(Closing),
    /// Handing one address from a server to the next, under a load.
    Handover(Handover),
}

impl From<Closing> for Measures {
    fn from(closing: Closing) -> Self {
        Self::Closing(closing)
    }
}

impl From<Handover> for Measures {
    fn from(load: Handover) -> Self {
        Self::Handover(load)
    }
}

/// One case of a group: the subject it measures, and what measures it.
pub type Case = (&'static str, BenchSentRoutine);

/// The providers whose accept loop is composable, in the order the README lists them.
pub fn composable(measures: Measures) -> Vec<Case> {
    vec![
        ("hyper", rounds_of(|| HyperServer, hyper_route, measures)),
        ("axum", rounds_of(|| AxumServer, axum_route, measures)),
        ("warp", rounds_of(|| WarpServer, warp_route, measures)),
    ]
}

/// The providers whose accept loop is the framework's own, in the order the README lists them.
pub fn closed(measures: Measures) -> Vec<Case> {
    vec![
        ("poem", rounds_of(|| PoemServer, poem_route, measures)),
        ("salvo", rounds_of(|| SalvoServer, salvo_route, measures)),
        ("actix", rounds_of(|| ActixServer, actix_route, measures)),
        ("rocket", rounds_of(|| RocketServer, rocket_route, measures)),
    ]
}

/// Every provider, in the order the README lists them.
pub fn every_provider(measures: Measures) -> Vec<Case> {
    let mut cases = composable(measures);
    cases.extend(closed(measures));

    cases
}

/// States the routine one provider runs for what a case measures.
///
/// What it carries is how to make a server, not one server: every round opens its own.
fn rounds_of<Make, Server, Compile>(make: Make, route: Compile, measures: Measures) -> BenchSentRoutine
where
    Make: Fn() -> Server + Send + 'static,
    Server: HttpServerAlg,
    Server::Error: Into<LifecycleError>,
    Compile: FnMut() -> Server::Program + Copy + Send + 'static,
{
    Box::new(move |rounds: BenchRounds| match measures {
        Measures::Closing(closing) => measured(make().time_closing(route, closing, rounds.count())),
        Measures::Handover(load) => measured(make().time_handover(route, load, rounds.count())),
    })
}

fn hyper_route() -> HyperRoute {
    let api = DirectHandlerImpl::new(Shop);

    HyperRoute::new(api.compile_http(api.lifecycle_api::<Shop>()))
}

fn axum_route() -> AxumRoute {
    let api = AxumHandlerImpl::new(Shop);

    api.compile_http(api.lifecycle_api::<Shop>())
}

fn warp_route() -> WarpRoute {
    let api = WarpHandlerImpl::new(Shop);

    api.compile_http(api.lifecycle_api::<Shop>())
}

fn poem_route() -> PoemRoute {
    let api = PoemHandlerImpl::new(Shop);

    api.compile_http(api.lifecycle_api::<Shop>())
}

fn salvo_route() -> SalvoRoute {
    let api = SalvoHandlerImpl::new(Shop);

    api.compile_http(api.lifecycle_api::<Shop>())
}

fn actix_route() -> ActixRoute {
    let api = ActixHandlerImpl::new(Shop);

    api.compile_http(api.lifecycle_api::<Shop>())
}

fn rocket_route() -> RocketRoute {
    let api = RocketHandlerImpl::new(Shop);

    api.compile_http(api.lifecycle_api::<Shop>())
}
