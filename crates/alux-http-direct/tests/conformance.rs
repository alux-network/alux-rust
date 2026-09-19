//! Holds this interpretation to the shared surface and the shared scenario.

use alux_http::HttpProgramExt;
use alux_http_conformance::{
    AnswerAlg, LABELS, MultipartApiExt, Shop, ShopApiExt, StreamApiExt, WideApiExt, expect, expect_multipart,
    expect_streaming,
};
use alux_http_direct::{DirectHandlerImpl, DirectRequest, DirectResponse, DirectRoute};
use core::future::Future;

/// Answers the scenario's requests with this interpretation's own answering.
struct Answering(DirectRoute);

impl AnswerAlg for Answering {
    fn answer(&self, request: DirectRequest) -> impl Future<Output = DirectResponse> + Send {
        self.0.answer(request)
    }
}

#[tokio::test]
async fn answers_the_shared_surface_as_the_scenario_states() {
    let api = DirectHandlerImpl::new(Shop);
    let surface = api.compile_http(api.shop_api::<Shop>());

    assert_eq!(surface.labels(), LABELS);
    expect(&Answering(surface)).await.unwrap();
}

#[test]
fn compiles_the_widest_endpoint_the_specification_states() {
    // The assertion is the compile: sixteen arguments reach this interpretation's products.
    let api = DirectHandlerImpl::new(Shop);

    assert_eq!(api.compile_http(api.wide_api::<Shop>()).labels(), ["POST /wide"]);
}

#[tokio::test]
async fn answers_a_body_produced_over_time() {
    let api = DirectHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.stream_api::<Shop>());

    assert_eq!(compiled.labels(), ["GET /ticks"]);
    expect_streaming(&Answering(compiled)).await.unwrap();
}

#[tokio::test]
async fn reads_a_body_arriving_as_parts() {
    let api = DirectHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.multipart_api::<Shop>());

    assert_eq!(compiled.labels(), ["POST /upload"]);
    expect_multipart(&Answering(compiled)).await.unwrap();
}
