//! States a request and an answer without naming a transport.

use alux_http::{HttpErrorAlg, HttpStatus};
use core::fmt::{self, Debug, Display};
use core::pin::Pin;
use futures::{Stream, StreamExt};
use std::io::Error as IoError;

/// What a caller sent, stated without a transport.
///
/// Whatever moves bytes decides how a request arrives; this is only what arrived.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DirectRequest {
    method: Option<alux_http::HttpMethod>,
    path: String,
    query: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl DirectRequest {
    /// States a request for one path under one method.
    pub fn new(method: alux_http::HttpMethod, path: &str) -> Self {
        Self { method: Some(method), path: path.to_owned(), ..Self::default() }
    }

    /// States the query string the caller sent, without its leading separator.
    #[must_use]
    pub fn with_query(mut self, query: &str) -> Self {
        query.trim_start_matches('?').clone_into(&mut self.query);
        self
    }

    /// States one header the caller sent.
    #[must_use]
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_lowercase(), value.to_owned()));
        self
    }

    /// States the body the caller sent.
    #[must_use]
    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Returns the method this request was sent under.
    pub fn method(&self) -> Option<alux_http::HttpMethod> {
        self.method
    }

    /// Returns the path this request asks for.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the query string this request carries.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Returns the value of one header, matched without regard to case.
    pub fn header(&self, name: &str) -> Option<&str> {
        let name = name.to_lowercase();

        self.headers.iter().find(|(header, _)| *header == name).map(|(_, value)| value.as_str())
    }

    /// Returns every header this request carries, in the order it states them.
    pub fn headers(&self) -> impl Iterator<Item = (&str, &str)> {
        self.headers.iter().map(|(name, value)| (name.as_str(), value.as_str()))
    }

    /// Returns the body this request carries.
    pub fn body(&self) -> &[u8] {
        &self.body
    }
}

/// What an answer carries, which is either bytes already in hand or bytes still to come.
///
/// This interpretation carries no transport, so a body produced over time is carried as what
/// produces it. Whatever moves bytes drives it, and anything reading the answer whole collects it.
pub enum DirectBody {
    /// Bytes that are already there.
    Stated(Vec<u8>),
    /// Bytes produced over time, and what produces them.
    Produced(Chunks),
}

impl Debug for DirectBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stated(body) => formatter.debug_tuple("Stated").field(body).finish(),
            Self::Produced(_) => formatter.write_str("Produced(..)"),
        }
    }
}

impl Default for DirectBody {
    fn default() -> Self {
        Self::Stated(Vec::new())
    }
}

/// What produces a body over time, once an interpretation has chosen how to carry it.
pub type Chunks = Pin<Box<dyn Stream<Item = Result<Vec<u8>, IoError>> + Send>>;

/// What a surface answered, stated without a transport.
#[derive(Debug)]
pub struct DirectResponse {
    status: HttpStatus,
    headers: Vec<(String, String)>,
    body: DirectBody,
}

impl DirectResponse {
    /// Answers with a status and nothing else.
    pub fn new(status: HttpStatus) -> Self {
        Self { status, headers: Vec::new(), body: DirectBody::default() }
    }

    /// Answers with a status, a content type, and a body.
    pub fn content(status: HttpStatus, content_type: &str, body: impl Into<Vec<u8>>) -> Self {
        Self::new(status).with_header("content-type", content_type).with_body(body)
    }

    /// States one header on this answer.
    #[must_use]
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_lowercase(), value.to_owned()));
        self
    }

    /// States the body of this answer.
    #[must_use]
    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = DirectBody::Stated(body.into());
        self
    }

    /// States a body this answer produces over time.
    #[must_use]
    pub fn with_chunks(mut self, chunks: Chunks) -> Self {
        self.body = DirectBody::Produced(chunks);
        self
    }

    /// Returns what this answer carries, which is either bytes or what produces them.
    pub fn into_body(self) -> DirectBody {
        self.body
    }

    /// Answers the same thing with every chunk of its body taken.
    ///
    /// A body already in hand is already collected. One produced over time is read to its end, which
    /// is what anything reading an answer whole has to do.
    pub async fn collected(mut self) -> Self {
        let DirectBody::Produced(mut chunks) = self.body else {
            return self;
        };
        let mut collected = Vec::new();
        while let Some(chunk) = chunks.next().await {
            match chunk {
                Ok(chunk) => collected.extend(chunk),
                Err(error) => {
                    self.body = DirectBody::Stated(error.to_string().into_bytes());

                    return self.with_status(HttpStatus::INTERNAL);
                }
            }
        }
        self.body = DirectBody::Stated(collected);

        self
    }

    /// Answers the same thing under a different status.
    #[must_use]
    pub fn with_status(mut self, status: HttpStatus) -> Self {
        self.status = status;
        self
    }

    /// Returns the status this answer carries.
    pub fn status(&self) -> HttpStatus {
        self.status
    }

    /// Returns the value of one header, matched without regard to case.
    pub fn header(&self, name: &str) -> Option<&str> {
        let name = name.to_lowercase();

        self.headers.iter().find(|(header, _)| *header == name).map(|(_, value)| value.as_str())
    }

    /// Returns every header this answer carries, in the order it states them.
    pub fn headers(&self) -> impl Iterator<Item = (&str, &str)> {
        self.headers.iter().map(|(name, value)| (name.as_str(), value.as_str()))
    }

    /// Returns the bytes this answer already carries, which a body still to come has none of.
    pub fn body(&self) -> &[u8] {
        match &self.body {
            DirectBody::Stated(body) => body,
            DirectBody::Produced(_) => &[],
        }
    }

    /// Returns the body read as text, however it was encoded.
    pub fn text(&self) -> String {
        String::from_utf8_lossy(self.body()).into_owned()
    }
}

/// What the interpretation itself answers when no handler can be reached.
///
/// A domain states its own failures; these are the ones routing and reading a request produce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectError {
    status: HttpStatus,
    message: String,
}

impl DirectError {
    /// States a failure with the status it is answered with.
    pub fn new(status: HttpStatus, message: impl Into<String>) -> Self {
        Self { status, message: message.into() }
    }

    /// States that nothing is declared at a path.
    pub fn not_found(path: &str) -> Self {
        Self::new(HttpStatus::NOT_FOUND, format!("nothing answers at `{path}`"))
    }

    /// States that something is declared at a path, but not under this method.
    pub fn method_not_allowed(path: &str) -> Self {
        Self::new(HttpStatus::METHOD_NOT_ALLOWED, format!("`{path}` does not answer this method"))
    }

    /// States that an argument could not be read from where its role says it comes from.
    pub fn unreadable(role: &str, reason: &str) -> Self {
        Self::new(HttpStatus::BAD_REQUEST, format!("the {role} could not be read: {reason}"))
    }
}

impl HttpErrorAlg for DirectError {
    // Routing and reading a request state these; a domain failure carried here states its own.
    const HTTP_STATUSES: &'static [HttpStatus] =
        &[HttpStatus::BAD_REQUEST, HttpStatus::NOT_FOUND, HttpStatus::METHOD_NOT_ALLOWED];

    fn http_status(&self) -> HttpStatus {
        self.status
    }

    fn http_message(&self) -> String {
        self.message.clone()
    }
}

impl Display for DirectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({})", self.message, self.status.code())
    }
}

impl core::error::Error for DirectError {}

impl From<DirectError> for DirectResponse {
    fn from(error: DirectError) -> Self {
        Self::content(error.status, "text/plain; charset=utf-8", error.message)
    }
}
