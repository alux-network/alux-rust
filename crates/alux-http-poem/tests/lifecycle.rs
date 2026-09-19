//! Holds this interpretation to the shared opening and closing contract.

use alux_http::HttpProgramExt;
use alux_http_conformance::{ExpectLifecycleExt, LifecycleApiExt, Shop};
use alux_http_poem::{PoemHandlerImpl, PoemRoute, PoemServer};

/// Compiles the shared lifecycle surface as the route this interpretation serves.
fn route() -> PoemRoute {
    let api = PoemHandlerImpl::new(Shop);

    api.compile_http(api.lifecycle_api::<Shop>())
}

#[tokio::test]
async fn reopens_the_address_it_closed() {
    PoemServer.expect_reopening(route).await.unwrap();
}

#[tokio::test]
async fn reopens_the_address_it_closed_while_a_connection_is_held() {
    PoemServer.expect_reopening_while_a_connection_is_held(route).await.unwrap();
}

#[tokio::test]
async fn answers_a_request_in_flight_when_it_closes() {
    PoemServer.expect_answering_what_is_in_flight(route).await.unwrap();
}

#[tokio::test]
async fn ends_a_request_that_outlives_the_drain_when_it_closes() {
    PoemServer.expect_ending_what_outlives_the_drain(route).await.unwrap();
}
