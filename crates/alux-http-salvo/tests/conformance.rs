//! Holds this interpretation to the shared surface and the shared scenario.
//!
//! The adapter below is the only place Salvo is named: everything the scenario states is stated
//! without a framework, so what is compared is the surface rather than the response type.

use alux_http::{HttpProgramExt, HttpStatus};
use alux_http_conformance::{
    AnswerAlg, LABELS, MultipartApiExt, Shop, ShopApiExt, StreamApiExt, WideApiExt, expect, expect_multipart,
    expect_streaming,
};
use alux_http_direct::{DirectRequest, DirectResponse};
use alux_http_salvo::SalvoHandlerImpl;
use salvo::Service;
use salvo::http::{HeaderName, Method};
use salvo::test::{RequestBuilder, ResponseExt};

/// Answers a stated request with a Salvo service.
struct Answering(Service);

impl AnswerAlg for Answering {
    async fn answer(&self, request: DirectRequest) -> DirectResponse {
        let method = request
            .method()
            .and_then(|method| Method::from_bytes(method.label().as_bytes()).ok())
            .unwrap_or(Method::GET);
        let target = format!("http://conformance{}", request.path());
        let mut sent = RequestBuilder::new(target, method);
        for (name, value) in request.headers() {
            let name = HeaderName::from_bytes(name.as_bytes()).expect("a stated header is a header");
            sent = sent.add_header(name, value.to_owned(), true);
        }
        let mut answered = sent.body(request.body().to_vec()).send(&self.0).await;

        let status = HttpStatus::new(answered.status_code.unwrap_or(salvo::http::StatusCode::OK).as_u16());
        let headers = answered
            .headers()
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or_default().to_owned()))
            .collect::<Vec<_>>();
        let body = answered.take_bytes(None).await.unwrap_or_default();

        headers
            .into_iter()
            .fold(DirectResponse::new(status), |answer, (name, value)| answer.with_header(&name, &value))
            .with_body(body.to_vec())
    }
}

#[tokio::test]
async fn answers_the_shared_surface_as_the_scenario_states() {
    let api = SalvoHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.shop_api::<Shop>());

    assert_eq!(compiled.labels(), LABELS);
    expect(&Answering(Service::new(compiled.into_salvo()))).await.unwrap();
}

#[test]
fn compiles_the_widest_endpoint_the_specification_states() {
    // The assertion is the compile: sixteen arguments reach this interpretation's products.
    let api = SalvoHandlerImpl::new(Shop);

    assert_eq!(api.compile_http(api.wide_api::<Shop>()).labels(), ["POST /wide"]);
}

#[tokio::test]
async fn answers_a_body_produced_over_time() {
    let api = SalvoHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.stream_api::<Shop>());

    assert_eq!(compiled.labels(), ["GET /ticks"]);
    expect_streaming(&Answering(Service::new(compiled.into_salvo()))).await.unwrap();
}

#[tokio::test]
async fn reads_a_body_arriving_as_parts() {
    let api = SalvoHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.multipart_api::<Shop>());

    assert_eq!(compiled.labels(), ["POST /upload"]);
    expect_multipart(&Answering(Service::new(compiled.into_salvo()))).await.unwrap();
}
