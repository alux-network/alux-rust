//! Renders each output kind as the response Rocket answers with.

use crate::RocketHandlerImpl;
use alux_http::{
    BytesOutAlg, ChunksAlg, ChunksExt, EmptyOutAlg, FileOutAlg, HeaderNameAlg, HeaderOutAlg, HtmlOutAlg, HttpErrorAlg,
    HttpStatus, JsonOutAlg, OutputAlg, RedirectOutAlg, ResultOutAlg, StatusOutAlg, StreamOutAlg, TextOutAlg,
};
use core::fmt::{self, Debug, Display};
use core::marker::PhantomData;
use core::pin::Pin;
use futures::{Stream, TryStreamExt};
use serde::Serialize;
use std::io::Error as IoError;

/// What one endpoint answers with, before Rocket builds a response from it.
///
/// Rocket's own response borrows the request it answers, so what an output kind states here is the
/// answer itself and the endpoint hands it over.
#[derive(Debug)]
pub struct RocketAnswer {
    pub(crate) status: HttpStatus,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: RocketBody,
}

/// What an answer carries, which is either bytes already in hand or bytes still to come.
pub enum RocketBody {
    /// Bytes that are already there.
    Stated(Vec<u8>),
    /// Bytes produced over time, and what produces them.
    Produced(Chunks),
}

impl Debug for RocketBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stated(body) => formatter.debug_tuple("Stated").field(body).finish(),
            Self::Produced(_) => formatter.write_str("Produced(..)"),
        }
    }
}

/// What produces a body over time, once this interpretation has chosen how to carry it.
pub type Chunks = Pin<Box<dyn Stream<Item = Result<Vec<u8>, IoError>> + Send>>;

impl RocketAnswer {
    /// States an answer carrying a status and nothing else.
    pub fn new(status: HttpStatus) -> Self {
        Self { status, headers: Vec::new(), body: RocketBody::Stated(Vec::new()) }
    }

    /// States an answer carrying a status, a content type, and a body.
    pub fn content(status: HttpStatus, content_type: &str, body: impl Into<Vec<u8>>) -> Self {
        Self::new(status).with_header("content-type", content_type).with_body(body)
    }

    /// States one header on this answer.
    #[must_use]
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }

    /// States the body of this answer.
    #[must_use]
    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = RocketBody::Stated(body.into());
        self
    }

    /// States a body this answer produces over time.
    #[must_use]
    pub fn with_chunks(mut self, chunks: Chunks) -> Self {
        self.body = RocketBody::Produced(chunks);
        self
    }

    /// Answers the same thing under a different status.
    #[must_use]
    pub fn with_status(mut self, status: HttpStatus) -> Self {
        self.status = status;
        self
    }
}

/// Renders a semantic result as a JSON answer.
pub struct RocketJsonOutput;

impl<From> OutputAlg<From> for RocketJsonOutput
where
    From: Serialize,
{
    type Output = RocketAnswer;

    fn output(from: From) -> Self::Output {
        match serde_json::to_vec(&from) {
            Ok(body) => RocketAnswer::content(HttpStatus::OK, "application/json", body),
            Err(error) => RocketAnswer::content(HttpStatus::INTERNAL, "text/plain; charset=utf-8", error.to_string()),
        }
    }
}

macro_rules! rocket_text_outputs {
    ($($output:ident => $content_type:literal, $meaning:literal),+ $(,)?) => {
        $(
            #[doc = concat!("Renders a semantic result as ", $meaning, ".")]
            pub struct $output;

            impl<From> OutputAlg<From> for $output
            where
                From: Display,
            {
                type Output = RocketAnswer;

                fn output(from: From) -> Self::Output {
                    RocketAnswer::content(HttpStatus::OK, $content_type, from.to_string())
                }
            }
        )+
    };
}

rocket_text_outputs! {
    RocketTextOutput => "text/plain; charset=utf-8", "a plain-text answer",
    RocketHtmlOutput => "text/html; charset=utf-8", "an HTML answer",
}

/// Renders a semantic result as a raw-byte answer.
pub struct RocketBytesOutput;

impl<From> OutputAlg<From> for RocketBytesOutput
where
    From: Into<Vec<u8>>,
{
    type Output = RocketAnswer;

    fn output(from: From) -> Self::Output {
        RocketAnswer::content(HttpStatus::OK, "application/octet-stream", from)
    }
}

/// Renders a handler that returns nothing as an answer with no body.
pub struct RocketEmptyOutput;

impl OutputAlg<()> for RocketEmptyOutput {
    type Output = RocketAnswer;

    fn output((): ()) -> Self::Output {
        RocketAnswer::new(HttpStatus::NO_CONTENT)
    }
}

/// Renders a semantic location as a redirect.
pub struct RocketRedirectOutput;

impl<From> OutputAlg<From> for RocketRedirectOutput
where
    From: Display,
{
    type Output = RocketAnswer;

    fn output(from: From) -> Self::Output {
        RocketAnswer::new(HttpStatus::SEE_OTHER).with_header("location", &from.to_string())
    }
}

/// Renders a semantic file result as a downloadable answer.
pub struct RocketFileOutput;

impl<File, Error> OutputAlg<(Result<File, Error>, String)> for RocketFileOutput
where
    File: Into<Vec<u8>>,
    Error: HttpErrorAlg,
{
    type Output = RocketAnswer;

    fn output((file, name): (Result<File, Error>, String)) -> Self::Output {
        match file {
            Ok(file) => {
                let name = name.replace(['\r', '\n', '"'], "_");
                RocketAnswer::content(HttpStatus::OK, "application/octet-stream", file)
                    .with_header("content-disposition", &format!("attachment; filename=\"{name}\""))
            }
            Err(error) => RocketAnswer::new(error.http_status()),
        }
    }
}

/// Answers with a body produced over time.
pub struct RocketStreamOutput;

impl<From> OutputAlg<From> for RocketStreamOutput
where
    From: ChunksAlg + Send + 'static,
    From::Chunk: Into<Vec<u8>> + Send,
    From::Error: Display,
{
    type Output = RocketAnswer;

    fn output(from: From) -> Self::Output {
        let moving = from.moving().map_ok(Into::into).map_err(|error| IoError::other(error.to_string()));

        RocketAnswer::new(HttpStatus::OK)
            .with_header("content-type", "application/octet-stream")
            .with_chunks(Box::pin(moving))
    }
}

/// Answers with a header the handler stated, beside the body it stated.
pub struct RocketHeaderOutput<Inner, Name>(PhantomData<fn(Inner, Name)>);

impl<Inner, Name, Value, Rest> OutputAlg<(Value, Rest)> for RocketHeaderOutput<Inner, Name>
where
    Inner: OutputAlg<Rest, Output = RocketAnswer>,
    Name: HeaderNameAlg,
    Value: Display,
{
    type Output = RocketAnswer;

    fn output((value, rest): (Value, Rest)) -> Self::Output {
        Inner::output(rest).with_header(Name::HEADER_NAME, &value.to_string())
    }
}

/// Answers with the status an endpoint declared, around the body it already states.
pub struct RocketStatusOutput<Inner, const CODE: u16>(PhantomData<Inner>);

impl<Inner, From, const CODE: u16> OutputAlg<From> for RocketStatusOutput<Inner, CODE>
where
    Inner: OutputAlg<From, Output = RocketAnswer>,
{
    type Output = RocketAnswer;

    fn output(from: From) -> Self::Output {
        Inner::output(from).with_status(HttpStatus::new(CODE))
    }
}

/// Answers with what a failure means when the handler failed, and with the body it states otherwise.
pub struct RocketResultOutput<Inner, Error>(PhantomData<fn(Inner, Error)>);

impl<Inner, Error, Value> OutputAlg<Result<Value, Error>> for RocketResultOutput<Inner, Error>
where
    Inner: OutputAlg<Value, Output = RocketAnswer>,
    Error: HttpErrorAlg,
{
    type Output = RocketAnswer;

    fn output(from: Result<Value, Error>) -> Self::Output {
        match from {
            Ok(value) => Inner::output(value),
            Err(error) => RocketAnswer::content(error.http_status(), "text/plain; charset=utf-8", error.http_message()),
        }
    }
}

macro_rules! rocket_outputs {
    ($($alg:ident => $selected:ident, $output:ty),+ $(,)?) => {
        $(
            impl<Context> $alg for RocketHandlerImpl<Context> {
                type $selected<From> = $output;
            }
        )+
    };
}

rocket_outputs! {
    JsonOutAlg     => Json, RocketJsonOutput,
    FileOutAlg     => File, RocketFileOutput,
    TextOutAlg     => Text, RocketTextOutput,
    HtmlOutAlg     => Html, RocketHtmlOutput,
    BytesOutAlg    => Bytes, RocketBytesOutput,
    EmptyOutAlg    => Empty, RocketEmptyOutput,
    RedirectOutAlg => Redirect, RocketRedirectOutput,
    StreamOutAlg   => Stream, RocketStreamOutput,
}

impl<Context> HeaderOutAlg for RocketHandlerImpl<Context> {
    type Header<Inner, Name> = RocketHeaderOutput<Inner, Name>;
}

impl<Context> StatusOutAlg for RocketHandlerImpl<Context> {
    type Status<Inner, const CODE: u16> = RocketStatusOutput<Inner, CODE>;
}

impl<Context> ResultOutAlg for RocketHandlerImpl<Context> {
    type Result<Inner, Error> = RocketResultOutput<Inner, Error>;
}
