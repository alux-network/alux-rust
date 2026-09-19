//! Holds this interpretation to the shared opening and closing contract.

use alux_http::HttpProgramExt;
use alux_http_actix::{ActixHandlerImpl, ActixRoute, ActixServer};
use alux_http_conformance::{ExpectLifecycleExt, LifecycleApiExt, Shop};

/// Compiles the shared lifecycle surface as the route this interpretation serves.
fn route() -> ActixRoute {
    let api = ActixHandlerImpl::new(Shop);

    api.compile_http(api.lifecycle_api::<Shop>())
}

#[tokio::test]
async fn reopens_the_address_it_closed() {
    ActixServer.expect_reopening(route).await.unwrap();
}

#[tokio::test]
async fn reopens_the_address_it_closed_while_a_connection_is_held() {
    ActixServer.expect_reopening_while_a_connection_is_held(route).await.unwrap();
}

#[tokio::test]
async fn answers_a_request_in_flight_when_it_closes() {
    ActixServer.expect_answering_what_is_in_flight(route).await.unwrap();
}

#[tokio::test]
async fn ends_a_request_that_outlives_the_drain_when_it_closes() {
    ActixServer.expect_ending_what_outlives_the_drain(route).await.unwrap();
}
