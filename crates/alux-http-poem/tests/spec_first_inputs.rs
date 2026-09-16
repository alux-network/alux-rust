//! Declares one surface reading each request body role, and holds both interpretations to it.
//!
//! An input role states where an argument comes from, not how a framework reads it. The same
//! handler argument arrives as JSON, as a form, or as the bytes that were sent, and only the
//! declaration says which.

#![allow(async_fn_in_trait)]

use alux_ext::ext;
use alux_http::{HttpApiAlg, HttpProgramExt, JsonOutAlg, TextOutAlg, http};
use alux_http_poem::PoemHandlerImpl;
use alux_http_text::TextHandlerImpl;
use core::future::Future;
use poem::http::Method;
use poem::{Endpoint, Request, Response};
use serde::Deserialize;

/// Carries one reading sent by a caller.
#[derive(Debug, Deserialize)]
struct Amount {
    value: u32,
}

/// Takes in whatever a caller sends.
trait IntakeAlg {
    /// Returns the reading that was sent.
    fn take(&self, value: u32) -> impl Future<Output = u32> + Send;
    /// Returns what the sent bytes say.
    fn take_raw(&self, body: Vec<u8>) -> impl Future<Output = String> + Send;
}

#[ext(name = IntakeOperationExt, defunc)]
impl<This> This
where
    This: IntakeAlg,
{
    /// Returns the reading sent as a document.
    async fn intake_sent(&self, amount: u32) -> u32 {
        self.take(amount).await
    }

    /// Returns the reading sent as a form.
    async fn intake_filled(&self, amount: Amount) -> u32 {
        self.take(amount.value).await
    }

    /// Returns what the bytes that were sent say.
    async fn intake_raw(&self, body: Vec<u8>) -> String {
        self.take_raw(body).await
    }
}

#[ext(name = IntakeApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg + JsonOutAlg + TextOutAlg,
{
    /// Declares one endpoint per body role, all reaching the same reading.
    fn intake_api<Alg>(&self)
    where
        Alg: IntakeAlg,
    {
        self.routes()
            // A reading sent as a document.
            .post("/sent", self.op(Alg::intake_sent).body::<u32>().json())
            // The same reading, sent as a form.
            .post("/filled", self.op(Alg::intake_filled).form::<Amount>().json())
            // The bytes exactly as they arrived.
            .post("/raw", self.op(Alg::intake_raw).raw_body::<Vec<u8>>().text())
    }
}

struct Intake;

impl IntakeAlg for Intake {
    async fn take(&self, value: u32) -> u32 {
        value
    }

    async fn take_raw(&self, body: Vec<u8>) -> String {
        String::from_utf8(body).unwrap_or_default()
    }
}

async fn post<E>(app: &E, path: &str, content_type: &str, body: &'static str) -> Response
where
    E: Endpoint<Output = Response>,
{
    let request = Request::builder().method(Method::POST).uri_str(path).content_type(content_type).body(body);

    app.call(request).await.unwrap()
}

#[tokio::test]
async fn reads_one_argument_from_whichever_body_role_the_declaration_states() {
    let api = PoemHandlerImpl::new(Intake);
    let app = api.compile_http(api.intake_api::<Intake>()).into_poem();

    let mut sent = post(&app, "/sent", "application/json", "7").await;
    assert!(sent.status().is_success());
    assert_eq!(sent.take_body().into_string().await.unwrap(), "7");

    let mut filled = post(&app, "/filled", "application/x-www-form-urlencoded", "value=7").await;
    assert!(filled.status().is_success());
    assert_eq!(filled.take_body().into_string().await.unwrap(), "7");

    let mut raw = post(&app, "/raw", "application/octet-stream", "seven").await;
    assert!(raw.status().is_success());
    assert_eq!(raw.take_body().into_string().await.unwrap(), "seven");
}

#[test]
fn describes_the_role_each_argument_is_read_from() {
    let text = TextHandlerImpl;
    let poem = PoemHandlerImpl::new(Intake);
    let described = text.compile_http(text.intake_api::<Intake>());
    let compiled = poem.compile_http(poem.intake_api::<Intake>());

    assert_eq!(compiled.labels(), described.labels());
    assert_eq!(compiled.labels(), ["POST /sent", "POST /filled", "POST /raw"]);

    let lines = described.lines();
    assert!(lines[0].contains("BodyRole"));
    assert!(lines[1].contains("FormRole"));
    assert!(lines[2].contains("RawBodyRole"));
}
