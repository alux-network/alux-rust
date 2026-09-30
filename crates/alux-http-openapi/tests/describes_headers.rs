//! Documents a header on every answer the kind inside it states.

use alux_ext::ext;
use alux_http::{ETag, HttpApiAlg, HttpProgramExt, http};
use alux_http_openapi::OpenApiHandlerImpl;
use core::future::Future;
use serde_json::Value;
use std::io::Error as IoError;

/// Reads one reading and the version it is at.
pub trait TagAlg {
    /// Returns the version, and the reading or why it could not be read.
    fn tagged(&self) -> impl Future<Output = (String, Result<u32, IoError>)> + Send;
    /// Returns the reading and its version, or why it could not be read.
    fn found(&self) -> impl Future<Output = Result<(String, u32), IoError>> + Send;
}

/// Derives the operations the headers surface exposes.
#[ext(name = TagOperationExt, defunc)]
pub impl<This> This
where
    This: TagAlg,
{
    /// Returns the version, and the reading or why it could not be read.
    async fn tag_around(&self) -> (String, Result<u32, IoError>) {
        self.tagged().await
    }

    /// Returns the reading and its version, or why it could not be read.
    async fn tag_inside(&self) -> Result<(String, u32), IoError> {
        self.found().await
    }
}

/// Declares one header around a result, and one inside it.
#[ext(name = TagApiExt, defunc(via = http))]
pub impl<This> This
where
    This: HttpApiAlg,
{
    /// Declares both placements of one header.
    fn tag_api<Alg>(&self)
    where
        Alg: TagAlg,
    {
        self.routes()
            .get("/around", self.op(Alg::tag_around).out_header::<ETag>().result().json())
            .get("/inside", self.op(Alg::tag_inside).result().out_header::<ETag>().json())
    }
}

/// The domain the surface is documented against.
struct Tags;

impl TagAlg for Tags {
    async fn tagged(&self) -> (String, Result<u32, IoError>) {
        ("v1".to_owned(), Ok(7))
    }

    async fn found(&self) -> Result<(String, u32), IoError> {
        Ok(("v1".to_owned(), 7))
    }
}

/// Returns whether each answer an operation states carries `etag`, keyed by status.
fn carrying(document: &Value, path: &str) -> Vec<(String, bool)> {
    let responses = document["paths"][path]["get"]["responses"].as_object().unwrap();

    responses.iter().map(|(status, answer)| (status.clone(), answer["headers"]["etag"].is_object())).collect()
}

#[test]
fn documents_a_header_on_every_answer_it_is_written_around() {
    let api = OpenApiHandlerImpl::<Tags>::new();
    let document = api.document("tags", "1.0", &api.compile_http(api.tag_api::<Tags>()));

    // Around the result, the header is on the success and on every failure.
    let around = carrying(&document, "/around");
    assert!(around.len() > 1, "{around:?}");
    assert!(around.iter().all(|(_, carried)| *carried), "{around:?}");

    // Inside the result, only the success carries it.
    let inside = carrying(&document, "/inside");
    assert!(inside.iter().all(|(status, carried)| *carried == status.starts_with('2')), "{inside:?}");
}
