//! Renders each output kind as the response Salvo answers with.

use crate::SalvoHandlerImpl;
use alux_http::{
    BytesOutAlg, ChunksAlg, ChunksExt, EmptyOutAlg, FileOutAlg, HeaderNameAlg, HeaderOutAlg, HtmlOutAlg, HttpErrorAlg,
    HttpStatus, JsonOutAlg, OutputAlg, RedirectOutAlg, ResultOutAlg, StatusOutAlg, StreamOutAlg, TextOutAlg,
};
use core::fmt::Display;
use core::marker::PhantomData;
use futures::{Stream, TryStreamExt};
use salvo::Response;
use salvo::http::ResBody;
use salvo::http::StatusCode;
use salvo::http::header::{self, HeaderName, HeaderValue};
use salvo::hyper::body::Bytes;
use serde::Serialize;
use std::io::Error as IoError;

/// Interprets a portable status as the one Salvo answers with.
pub fn salvo_status(status: HttpStatus) -> StatusCode {
    StatusCode::from_u16(status.code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

/// States a response carrying a status, a content type, and a body.
fn answered(status: HttpStatus, content_type: &str, body: impl Into<Vec<u8>>) -> Response {
    let mut answer = Response::new();
    answer.status_code(salvo_status(status));
    if let Ok(value) = HeaderValue::from_str(content_type) {
        answer.headers_mut().insert(header::CONTENT_TYPE, value);
    }
    answer.body(body.into());

    answer
}

/// Renders a semantic result as a JSON response.
pub struct SalvoJsonOutput;

impl<From> OutputAlg<From> for SalvoJsonOutput
where
    From: Serialize,
{
    type Output = Response;

    fn output(from: From) -> Self::Output {
        match serde_json::to_vec(&from) {
            Ok(body) => answered(HttpStatus::OK, "application/json", body),
            Err(error) => answered(HttpStatus::INTERNAL, "text/plain; charset=utf-8", error.to_string()),
        }
    }
}

macro_rules! salvo_text_outputs {
    ($($output:ident => $content_type:literal, $meaning:literal),+ $(,)?) => {
        $(
            #[doc = concat!("Renders a semantic result as ", $meaning, ".")]
            pub struct $output;

            impl<From> OutputAlg<From> for $output
            where
                From: Display,
            {
                type Output = Response;

                fn output(from: From) -> Self::Output {
                    answered(HttpStatus::OK, $content_type, from.to_string())
                }
            }
        )+
    };
}

salvo_text_outputs! {
    SalvoTextOutput => "text/plain; charset=utf-8", "a plain-text response",
    SalvoHtmlOutput => "text/html; charset=utf-8", "an HTML response",
}

/// Renders a semantic result as a raw-byte response.
pub struct SalvoBytesOutput;

impl<From> OutputAlg<From> for SalvoBytesOutput
where
    From: Into<Vec<u8>>,
{
    type Output = Response;

    fn output(from: From) -> Self::Output {
        answered(HttpStatus::OK, "application/octet-stream", from)
    }
}

/// Renders a handler that returns nothing as a response with no body.
pub struct SalvoEmptyOutput;

impl OutputAlg<()> for SalvoEmptyOutput {
    type Output = Response;

    fn output((): ()) -> Self::Output {
        let mut answer = Response::new();
        answer.status_code(salvo_status(HttpStatus::NO_CONTENT));

        answer
    }
}

/// Renders a semantic location as a redirect.
pub struct SalvoRedirectOutput;

impl<From> OutputAlg<From> for SalvoRedirectOutput
where
    From: Display,
{
    type Output = Response;

    fn output(from: From) -> Self::Output {
        let mut answer = Response::new();
        answer.status_code(salvo_status(HttpStatus::SEE_OTHER));
        if let Ok(value) = HeaderValue::from_str(&from.to_string()) {
            answer.headers_mut().insert(header::LOCATION, value);
        }

        answer
    }
}

/// Renders a semantic file result as a downloadable response.
pub struct SalvoFileOutput;

impl<File, Error> OutputAlg<(Result<File, Error>, String)> for SalvoFileOutput
where
    File: Into<Vec<u8>>,
    Error: HttpErrorAlg,
{
    type Output = Response;

    fn output((file, name): (Result<File, Error>, String)) -> Self::Output {
        match file {
            Ok(file) => {
                let name = name.replace(['\r', '\n', '"'], "_");
                let mut answer = answered(HttpStatus::OK, "application/octet-stream", file);
                if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{name}\"")) {
                    answer.headers_mut().insert(header::CONTENT_DISPOSITION, value);
                }

                answer
            }
            Err(error) => {
                let mut answer = Response::new();
                answer.status_code(salvo_status(error.http_status()));

                answer
            }
        }
    }
}

/// Reads a body stated as chunks as the bytes this framework moves.
fn moving<Chunks>(chunks: Chunks) -> impl Stream<Item = Result<Vec<u8>, IoError>> + Send
where
    Chunks: ChunksAlg + Send + 'static,
    Chunks::Chunk: Into<Vec<u8>> + Send,
    Chunks::Error: Display,
{
    chunks.moving().map_ok(Into::into).map_err(|error| IoError::other(error.to_string()))
}

/// Answers with a body produced over time.
pub struct SalvoStreamOutput;

impl<From> OutputAlg<From> for SalvoStreamOutput
where
    From: ChunksAlg + Send + 'static,
    From::Chunk: Into<Vec<u8>> + Send,
    From::Error: Display,
{
    type Output = Response;

    fn output(from: From) -> Self::Output {
        let moving = moving(from).map_ok(Bytes::from);
        let mut answer = Response::new();
        answer.status_code(salvo_status(HttpStatus::OK));
        if let Ok(value) = HeaderValue::from_str("application/octet-stream") {
            answer.headers_mut().insert(header::CONTENT_TYPE, value);
        }
        answer.body(ResBody::stream(moving));

        answer
    }
}

/// Answers with a header the handler stated, beside the body it stated.
pub struct SalvoHeaderOutput<Inner, Name>(PhantomData<fn(Inner, Name)>);

impl<Inner, Name, Value, Rest> OutputAlg<(Value, Rest)> for SalvoHeaderOutput<Inner, Name>
where
    Inner: OutputAlg<Rest, Output = Response>,
    Name: HeaderNameAlg,
    Value: Display,
{
    type Output = Response;

    fn output((value, rest): (Value, Rest)) -> Self::Output {
        let mut answer = Inner::output(rest);
        if let Ok(value) = HeaderValue::from_str(&value.to_string()) {
            answer.headers_mut().insert(HeaderName::from_static(Name::HEADER_NAME), value);
        }

        answer
    }
}

/// Answers with the status an endpoint declared, around the body it already states.
pub struct SalvoStatusOutput<Inner, const CODE: u16>(PhantomData<Inner>);

impl<Inner, From, const CODE: u16> OutputAlg<From> for SalvoStatusOutput<Inner, CODE>
where
    Inner: OutputAlg<From, Output = Response>,
{
    type Output = Response;

    fn output(from: From) -> Self::Output {
        let mut answer = Inner::output(from);
        answer.status_code(salvo_status(HttpStatus::new(CODE)));

        answer
    }
}

/// Answers with what a failure means when the handler failed, and with the body it states otherwise.
pub struct SalvoResultOutput<Inner, Error>(PhantomData<fn(Inner, Error)>);

impl<Inner, Error, Value> OutputAlg<Result<Value, Error>> for SalvoResultOutput<Inner, Error>
where
    Inner: OutputAlg<Value, Output = Response>,
    Error: HttpErrorAlg,
{
    type Output = Response;

    fn output(from: Result<Value, Error>) -> Self::Output {
        match from {
            Ok(value) => Inner::output(value),
            Err(error) => answered(error.http_status(), "text/plain; charset=utf-8", error.http_message()),
        }
    }
}

macro_rules! salvo_outputs {
    ($($alg:ident => $selected:ident, $output:ty),+ $(,)?) => {
        $(
            impl<Context> $alg for SalvoHandlerImpl<Context> {
                type $selected<From> = $output;
            }
        )+
    };
}

salvo_outputs! {
    JsonOutAlg     => Json, SalvoJsonOutput,
    FileOutAlg     => File, SalvoFileOutput,
    TextOutAlg     => Text, SalvoTextOutput,
    HtmlOutAlg     => Html, SalvoHtmlOutput,
    BytesOutAlg    => Bytes, SalvoBytesOutput,
    EmptyOutAlg    => Empty, SalvoEmptyOutput,
    RedirectOutAlg => Redirect, SalvoRedirectOutput,
    StreamOutAlg   => Stream, SalvoStreamOutput,
}

impl<Context> HeaderOutAlg for SalvoHandlerImpl<Context> {
    type Header<Inner, Name> = SalvoHeaderOutput<Inner, Name>;
}

impl<Context> StatusOutAlg for SalvoHandlerImpl<Context> {
    type Status<Inner, const CODE: u16> = SalvoStatusOutput<Inner, CODE>;
}

impl<Context> ResultOutAlg for SalvoHandlerImpl<Context> {
    type Result<Inner, Error> = SalvoResultOutput<Inner, Error>;
}
