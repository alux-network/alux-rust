//! Holds this interpretation to the shared surface and the shared scenario.
//!
//! The adapter below is the only place actix-web is named. It configures a fresh service for every
//! exchange, which is also what a running server does for every worker.

use actix_web::App;
use actix_web::body::to_bytes;
use actix_web::http::Method;
use actix_web::test::{TestRequest, call_service, init_service};
use alux_http::{HttpProgramExt, HttpStatus};
use alux_http_actix::ActixHandlerImpl;
use alux_http_conformance::{
    AnswerAlg, LABELS, MultipartApiExt, Shop, ShopApiExt, StreamApiExt, WideApiExt, expect, expect_multipart,
    expect_streaming,
};
use alux_http_direct::{DirectRequest, DirectResponse};
use core::future::Future;

/// Answers a stated request with an actix-web service.
struct Answering<Configure>(Configure);

impl<Configure> AnswerAlg for Answering<Configure>
where
    Configure: Fn(&mut actix_web::web::ServiceConfig) + Clone + Send + 'static,
{
    fn answer(&self, request: DirectRequest) -> impl Future<Output = DirectResponse> + Send {
        let configure = self.0.clone();

        // actix-web runs its services on a runtime of its own, so the exchange is answered there.
        let answered = std::thread::spawn(move || {
            actix_web::rt::System::new().block_on(async move {
                let service = init_service(App::new().configure(configure)).await;
                let method = request
                    .method()
                    .and_then(|method| Method::from_bytes(method.label().as_bytes()).ok())
                    .unwrap_or(Method::GET);
                let mut sent = TestRequest::with_uri(request.path()).method(method);
                for (name, value) in request.headers() {
                    sent = sent.insert_header((name.to_owned(), value.to_owned()));
                }
                let answered = call_service(&service, sent.set_payload(request.body().to_vec()).to_request()).await;

                let status = HttpStatus::new(answered.status().as_u16());
                let headers = answered
                    .headers()
                    .iter()
                    .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or_default().to_owned()))
                    .collect::<Vec<_>>();
                let body = to_bytes(answered.into_body()).await.unwrap_or_default();

                headers
                    .into_iter()
                    .fold(DirectResponse::new(status), |answer, (name, value)| answer.with_header(&name, &value))
                    .with_body(body.to_vec())
            })
        })
        .join()
        .expect("this surface answers every request");

        async move { answered }
    }
}

#[tokio::test]
async fn answers_the_shared_surface_as_the_scenario_states() {
    let api = ActixHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.shop_api::<Shop>());

    assert_eq!(compiled.labels(), LABELS);
    expect(&Answering(compiled.into_actix())).await.unwrap();
}

#[test]
fn compiles_the_widest_endpoint_the_specification_states() {
    // The assertion is the compile: sixteen arguments reach this interpretation's products.
    let api = ActixHandlerImpl::new(Shop);

    assert_eq!(api.compile_http(api.wide_api::<Shop>()).labels(), ["POST /wide"]);
}

#[tokio::test]
async fn answers_a_body_produced_over_time() {
    let api = ActixHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.stream_api::<Shop>());

    assert_eq!(compiled.labels(), ["GET /ticks"]);
    expect_streaming(&Answering(compiled.into_actix())).await.unwrap();
}

#[tokio::test]
async fn reads_a_body_arriving_as_parts() {
    let api = ActixHandlerImpl::new(Shop);
    let compiled = api.compile_http(api.multipart_api::<Shop>());

    assert_eq!(compiled.labels(), ["POST /upload"]);
    expect_multipart(&Answering(compiled.into_actix())).await.unwrap();
}
