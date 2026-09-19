//! Reads one declared surface from the caller's side.
//!
//! The same declaration an executing interpretation answers is folded here into the module that
//! calls it. Nothing about the surface is written twice, which is what makes a client and a service
//! unable to disagree about it.

use alux_ext::ext;
use alux_http::{
    EmptyOutAlg, HttpApiAlg, HttpProgramExt, JsonOutAlg, NamedValuesAlg, ResultOutAlg, StatusOutAlg, TextOutAlg, http,
};
use alux_http_text::TextHandlerImpl;
use alux_http_typescript::TsHttpClient;
use alux_shape::{Shape, Spelling};
use core::future::Future;
use serde::{Deserialize, Serialize};
use std::io::{Error as IoError, ErrorKind};

/// What one reading carries. The layout states its own shape.
#[derive(Debug, Serialize, Deserialize, Shape)]
#[serde(rename_all = "camelCase")]
pub struct Reading {
    /// Which reading this is.
    pub id: u64,
    /// Whatever was said about it, where anything was.
    pub display_note: Option<String>,
}

/// What a caller searches by, stated as the query string carries it.
#[derive(Debug, Serialize, Deserialize, Shape)]
pub struct Term {
    /// The term to match against.
    pub term: String,
}

impl NamedValuesAlg for Term {}

/// Keeps whatever readings the domain has been told about.
trait ReadingsAlg {
    /// Returns the reading recorded for `id`, or why it could not be read.
    fn reading(&self, id: u64) -> impl Future<Output = Result<Reading, IoError>> + Send;
    /// Records a reading and returns it.
    fn record(&self, reading: Reading) -> impl Future<Output = Reading> + Send;
    /// Returns the readings as a line of prose.
    fn summary(&self, term: String) -> impl Future<Output = String> + Send;
    /// Forgets every reading.
    fn forget(&self) -> impl Future<Output = ()> + Send;
}

#[ext(name = ReadingsOperationExt, defunc)]
impl<This> This
where
    This: ReadingsAlg,
{
    /// Returns one identified reading, or why it could not be read.
    async fn reading_at(&self, id: u64) -> Result<Reading, IoError> {
        self.reading(id).await
    }

    /// Records one reading and returns it.
    async fn reading_record(&self, reading: Reading) -> Reading {
        self.record(reading).await
    }

    /// Returns the readings matching a term, as prose.
    async fn reading_summary(&self, search_term: Term) -> String {
        self.summary(search_term.term).await
    }

    /// Forgets every reading.
    async fn reading_forget(&self) {
        self.forget().await;
    }
}

#[ext(name = ReadingsApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg + JsonOutAlg + TextOutAlg + EmptyOutAlg + StatusOutAlg + ResultOutAlg,
{
    /// Declares the readings surface.
    fn readings_api<Alg>(&self)
    where
        Alg: ReadingsAlg,
    {
        self.routes()
            // One identified reading, or what its failure means.
            .get("/readings/:id", self.op(Alg::reading_at).path::<u64>().json().result())
            // A summary, its term taken from the query string.
            .get("/readings", self.op(Alg::reading_summary).query::<Term>().text())
            // A recording, which creates something and says so.
            .post("/readings", self.op(Alg::reading_record).body::<Reading>().json().status::<201>())
            // A removal, which answers with nothing at all.
            .delete("/readings", self.op(Alg::reading_forget).empty())
    }
}

struct Readings;

impl ReadingsAlg for Readings {
    async fn reading(&self, id: u64) -> Result<Reading, IoError> {
        match id {
            1 => Ok(Reading { id, display_note: None }),
            _ => Err(IoError::new(ErrorKind::NotFound, "no such reading")),
        }
    }

    async fn record(&self, reading: Reading) -> Reading {
        reading
    }

    async fn summary(&self, _term: String) -> String {
        String::new()
    }

    async fn forget(&self) {}
}

fn module() -> String {
    let client = TsHttpClient::new(Spelling::LowerCamel);

    client.compile_http(client.readings_api::<Readings>()).render()
}

#[test]
fn writes_one_call_per_endpoint_under_the_name_it_was_declared_with() {
    let client = TsHttpClient::new(Spelling::LowerCamel);
    let compiled = client.compile_http(client.readings_api::<Readings>());

    assert_eq!(compiled.call_names(), ["readingAt", "readingForget", "readingRecord", "readingSummary"]);
}

#[test]
fn states_the_method_and_path_each_call_is_made_under() {
    let module = module();

    // A described path is already a template, so nothing is spelled a second way for a caller.
    assert!(module.contains(r#"endpoint<[id: number], Reading>("GET", "/readings/{id}", ["path"])"#), "{module}");
    assert!(module.contains(r#"endpoint<[reading: Reading], Reading>("POST", "/readings", ["body"])"#), "{module}");
    assert!(module.contains(r#"endpoint<[searchTerm: Term], string>("GET", "/readings", ["query"])"#), "{module}");
    assert!(module.contains(r#"endpoint<[], null>("DELETE", "/readings", [])"#), "{module}");
}

#[test]
fn declares_every_type_its_calls_depend_on_once() {
    let module = module();

    assert!(module.contains("export interface Reading {"), "{module}");
    assert_eq!(module.matches("export interface Reading").count(), 1, "{module}");
    // A member is spelled the way the surface spells names, which is what the wire carries.
    assert!(module.contains("displayNote"), "{module}");
}

#[test]
fn calls_the_surface_the_other_interpretations_compile() {
    let client = TsHttpClient::new(Spelling::LowerCamel);
    let text = TextHandlerImpl;

    let described = text.compile_http(text.readings_api::<Readings>());
    let called = client.compile_http(client.readings_api::<Readings>());

    assert_eq!(called.labels(), described.labels());
    assert_eq!(called.labels(), ["GET /readings/{id}", "GET /readings", "POST /readings", "DELETE /readings",]);
}

#[test]
fn says_what_each_call_is_for() {
    let module = module();

    assert!(module.contains("/** Returns one identified reading, or why it could not be read. */"), "{module}");
    assert!(module.contains("/** Records one reading and returns it. */"), "{module}");
}
