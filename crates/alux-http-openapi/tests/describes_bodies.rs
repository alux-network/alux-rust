//! Documents the endpoints whose body is bytes rather than a shape.
//!
//! A streamed body and a downloaded file are produced rather than described: what reaches a caller
//! is `application/octet-stream`, and what a handler answered with is the interpretation's own
//! business. What this holds is that a document says so, and that a download also states what
//! reading its file can fail as.

use alux_ext::ext;
use alux_http::{FileOutAlg, HttpApiAlg, HttpErrorAlg, HttpProgramExt, HttpStatus, http};
use alux_http_conformance::{Shop, StreamApiExt};
use alux_http_openapi::OpenApiHandlerImpl;
use core::future::Future;

/// The bytes a caller receives, however they were produced.
const BYTES: &str = "application/octet-stream";

/// Reads the file a download answers with.
pub trait DownloadAlg {
    /// The file contents a download denotes.
    type File;
    /// The reason a file cannot be read.
    type Error;

    /// Reads the file.
    fn dl_file(&self) -> impl Future<Output = Result<Self::File, Self::Error>> + Send;
}

/// Derives the download an API exposes.
#[ext(name = DownloadOperationExt, defunc)]
pub impl<This> This
where
    This: DownloadAlg,
{
    /// Returns the file to send and the name to offer it under.
    async fn download_current(&self) -> (Result<This::File, This::Error>, String) {
        (self.dl_file().await, "reading.txt".to_owned())
    }
}

/// Declares the download, which every interpretation carrying a file compiles.
#[ext(name = DownloadApiExt, defunc(via = http))]
pub impl<This> This
where
    This: HttpApiAlg + FileOutAlg,
{
    /// Declares one endpoint answering with a file.
    fn download_api<Alg>(&self)
    where
        Alg: DownloadAlg,
    {
        self.routes().get("/download", self.op(Alg::download_current).file())
    }
}

/// States that the file could not be read.
#[derive(Debug)]
pub struct Unreadable;

impl HttpErrorAlg for Unreadable {
    const HTTP_STATUSES: &'static [HttpStatus] = &[HttpStatus::NOT_FOUND];

    fn http_status(&self) -> HttpStatus {
        HttpStatus::NOT_FOUND
    }

    fn http_message(&self) -> String {
        "no such file".to_owned()
    }
}

impl DownloadAlg for Shop {
    type File = Vec<u8>;
    type Error = Unreadable;

    async fn dl_file(&self) -> Result<Self::File, Self::Error> {
        Ok(Vec::new())
    }
}

#[test]
fn documents_a_streamed_body_as_bytes() {
    let api = OpenApiHandlerImpl::<Shop>::new();
    let route = api.compile_http(api.stream_api::<Shop>());

    let document = api.document("shop", "1.0", &route);
    let answered = &document["paths"]["/ticks"]["get"]["responses"]["200"]["content"][BYTES]["schema"];
    assert_eq!(answered["type"], "string");
    assert_eq!(answered["format"], "binary");
}

#[test]
fn documents_a_download_as_bytes_and_what_reading_it_fails_as() {
    let api = OpenApiHandlerImpl::<Shop>::new();
    let route = api.compile_http(api.download_api::<Shop>());

    let document = api.document("shop", "1.0", &route);
    let answers = &document["paths"]["/download"]["get"]["responses"];
    assert_eq!(answers["200"]["content"][BYTES]["schema"]["format"], "binary");
    // What the file failed to be read as is stated by the failure, not by the endpoint.
    assert_eq!(answers["404"]["content"]["text/plain"]["schema"]["type"], "string");
}
