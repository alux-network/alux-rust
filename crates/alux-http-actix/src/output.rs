//! Renders each output kind as the response actix-web answers with.

use crate::ActixHandlerImpl;
use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use actix_web::http::header::{self, HeaderName, HeaderValue};
use actix_web::web::Bytes;
use alux_http::{
    BytesOutAlg, ChunksAlg, ChunksExt, EmptyOutAlg, FileOutAlg, HeaderNameAlg, HeaderOutAlg, HtmlOutAlg, HttpErrorAlg,
    HttpStatus, JsonOutAlg, OutputAlg, RedirectOutAlg, ResultOutAlg, StatusOutAlg, StreamOutAlg, TextOutAlg,
};
use core::fmt::Display;
use core::marker::PhantomData;
use futures::{Stream, TryStreamExt};
use serde::Serialize;
use std::io::Error as IoError;

/// Interprets a portable status as the one actix-web answers with.
pub fn actix_status(status: HttpStatus) -> StatusCode {
    StatusCode::from_u16(status.code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Renders a semantic result as a JSON response.
pub struct ActixJsonOutput;

impl<From> OutputAlg<From> for ActixJsonOutput
where
    From: Serialize,
{
    type Output = HttpResponse;

    fn output(from: From) -> Self::Output {
        HttpResponse::Ok().json(from)
    }
}

macro_rules! actix_text_outputs {
    ($($output:ident => $content_type:literal, $meaning:literal),+ $(,)?) => {
        $(
            #[doc = concat!("Renders a semantic result as ", $meaning, ".")]
            pub struct $output;

            impl<From> OutputAlg<From> for $output
            where
                From: Display,
            {
                type Output = HttpResponse;

                fn output(from: From) -> Self::Output {
                    HttpResponse::Ok().content_type($content_type).body(from.to_string())
                }
            }
        )+
    };
}

actix_text_outputs! {
    ActixTextOutput => "text/plain; charset=utf-8", "a plain-text response",
    ActixHtmlOutput => "text/html; charset=utf-8", "an HTML response",
}

/// Renders a semantic result as a raw-byte response.
pub struct ActixBytesOutput;

impl<From> OutputAlg<From> for ActixBytesOutput
where
    From: Into<Vec<u8>>,
{
    type Output = HttpResponse;

    fn output(from: From) -> Self::Output {
        HttpResponse::Ok().content_type("application/octet-stream").body(from.into())
    }
}

/// Renders a handler that returns nothing as a response with no body.
pub struct ActixEmptyOutput;

impl OutputAlg<()> for ActixEmptyOutput {
    type Output = HttpResponse;

    fn output((): ()) -> Self::Output {
        HttpResponse::NoContent().finish()
    }
}

/// Renders a semantic location as a redirect.
pub struct ActixRedirectOutput;

impl<From> OutputAlg<From> for ActixRedirectOutput
where
    From: Display,
{
    type Output = HttpResponse;

    fn output(from: From) -> Self::Output {
        let mut answer = HttpResponse::SeeOther().finish();
        if let Ok(value) = HeaderValue::from_str(&from.to_string()) {
            answer.headers_mut().insert(header::LOCATION, value);
        }

        answer
    }
}

/// Renders a semantic file result as a downloadable response.
pub struct ActixFileOutput;

impl<File, Error> OutputAlg<(Result<File, Error>, String)> for ActixFileOutput
where
    File: Into<Vec<u8>>,
    Error: HttpErrorAlg,
{
    type Output = HttpResponse;

    fn output((file, name): (Result<File, Error>, String)) -> Self::Output {
        match file {
            Ok(file) => {
                let name = name.replace(['\r', '\n', '"'], "_");
                HttpResponse::Ok()
                    .content_type("application/octet-stream")
                    .insert_header((header::CONTENT_DISPOSITION, format!("attachment; filename=\"{name}\"")))
                    .body(file.into())
            }
            Err(error) => HttpResponse::new(actix_status(error.http_status())),
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
pub struct ActixStreamOutput;

impl<From> OutputAlg<From> for ActixStreamOutput
where
    From: ChunksAlg + Send + 'static,
    From::Chunk: Into<Vec<u8>> + Send,
    From::Error: Display,
{
    type Output = HttpResponse;

    fn output(from: From) -> Self::Output {
        let moving = moving(from).map_ok(Bytes::from);

        HttpResponse::Ok().content_type("application/octet-stream").streaming(moving)
    }
}

/// Answers with a header the handler stated, beside the body it stated.
pub struct ActixHeaderOutput<Inner, Name>(PhantomData<fn(Inner, Name)>);

impl<Inner, Name, Value, Rest> OutputAlg<(Value, Rest)> for ActixHeaderOutput<Inner, Name>
where
    Inner: OutputAlg<Rest, Output = HttpResponse>,
    Name: HeaderNameAlg,
    Value: Display,
{
    type Output = HttpResponse;

    fn output((value, rest): (Value, Rest)) -> Self::Output {
        let mut answer = Inner::output(rest);
        if let Ok(value) = HeaderValue::from_str(&value.to_string()) {
            answer.headers_mut().insert(HeaderName::from_static(Name::HEADER_NAME), value);
        }

        answer
    }
}

/// Answers with the status an endpoint declared, around the body it already states.
pub struct ActixStatusOutput<Inner, const CODE: u16>(PhantomData<Inner>);

impl<Inner, From, const CODE: u16> OutputAlg<From> for ActixStatusOutput<Inner, CODE>
where
    Inner: OutputAlg<From, Output = HttpResponse>,
{
    type Output = HttpResponse;

    fn output(from: From) -> Self::Output {
        let mut answer = Inner::output(from);
        *answer.status_mut() = actix_status(HttpStatus::new(CODE));

        answer
    }
}

/// Answers with what a failure means when the handler failed, and with the body it states otherwise.
pub struct ActixResultOutput<Inner, Error>(PhantomData<fn(Inner, Error)>);

impl<Inner, Error, Value> OutputAlg<Result<Value, Error>> for ActixResultOutput<Inner, Error>
where
    Inner: OutputAlg<Value, Output = HttpResponse>,
    Error: HttpErrorAlg,
{
    type Output = HttpResponse;

    fn output(from: Result<Value, Error>) -> Self::Output {
        match from {
            Ok(value) => Inner::output(value),
            Err(error) => HttpResponse::build(actix_status(error.http_status()))
                .content_type("text/plain; charset=utf-8")
                .body(error.http_message()),
        }
    }
}

macro_rules! actix_outputs {
    ($($alg:ident => $selected:ident, $output:ty),+ $(,)?) => {
        $(
            impl<Context> $alg for ActixHandlerImpl<Context> {
                type $selected<From> = $output;
            }
        )+
    };
}

actix_outputs! {
    JsonOutAlg     => Json, ActixJsonOutput,
    FileOutAlg     => File, ActixFileOutput,
    TextOutAlg     => Text, ActixTextOutput,
    HtmlOutAlg     => Html, ActixHtmlOutput,
    BytesOutAlg    => Bytes, ActixBytesOutput,
    EmptyOutAlg    => Empty, ActixEmptyOutput,
    RedirectOutAlg => Redirect, ActixRedirectOutput,
    StreamOutAlg   => Stream, ActixStreamOutput,
}

impl<Context> HeaderOutAlg for ActixHandlerImpl<Context> {
    type Header<Inner, Name> = ActixHeaderOutput<Inner, Name>;
}

impl<Context> StatusOutAlg for ActixHandlerImpl<Context> {
    type Status<Inner, const CODE: u16> = ActixStatusOutput<Inner, CODE>;
}

impl<Context> ResultOutAlg for ActixHandlerImpl<Context> {
    type Result<Inner, Error> = ActixResultOutput<Inner, Error>;
}
