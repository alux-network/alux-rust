//! Holds this interpretation to the shared surface and the shared scenario.
//!
//! The adapter below is the only place warp is named: everything the scenario states is stated
//! without a framework, so what is compared is the surface rather than the reply type.

use alux_http::{HttpProgramExt, HttpStatus};
use alux_http_conformance::{
    AnswerAlg, LABELS, MultipartApiExt, Shop, ShopApiExt, WideApiExt, expect, expect_multipart,
};
use alux_http_direct::{DirectRequest, DirectResponse};
use alux_http_warp::WarpHandlerImpl;
use warp::filters::BoxedFilter;
use warp::reply::Response;

/// Answers a stated request with a warp filter.
struct Answering(BoxedFilter<(Response,)>);

impl AnswerAlg for Answering {
    async fn answer(&self, request: DirectRequest) -> DirectResponse {
        let method = request.method().map_or("GET", |method| method.label());
        let mut sent = warp::test::request().method(method).path(request.path());
        for (name, value) in request.headers() {
            sent = sent.header(name, value);
        }
        let answered = sent.body(request.body()).reply(&self.0).await;

        let status = HttpStatus::new(answered.status().as_u16());
        let headers = answered
            .headers()
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or_default().to_owned()))
            .collect::<Vec<_>>();

        headers
            .into_iter()
            .fold(DirectResponse::new(status), |answer, (name, value)| answer.with_header(&name, &value))
            .with_body(answered.body().to_vec())
    }
}

#[tokio::test]
async fn answers_the_shared_surface_as_the_scenario_states() {
    let api = WarpHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.shop_api::<Shop>());

    assert_eq!(compiled.labels(), LABELS);
    expect(&Answering(compiled.into_warp())).await.unwrap();
}

#[test]
fn compiles_the_widest_endpoint_the_specification_states() {
    // The assertion is the compile: sixteen arguments reach this interpretation's products.
    let api = WarpHandlerImpl::new(Shop);

    assert_eq!(api.compile_http(api.wide_api::<Shop>()).labels(), ["POST /wide"]);
}

#[tokio::test]
async fn reads_a_body_arriving_as_parts() {
    let api = WarpHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.multipart_api::<Shop>());

    assert_eq!(compiled.labels(), ["POST /upload"]);
    expect_multipart(&Answering(compiled.into_warp())).await.unwrap();
}
