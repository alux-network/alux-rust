//! Holds the surface served over hyper to the shared scenario.
//!
//! What answers here is `alux-http-direct`: it does the routing and the reading. This crate only
//! carries the bytes, so passing the same exchanges as every framework is what shows the transport
//! changes nothing about what a surface means.

use alux_http::{HttpMethod, HttpProgramExt, HttpStatus};
use alux_http_conformance::{
    AnswerAlg, LABELS, MultipartApiExt, Shop, ShopApiExt, StreamApiExt, WideApiExt, expect, expect_multipart,
    expect_streaming,
};
use alux_http_direct::{DirectHandlerImpl, DirectRequest, DirectResponse};
use alux_http_hyper::HyperRoute;
use bytes::Bytes;
use http_body_util::{BodyExt, Collected, Full};
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Method, Request};

/// Answers a stated request by sending it through hyper, as a caller's connection would.
struct Answering(HyperRoute);

impl AnswerAlg for Answering {
    async fn answer(&self, request: DirectRequest) -> DirectResponse {
        let method = request
            .method()
            .and_then(|method| Method::from_bytes(method.label().as_bytes()).ok())
            .unwrap_or(Method::GET);
        let mut sent = Request::builder().method(method).uri(request.path());
        for (name, value) in request.headers() {
            let name = HeaderName::from_bytes(name.as_bytes()).expect("a stated header is a header");
            let value = HeaderValue::from_str(value).expect("a stated value is a value");
            sent = sent.header(name, value);
        }
        let sent = sent.body(Full::new(Bytes::from(request.body().to_vec()))).expect("a stated request is a request");
        let answered = self.0.answer(sent).await;

        let status = HttpStatus::new(answered.status().as_u16());
        let headers = answered
            .headers()
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or_default().to_owned()))
            .collect::<Vec<_>>();
        let body = answered.into_body().collect().await.map(Collected::to_bytes).unwrap_or_default();

        headers
            .into_iter()
            .fold(DirectResponse::new(status), |answer, (name, value)| answer.with_header(&name, &value))
            .with_body(body.to_vec())
    }
}

fn serving<Api>(api: Api) -> Answering
where
    Api: FnOnce() -> alux_http_direct::DirectRoute,
{
    Answering(HyperRoute::new(api()))
}

#[tokio::test]
async fn answers_the_shared_surface_as_the_scenario_states() {
    let api = DirectHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.shop_api::<Shop>());

    assert_eq!(compiled.labels(), LABELS);
    expect(&serving(|| compiled)).await.unwrap();
}

#[tokio::test]
async fn answers_a_body_produced_over_time() {
    let api = DirectHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.stream_api::<Shop>());

    assert_eq!(compiled.labels(), ["GET /ticks"]);
    expect_streaming(&serving(|| compiled)).await.unwrap();
}

#[tokio::test]
async fn reads_a_body_arriving_as_parts() {
    let api = DirectHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.multipart_api::<Shop>());

    assert_eq!(compiled.labels(), ["POST /upload"]);
    expect_multipart(&serving(|| compiled)).await.unwrap();
}

#[test]
fn compiles_the_widest_endpoint_the_specification_states() {
    let api = DirectHandlerImpl::new(Shop);

    assert_eq!(api.compile_http(api.wide_api::<Shop>()).labels(), ["POST /wide"]);
}

#[tokio::test]
async fn answers_a_method_it_states_nothing_for() {
    let api = DirectHandlerImpl::new(Shop);
    let route = HyperRoute::new(api.compile_http(api.shop_api::<Shop>()));
    let sent = Request::builder()
        .method(Method::from_bytes(b"PROPFIND").expect("an extension method is a method"))
        .uri("/items")
        .body(Full::new(Bytes::new()))
        .expect("a stated request is a request");

    // A method the specification names none of reaches no endpoint, and is still answered.
    assert_eq!(route.answer(sent).await.status(), HttpStatus::METHOD_NOT_ALLOWED.code());
    let _ = HttpMethod::ALL;
}
