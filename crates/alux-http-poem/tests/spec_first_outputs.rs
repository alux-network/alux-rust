//! Declares one surface stating every output kind, and holds both interpretations to it.
//!
//! An output kind is what an endpoint answers with, so the evidence that a kind means something is
//! a response: a status, a header, a body. The text interpretation describes the same declaration
//! without answering anything, which is what shows the kinds are stated by the program rather than
//! chosen by the framework.

use alux_ext::ext;
use alux_http::{
    BytesOutAlg, EmptyOutAlg, HtmlOutAlg, HttpApiAlg, HttpProgramExt, JsonOutAlg, RedirectOutAlg, ResultOutAlg,
    StatusOutAlg, TextOutAlg, http,
};
use alux_http_poem::PoemHandlerImpl;
use alux_http_text::TextHandlerImpl;
use core::future::Future;
use poem::http::{Method, StatusCode, header};
use poem::{Endpoint, Request, Response};
use std::io::{Error as IoError, ErrorKind};

/// Reports whatever the domain has to say, in whatever shape a caller asked for.
trait ReportAlg {
    /// Returns the report as a line of prose.
    fn note(&self) -> impl Future<Output = String> + Send;
    /// Returns the report as a page.
    fn page(&self) -> impl Future<Output = String> + Send;
    /// Returns the report as it is stored.
    fn raw(&self) -> impl Future<Output = Vec<u8>> + Send;
    /// Records `value` and returns what was recorded.
    fn record(&self, value: u32) -> impl Future<Output = u32> + Send;
    /// Forgets everything recorded.
    fn forget(&self) -> impl Future<Output = ()> + Send;
    /// Returns where the report actually lives.
    fn elsewhere(&self) -> impl Future<Output = String> + Send;
    /// Returns the reading recorded for `id`, or why it could not be read.
    fn find(&self, id: u32) -> impl Future<Output = Result<u32, IoError>> + Send;
}

#[ext(name = ReportOperationExt, defunc)]
impl<This> This
where
    This: ReportAlg,
{
    /// Returns the report as a line of prose.
    async fn report_note(&self) -> String {
        self.note().await
    }

    /// Returns the report as a page.
    async fn report_page(&self) -> String {
        self.page().await
    }

    /// Returns the report as it is stored.
    async fn report_raw(&self) -> Vec<u8> {
        self.raw().await
    }

    /// Records one reading and returns it.
    async fn report_record(&self, value: u32) -> u32 {
        self.record(value).await
    }

    /// Forgets everything recorded.
    async fn report_forget(&self) {
        self.forget().await;
    }

    /// Returns where the report actually lives.
    async fn report_elsewhere(&self) -> String {
        self.elsewhere().await
    }

    /// Returns one identified reading, or why it could not be read.
    async fn report_find(&self, id: u32) -> Result<u32, IoError> {
        self.find(id).await
    }
}

#[ext(name = ReportApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg
        + JsonOutAlg
        + TextOutAlg
        + HtmlOutAlg
        + BytesOutAlg
        + EmptyOutAlg
        + RedirectOutAlg
        + StatusOutAlg
        + ResultOutAlg,
{
    /// Declares one endpoint per output kind the specification names.
    fn report_api<Alg>(&self)
    where
        Alg: ReportAlg,
    {
        self.routes()
            // A line of prose.
            .get("/note", self.op(Alg::report_note).text())
            // A page.
            .get("/page", self.op(Alg::report_page).html())
            // The report as it is stored.
            .get("/raw", self.op(Alg::report_raw).bytes())
            // A recording, which creates something and says so.
            .post("/record", self.op(Alg::report_record).body::<u32>().json().status::<201>())
            // A removal, which answers with nothing at all.
            .delete("/record", self.op(Alg::report_forget).empty())
            // Where the report actually lives.
            .get("/elsewhere", self.op(Alg::report_elsewhere).redirect())
            // One identified reading, or what its failure means.
            .get("/find/{id}", self.op(Alg::report_find).path::<u32>().json().result())
    }
}

struct Reports;

impl ReportAlg for Reports {
    async fn note(&self) -> String {
        "all quiet".to_owned()
    }

    async fn page(&self) -> String {
        "<p>all quiet</p>".to_owned()
    }

    async fn raw(&self) -> Vec<u8> {
        b"quiet".to_vec()
    }

    async fn record(&self, value: u32) -> u32 {
        value
    }

    async fn forget(&self) {}

    async fn elsewhere(&self) -> String {
        "/note".to_owned()
    }

    async fn find(&self, id: u32) -> Result<u32, IoError> {
        match id {
            1 => Ok(7),
            _ => Err(IoError::new(ErrorKind::NotFound, "no such reading")),
        }
    }
}

async fn call<E>(app: &E, request: Request) -> Response
where
    E: Endpoint<Output = Response>,
{
    app.call(request).await.unwrap()
}

#[tokio::test]
async fn answers_with_what_each_output_kind_states() {
    let api = PoemHandlerImpl::new(Reports);
    let app = api.compile_http(api.report_api::<Reports>()).into_poem();

    let mut note = call(&app, Request::builder().uri_str("/note").finish()).await;
    assert!(note.status().is_success());
    assert_eq!(note.take_body().into_string().await.unwrap(), "all quiet");

    let mut page = call(&app, Request::builder().uri_str("/page").finish()).await;
    assert_eq!(page.headers().get(header::CONTENT_TYPE).unwrap(), "text/html; charset=utf-8");
    assert_eq!(page.take_body().into_string().await.unwrap(), "<p>all quiet</p>");

    let mut raw = call(&app, Request::builder().uri_str("/raw").finish()).await;
    assert_eq!(raw.take_body().into_vec().await.unwrap(), b"quiet");
}

#[tokio::test]
async fn answers_with_the_status_the_declaration_states() {
    let api = PoemHandlerImpl::new(Reports);
    let app = api.compile_http(api.report_api::<Reports>()).into_poem();

    let request = Request::builder().method(Method::POST).uri_str("/record").content_type("application/json").body("7");
    let mut created = call(&app, request).await;

    assert_eq!(created.status(), StatusCode::CREATED);
    assert_eq!(created.take_body().into_string().await.unwrap(), "7");

    let request = Request::builder().method(Method::DELETE).uri_str("/record").finish();
    let mut forgotten = call(&app, request).await;

    assert_eq!(forgotten.status(), StatusCode::NO_CONTENT);
    assert_eq!(forgotten.take_body().into_string().await.unwrap(), "");

    let elsewhere = call(&app, Request::builder().uri_str("/elsewhere").finish()).await;

    assert_eq!(elsewhere.status(), StatusCode::SEE_OTHER);
    assert_eq!(elsewhere.headers().get(header::LOCATION).unwrap(), "/note");
}

#[tokio::test]
async fn answers_with_what_a_failure_means() {
    let api = PoemHandlerImpl::new(Reports);
    let app = api.compile_http(api.report_api::<Reports>()).into_poem();

    let mut found = call(&app, Request::builder().uri_str("/find/1").finish()).await;

    assert!(found.status().is_success());
    assert_eq!(found.take_body().into_string().await.unwrap(), "7");

    let mut missing = call(&app, Request::builder().uri_str("/find/9").finish()).await;

    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert_eq!(missing.take_body().into_string().await.unwrap(), "no such reading");
}

#[test]
fn describes_the_kind_each_endpoint_states() {
    let text = TextHandlerImpl;
    let poem = PoemHandlerImpl::new(Reports);
    let described = text.compile_http(text.report_api::<Reports>());
    let compiled = poem.compile_http(poem.report_api::<Reports>());

    assert_eq!(compiled.labels(), described.labels());
    assert_eq!(
        compiled.labels(),
        ["GET /note", "GET /page", "GET /raw", "POST /record", "DELETE /record", "GET /elsewhere", "GET /find/{id}",]
    );

    let lines = described.lines();
    assert!(lines[0].contains("TextTextOutput"));
    assert!(lines[1].contains("TextHtmlOutput"));
    assert!(lines[2].contains("TextBytesOutput"));
    assert!(lines[3].contains("TextStatusOutput<alux_http_text::output::TextJsonOutput, 201>"));
    assert!(lines[4].contains("TextEmptyOutput"));
    assert!(lines[5].contains("TextRedirectOutput"));
    assert!(lines[6].contains("TextResultOutput"));
}
