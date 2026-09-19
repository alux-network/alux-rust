//! Reads one declared surface as the document that describes it.
//!
//! A document is the strictest reader a program has. It cannot be written unless the program states
//! an operation's name, what each argument is called and where it comes from, the shape of
//! everything crossing the wire, and every status an endpoint can answer with. Nothing here is
//! restated: it is the same declaration the executing interpretations compile.

use alux_ext::ext;
use alux_http::{
    EmptyOutAlg, HttpApiAlg, HttpProgramExt, JsonOutAlg, NamedValuesAlg, ResultOutAlg, StatusOutAlg, http,
};
use alux_http_openapi::OpenApiHandlerImpl;
use alux_http_text::TextHandlerImpl;
use alux_shape::Shape;
use core::future::Future;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::{Error as IoError, ErrorKind};

/// What one reading carries. The layout states its own shape.
#[derive(Debug, Serialize, Deserialize, Shape)]
pub struct Reading {
    /// Which reading this is.
    pub id: u64,
    /// What was read.
    pub value: u32,
    /// Whatever was said about it, where anything was.
    pub note: Option<String>,
}

/// What a caller searches by, stated as the query string carries it.
#[derive(Debug, Serialize, Deserialize, Shape)]
pub struct Term {
    /// The term to match a note against.
    pub term: String,
}

impl NamedValuesAlg for Term {}

/// Keeps whatever readings the domain has been told about.
trait ReadingsAlg {
    /// Returns the reading recorded for `id`, or why it could not be read.
    fn reading(&self, id: u64) -> impl Future<Output = Result<Reading, IoError>> + Send;
    /// Records a reading and returns it.
    fn record(&self, reading: Reading) -> impl Future<Output = Reading> + Send;
    /// Returns every reading whose note says `term`.
    fn search(&self, term: String) -> impl Future<Output = Vec<Reading>> + Send;
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

    /// Returns every reading matching a term.
    async fn reading_search(&self, term: Term) -> Vec<Reading> {
        self.search(term.term).await
    }

    /// Forgets every reading.
    async fn reading_forget(&self) {
        self.forget().await;
    }
}

#[ext(name = ReadingsApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg + JsonOutAlg + EmptyOutAlg + StatusOutAlg + ResultOutAlg,
{
    /// Declares the readings surface.
    fn readings_api<Alg>(&self)
    where
        Alg: ReadingsAlg,
    {
        self.routes()
            // One identified reading, or what its failure means.
            .get("/readings/:id", self.op(Alg::reading_at).path::<u64>().json().result())
            // A search, its term taken from the query string.
            .get("/readings", self.op(Alg::reading_search).query::<Term>().json())
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
            1 => Ok(Reading { id, value: 7, note: None }),
            _ => Err(IoError::new(ErrorKind::NotFound, "no such reading")),
        }
    }

    async fn record(&self, reading: Reading) -> Reading {
        reading
    }

    async fn search(&self, _term: String) -> Vec<Reading> {
        Vec::new()
    }

    async fn forget(&self) {}
}

fn document() -> Value {
    let api = OpenApiHandlerImpl::<Readings>::new();
    let route = api.compile_http(api.readings_api::<Readings>());

    api.document("readings", "1.0", &route)
}

fn operation(document: &Value, path: &str, method: &str) -> Value {
    document["paths"][path][method].clone()
}

#[test]
fn keys_an_operation_by_the_path_and_method_it_answers_on() {
    let document = document();

    assert_eq!(document["openapi"], "3.1.0");
    assert_eq!(document["info"]["title"], "readings");

    // The declaration writes `:id`; a document templates a path the way a description states one.
    let mut paths = document["paths"].as_object().unwrap().keys().cloned().collect::<Vec<_>>();
    paths.sort();
    assert_eq!(paths, ["/readings", "/readings/{id}"]);

    let methods = document["paths"]["/readings"].as_object().unwrap().keys().cloned().collect::<Vec<_>>();
    assert_eq!(methods, ["delete", "get", "post"]);
}

#[test]
fn names_each_operation_as_it_was_declared() {
    let document = document();

    // A type name says where an operation lives; only the operation says what it was called.
    assert_eq!(operation(&document, "/readings/{id}", "get")["operationId"], "reading_at");
    assert_eq!(operation(&document, "/readings", "post")["operationId"], "reading_record");
    assert_eq!(operation(&document, "/readings", "delete")["operationId"], "reading_forget");
}

#[test]
fn describes_each_argument_where_the_declaration_reads_it_from() {
    let document = document();

    // An argument's name comes from the source it was declared with, and its place from its role.
    assert_eq!(
        operation(&document, "/readings/{id}", "get")["parameters"],
        json!([{
            "name": "id",
            "in": "path",
            "required": true,
            "schema": { "type": "integer", "format": "int64", "minimum": 0 },
        }])
    );

    // A query string carries names and values, so what a document keys is the member, not the
    // argument that reads it.
    assert_eq!(
        operation(&document, "/readings", "get")["parameters"],
        json!([{
            "name": "term",
            "in": "query",
            "required": true,
            "schema": { "type": "string" },
        }])
    );

    // A body is not a parameter, and the media type is the role it was declared under.
    let posted = operation(&document, "/readings", "post");
    assert_eq!(posted["requestBody"]["required"], true);
    assert_eq!(
        posted["requestBody"]["content"]["application/json"]["schema"],
        json!({ "$ref": "#/components/schemas/Reading" })
    );
    assert!(posted["parameters"].is_null());
}

#[test]
fn states_the_shape_of_everything_that_crosses_the_wire() {
    let document = document();

    // A layout that states a name is stated once, and referred to wherever it is used.
    assert_eq!(
        document["components"]["schemas"]["Reading"],
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "integer", "format": "int64", "minimum": 0 },
                "value": { "type": "integer", "format": "int32", "minimum": 0 },
                "note": { "anyOf": [{ "type": "string" }, { "type": "null" }] },
            },
            "required": ["id", "value"],
        })
    );

    let searched = operation(&document, "/readings", "get");
    assert_eq!(
        searched["responses"]["200"]["content"]["application/json"]["schema"],
        json!({
            "type": "array",
            "items": { "$ref": "#/components/schemas/Reading" },
        })
    );
}

#[test]
fn states_every_status_an_endpoint_can_answer_with() {
    let document = document();

    // A declared status is the endpoint's, not the handler's.
    let created = operation(&document, "/readings", "post");
    assert_eq!(created["responses"].as_object().unwrap().keys().collect::<Vec<_>>(), ["201"]);

    // An answer with no body states no content at all.
    let forgotten = operation(&document, "/readings", "delete");
    assert_eq!(forgotten["responses"], json!({ "204": { "description": "Forgets every reading." } }));

    // A failing endpoint answers with what its failure can mean, read from the type rather than
    // from a failure this interpretation never holds.
    let read = operation(&document, "/readings/{id}", "get");
    let statuses = read["responses"].as_object().unwrap().keys().cloned().collect::<Vec<_>>();
    assert_eq!(statuses, ["200", "403", "404", "500"]);
    assert_eq!(read["responses"]["404"]["content"]["text/plain"]["schema"], json!({ "type": "string" }));
}

#[test]
fn describes_the_surface_the_other_interpretations_compile() {
    let api = OpenApiHandlerImpl::<Readings>::new();
    let text = TextHandlerImpl;

    let described = text.compile_http(text.readings_api::<Readings>());
    let documented = api.compile_http(api.readings_api::<Readings>());

    assert_eq!(documented.labels(), described.labels());
    assert_eq!(documented.labels(), ["GET /readings/{id}", "GET /readings", "POST /readings", "DELETE /readings",]);
    assert_eq!(documented.operations(), ["reading_at", "reading_search", "reading_record", "reading_forget"]);
}

#[test]
fn says_what_each_operation_is_for() {
    let document = document();

    // A doc comment already reads as a summary and then a description, so that is how it is stated.
    let read = operation(&document, "/readings/{id}", "get");
    assert_eq!(read["summary"], "Returns one identified reading, or why it could not be read.");
    assert!(read["description"].is_null());

    // The answer an endpoint succeeds with is what the operation was documented as. What a failure
    // answers with the program never says in words, so the document does not invent any.
    assert_eq!(read["responses"]["200"]["description"], "Returns one identified reading, or why it could not be read.");
    assert_eq!(read["responses"]["404"]["description"], "");
}

#[test]
fn keys_a_collection_of_names_and_values_by_each_name_it_carries() {
    use alux_http_conformance::{Shop, ShopApiExt};

    let api = OpenApiHandlerImpl::<Shop>::new();
    let route = api.compile_http(api.shop_api::<Shop>());
    let document = api.document("shop", "1.0", &route);

    // A caller states one cookie per member, never one cookie carrying the product.
    assert_eq!(
        document["paths"]["/session"]["get"]["parameters"],
        json!([{
            "name": "session",
            "in": "cookie",
            "required": true,
            "schema": { "type": "string" },
        }])
    );

    // A header states its name the way the wire spells one, which is not how an argument spells it.
    assert_eq!(
        document["paths"]["/agent"]["get"]["parameters"],
        json!([{
            "name": "user-agent",
            "in": "header",
            "required": true,
            "schema": { "type": "string" },
        }])
    );
}

/// What a caller narrows a search by, stated as the query string carries it.
#[derive(Debug, Serialize, Deserialize, Shape)]
pub struct Filters {
    /// The earliest reading to answer with.
    pub since: u64,
    /// How many to answer with at most.
    pub limit: Option<u32>,
}

impl NamedValuesAlg for Filters {}

/// Narrows what the domain answers with.
trait FilteredAlg {
    /// Returns the readings the filters select.
    fn filtered(&self, filters: Filters) -> impl Future<Output = Vec<Reading>> + Send;
}

#[ext(name = FilteredOperationExt, defunc)]
impl<This> This
where
    This: FilteredAlg,
{
    /// Returns every reading the filters select.
    async fn reading_filtered(&self, filters: Filters) -> Vec<Reading> {
        self.filtered(filters).await
    }
}

#[ext(name = FilteredApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg + JsonOutAlg,
{
    /// Declares one endpoint narrowed by a query string read into a product.
    fn filtered_api<Alg>(&self)
    where
        Alg: FilteredAlg,
    {
        self.routes().get("/filtered", self.op(Alg::reading_filtered).query::<Filters>().json())
    }
}

impl FilteredAlg for Readings {
    async fn filtered(&self, _filters: Filters) -> Vec<Reading> {
        Vec::new()
    }
}

#[test]
fn keys_a_query_read_into_a_product_by_each_member_it_carries() {
    let api = OpenApiHandlerImpl::<Readings>::new();
    let route = api.compile_http(api.filtered_api::<Readings>());
    let document = api.document("readings", "1.0", &route);

    // A caller states `?since=…&limit=…`, so that is what the document keys, one parameter each.
    // A member that may be absent is a parameter a caller need not state.
    assert_eq!(
        operation(&document, "/filtered", "get")["parameters"],
        json!([
            { "name": "limit", "in": "query", "required": false, "schema": { "anyOf": [{ "type": "integer", "format": "int32", "minimum": 0 }, { "type": "null" }] } },
            { "name": "since", "in": "query", "required": true, "schema": { "type": "integer", "format": "int64", "minimum": 0 } },
        ])
    );
}

#[test]
fn states_every_header_an_answer_carries() {
    use alux_http_conformance::{Shop, ShopApiExt};

    let api = OpenApiHandlerImpl::<Shop>::new();
    let route = api.compile_http(api.shop_api::<Shop>());
    let document = api.document("shop", "1.0", &route);

    // A header an answer carries is one a caller reads, so a document states it beside the body.
    let cached = &document["paths"]["/cached"]["get"]["responses"]["200"];
    assert_eq!(cached["headers"], json!({ "cache-control": { "schema": { "type": "string" } } }));
    assert!(cached["content"]["application/json"]["schema"].is_object());
}
