//! Holds this interpretation to the shared surface and the shared scenario.
//!
//! The adapter below is the only place axum is named: everything the scenario states is stated
//! without a framework, so what is compared is the surface rather than the response type.

use alux_http::{HttpProgramExt, HttpStatus};
use alux_http_axum::AxumHandlerImpl;
use alux_http_conformance::{
    AnswerAlg, LABELS, MultipartApiExt, Shop, ShopApiExt, StreamApiExt, WideApiExt, expect, expect_multipart,
    expect_streaming,
};
use alux_http_direct::{DirectRequest, DirectResponse};
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Method, Request};
use tower::ServiceExt;

/// Answers a stated request with an axum router.
struct Answering(Router);

impl AnswerAlg for Answering {
    async fn answer(&self, request: DirectRequest) -> DirectResponse {
        let method = request
            .method()
            .and_then(|method| Method::from_bytes(method.label().as_bytes()).ok())
            .unwrap_or(Method::GET);
        let mut sent = Request::builder().method(method).uri(request.path());
        for (name, value) in request.headers() {
            sent = sent.header(name, value);
        }
        let sent = sent.body(Body::from(request.body().to_vec())).expect("a stated request is a request");
        let answered = self.0.clone().oneshot(sent).await.expect("this surface answers every request");

        let status = HttpStatus::new(answered.status().as_u16());
        let headers = answered
            .headers()
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or_default().to_owned()))
            .collect::<Vec<_>>();
        let body = to_bytes(answered.into_body(), usize::MAX).await.unwrap_or_default();

        headers
            .into_iter()
            .fold(DirectResponse::new(status), |answer, (name, value)| answer.with_header(&name, &value))
            .with_body(body.to_vec())
    }
}

#[tokio::test]
async fn answers_the_shared_surface_as_the_scenario_states() {
    let api = AxumHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.shop_api::<Shop>());

    assert_eq!(compiled.labels(), LABELS);
    expect(&Answering(compiled.into_axum())).await.unwrap();
}

#[test]
fn compiles_the_widest_endpoint_the_specification_states() {
    // The assertion is the compile: sixteen arguments reach this interpretation's products.
    let api = AxumHandlerImpl::new(Shop);

    assert_eq!(api.compile_http(api.wide_api::<Shop>()).labels(), ["POST /wide"]);
}

#[tokio::test]
async fn answers_a_body_produced_over_time() {
    let api = AxumHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.stream_api::<Shop>());

    assert_eq!(compiled.labels(), ["GET /ticks"]);
    expect_streaming(&Answering(compiled.into_axum())).await.unwrap();
}

#[tokio::test]
async fn reads_a_body_arriving_as_parts() {
    let api = AxumHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.multipart_api::<Shop>());

    assert_eq!(compiled.labels(), ["POST /upload"]);
    expect_multipart(&Answering(compiled.into_axum())).await.unwrap();
}
