//! Renders each output kind as the answer it states.

use crate::{DirectHandlerImpl, DirectResponse};
use alux_http::{
    BytesOutAlg, ChunksAlg, ChunksExt, EmptyOutAlg, FileOutAlg, HeaderNameAlg, HeaderOutAlg, HtmlOutAlg, HttpErrorAlg,
    HttpStatus, JsonOutAlg, OutputAlg, RedirectOutAlg, ResultOutAlg, StatusOutAlg, StreamOutAlg, TextOutAlg,
};
use core::fmt::Display;
use core::marker::PhantomData;
use futures::TryStreamExt;
use serde::Serialize;
use std::io::Error as IoError;

/// Renders a semantic result as a JSON answer.
pub struct DirectJsonOutput;

impl<From> OutputAlg<From> for DirectJsonOutput
where
    From: Serialize,
{
    type Output = DirectResponse;

    fn output(from: From) -> Self::Output {
        match serde_json::to_vec(&from) {
            Ok(body) => DirectResponse::content(HttpStatus::OK, "application/json", body),
            Err(error) => DirectResponse::content(HttpStatus::INTERNAL, "text/plain; charset=utf-8", error.to_string()),
        }
    }
}

/// Renders a semantic result as a plain-text answer.
pub struct DirectTextOutput;

impl<From> OutputAlg<From> for DirectTextOutput
where
    From: Display,
{
    type Output = DirectResponse;

    fn output(from: From) -> Self::Output {
        DirectResponse::content(HttpStatus::OK, "text/plain; charset=utf-8", from.to_string())
    }
}

/// Renders a semantic result as an HTML answer.
pub struct DirectHtmlOutput;

impl<From> OutputAlg<From> for DirectHtmlOutput
where
    From: Display,
{
    type Output = DirectResponse;

    fn output(from: From) -> Self::Output {
        DirectResponse::content(HttpStatus::OK, "text/html; charset=utf-8", from.to_string())
    }
}

/// Renders a semantic result as a raw-byte answer.
pub struct DirectBytesOutput;

impl<From> OutputAlg<From> for DirectBytesOutput
where
    From: Into<Vec<u8>>,
{
    type Output = DirectResponse;

    fn output(from: From) -> Self::Output {
        DirectResponse::content(HttpStatus::OK, "application/octet-stream", from)
    }
}

/// Renders a handler that returns nothing as an answer with no body.
pub struct DirectEmptyOutput;

impl OutputAlg<()> for DirectEmptyOutput {
    type Output = DirectResponse;

    fn output((): ()) -> Self::Output {
        DirectResponse::new(HttpStatus::NO_CONTENT)
    }
}

/// Renders a semantic location as a redirect.
pub struct DirectRedirectOutput;

impl<From> OutputAlg<From> for DirectRedirectOutput
where
    From: Display,
{
    type Output = DirectResponse;

    fn output(from: From) -> Self::Output {
        DirectResponse::new(HttpStatus::SEE_OTHER).with_header("location", &from.to_string())
    }
}

/// Renders a semantic file result as a downloadable answer.
pub struct DirectFileOutput;

impl<File, Error> OutputAlg<(Result<File, Error>, String)> for DirectFileOutput
where
    File: Into<Vec<u8>>,
    Error: HttpErrorAlg,
{
    type Output = DirectResponse;

    fn output((file, name): (Result<File, Error>, String)) -> Self::Output {
        match file {
            Ok(file) => {
                let name = name.replace(['\r', '\n', '"'], "_");
                DirectResponse::content(HttpStatus::OK, "application/octet-stream", file)
                    .with_header("content-disposition", &format!("attachment; filename=\"{name}\""))
            }
            Err(error) => DirectResponse::new(error.http_status()),
        }
    }
}

/// Answers with a body produced over time.
///
/// Nothing is collected here. This interpretation carries no transport, so the answer carries what
/// produces the bytes and whatever moves them drives it.
pub struct DirectStreamOutput;

impl<From> OutputAlg<From> for DirectStreamOutput
where
    From: ChunksAlg + Send + 'static,
    From::Chunk: Into<Vec<u8>> + Send,
    From::Error: Display,
{
    type Output = DirectResponse;

    fn output(from: From) -> Self::Output {
        let moving = from.moving().map_ok(Into::into).map_err(|error| IoError::other(error.to_string()));

        DirectResponse::new(HttpStatus::OK)
            .with_header("content-type", "application/octet-stream")
            .with_chunks(Box::pin(moving))
    }
}

/// Answers with a header the handler stated, beside the body it stated.
pub struct DirectHeaderOutput<Inner, Name>(PhantomData<fn(Inner, Name)>);

impl<Inner, Name, Value, Rest> OutputAlg<(Value, Rest)> for DirectHeaderOutput<Inner, Name>
where
    Inner: OutputAlg<Rest, Output = DirectResponse>,
    Name: HeaderNameAlg,
    Value: Display,
{
    type Output = DirectResponse;

    fn output((value, rest): (Value, Rest)) -> Self::Output {
        Inner::output(rest).with_header(Name::HEADER_NAME, &value.to_string())
    }
}

/// Answers with the status an endpoint declared, around the body it already states.
pub struct DirectStatusOutput<Inner, const CODE: u16>(PhantomData<Inner>);

impl<Inner, From, const CODE: u16> OutputAlg<From> for DirectStatusOutput<Inner, CODE>
where
    Inner: OutputAlg<From, Output = DirectResponse>,
{
    type Output = DirectResponse;

    fn output(from: From) -> Self::Output {
        Inner::output(from).with_status(HttpStatus::new(CODE))
    }
}

/// Answers with what a failure means when the handler failed, and with the body it states otherwise.
pub struct DirectResultOutput<Inner, Error>(PhantomData<fn(Inner, Error)>);

impl<Inner, Error, Value> OutputAlg<Result<Value, Error>> for DirectResultOutput<Inner, Error>
where
    Inner: OutputAlg<Value, Output = DirectResponse>,
    Error: HttpErrorAlg,
{
    type Output = DirectResponse;

    fn output(from: Result<Value, Error>) -> Self::Output {
        match from {
            Ok(value) => Inner::output(value),
            Err(error) => {
                DirectResponse::content(error.http_status(), "text/plain; charset=utf-8", error.http_message())
            }
        }
    }
}

macro_rules! direct_outputs {
    ($($alg:ident => $selected:ident, $output:ty),+ $(,)?) => {
        $(
            impl<Context> $alg for DirectHandlerImpl<Context> {
                type $selected<From> = $output;
            }
        )+
    };
}

direct_outputs! {
    JsonOutAlg     => Json, DirectJsonOutput,
    FileOutAlg     => File, DirectFileOutput,
    TextOutAlg     => Text, DirectTextOutput,
    HtmlOutAlg     => Html, DirectHtmlOutput,
    BytesOutAlg    => Bytes, DirectBytesOutput,
    EmptyOutAlg    => Empty, DirectEmptyOutput,
    RedirectOutAlg => Redirect, DirectRedirectOutput,
    StreamOutAlg   => Stream, DirectStreamOutput,
}

impl<Context> HeaderOutAlg for DirectHandlerImpl<Context> {
    type Header<Inner, Name> = DirectHeaderOutput<Inner, Name>;
}

impl<Context> StatusOutAlg for DirectHandlerImpl<Context> {
    type Status<Inner, const CODE: u16> = DirectStatusOutput<Inner, CODE>;
}

impl<Context> ResultOutAlg for DirectHandlerImpl<Context> {
    type Result<Inner, Error> = DirectResultOutput<Inner, Error>;
}
