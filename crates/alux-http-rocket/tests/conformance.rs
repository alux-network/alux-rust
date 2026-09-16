//! Holds this interpretation to the shared surface and the shared scenario.
//!
//! The adapter below is the only place Rocket is named: everything the scenario states is stated
//! without a framework, so what is compared is the surface rather than the response type.

use alux_http::{HttpMethod, HttpProgramExt, HttpStatus};
use alux_http_conformance::{
    AnswerAlg, LABELS, MultipartApiExt, Shop, ShopApiExt, StreamApiExt, WideApiExt, expect, expect_multipart,
    expect_streaming,
};
use alux_http_direct::{DirectRequest, DirectResponse};
use alux_http_rocket::RocketHandlerImpl;
use rocket::http::{Header, Method};
use rocket::local::asynchronous::Client;

/// Answers a stated request with a Rocket client.
struct Answering(Client);

impl AnswerAlg for Answering {
    async fn answer(&self, request: DirectRequest) -> DirectResponse {
        let method = match request.method().map(HttpMethod::label) {
            Some("POST") => Method::Post,
            Some("PUT") => Method::Put,
            Some("PATCH") => Method::Patch,
            Some("DELETE") => Method::Delete,
            Some("HEAD") => Method::Head,
            Some("OPTIONS") => Method::Options,
            _ => Method::Get,
        };
        let mut sent = self.0.req(method, request.path().to_owned());
        for (name, value) in request.headers() {
            sent = sent.header(Header::new(name.to_owned(), value.to_owned()));
        }
        let answered = sent.body(request.body()).dispatch().await;

        let status = HttpStatus::new(answered.status().code);
        let headers = answered
            .headers()
            .iter()
            .map(|header| (header.name().to_string(), header.value().to_owned()))
            .collect::<Vec<_>>();
        let body = answered.into_bytes().await.unwrap_or_default();

        headers
            .into_iter()
            .fold(DirectResponse::new(status), |answer, (name, value)| answer.with_header(&name, &value))
            .with_body(body)
    }
}

#[tokio::test]
async fn answers_the_shared_surface_as_the_scenario_states() {
    let api = RocketHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.shop_api::<Shop>());

    assert_eq!(compiled.labels(), LABELS);

    let client = Client::tracked(compiled.mount(rocket::build())).await.expect("this surface mounts");
    expect(&Answering(client)).await.unwrap();
}

#[test]
fn compiles_the_widest_endpoint_the_specification_states() {
    // The assertion is the compile: sixteen arguments reach this interpretation's products.
    let api = RocketHandlerImpl::new(Shop);

    assert_eq!(api.compile_http(api.wide_api::<Shop>()).labels(), ["POST /wide"]);
}

#[tokio::test]
async fn answers_a_body_produced_over_time() {
    let api = RocketHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.stream_api::<Shop>());

    assert_eq!(compiled.labels(), ["GET /ticks"]);
    expect_streaming(&Answering(Client::tracked(compiled.mount(rocket::build())).await.expect("this surface mounts")))
        .await
        .unwrap();
}

#[tokio::test]
async fn reads_a_body_arriving_as_parts() {
    let api = RocketHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.multipart_api::<Shop>());

    assert_eq!(compiled.labels(), ["POST /upload"]);
    expect_multipart(&Answering(Client::tracked(compiled.mount(rocket::build())).await.expect("this surface mounts")))
        .await
        .unwrap();
}
