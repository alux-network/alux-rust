//! Renders each output kind as the reply warp answers with.

use crate::WarpHandlerImpl;
use alux_http::{
    BytesOutAlg, EmptyOutAlg, FileOutAlg, HeaderNameAlg, HeaderOutAlg, HtmlOutAlg, HttpErrorAlg, HttpStatus,
    JsonOutAlg, OutputAlg, RedirectOutAlg, ResultOutAlg, StatusOutAlg, TextOutAlg,
};
use core::fmt::Display;
use core::marker::PhantomData;
use serde::Serialize;
use warp::http::{HeaderName, HeaderValue, StatusCode, header};
use warp::reply::Response;

/// Interprets a portable status as the one warp answers with.
pub fn warp_status(status: HttpStatus) -> StatusCode {
    StatusCode::from_u16(status.code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

/// States a reply carrying a status, a content type, and a body.
fn answered(status: HttpStatus, content_type: &str, body: impl Into<Vec<u8>>) -> Response {
    warp::http::Response::builder()
        .status(warp_status(status))
        .header(header::CONTENT_TYPE, content_type)
        .body(body.into().into())
        .unwrap_or_else(|_| warp::http::Response::new(Vec::new().into()))
}

/// Renders a semantic result as a JSON reply.
pub struct WarpJsonOutput;

impl<From> OutputAlg<From> for WarpJsonOutput
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

macro_rules! warp_text_outputs {
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

warp_text_outputs! {
    WarpTextOutput => "text/plain; charset=utf-8", "a plain-text reply",
    WarpHtmlOutput => "text/html; charset=utf-8", "an HTML reply",
}

/// Renders a semantic result as a raw-byte reply.
pub struct WarpBytesOutput;

impl<From> OutputAlg<From> for WarpBytesOutput
where
    From: Into<Vec<u8>>,
{
    type Output = Response;

    fn output(from: From) -> Self::Output {
        answered(HttpStatus::OK, "application/octet-stream", from)
    }
}

/// Renders a handler that returns nothing as a reply with no body.
pub struct WarpEmptyOutput;

impl OutputAlg<()> for WarpEmptyOutput {
    type Output = Response;

    fn output((): ()) -> Self::Output {
        warp::http::Response::builder()
            .status(warp_status(HttpStatus::NO_CONTENT))
            .body(Vec::new().into())
            .unwrap_or_else(|_| warp::http::Response::new(Vec::new().into()))
    }
}

/// Renders a semantic location as a redirect.
pub struct WarpRedirectOutput;

impl<From> OutputAlg<From> for WarpRedirectOutput
where
    From: Display,
{
    type Output = Response;

    fn output(from: From) -> Self::Output {
        warp::http::Response::builder()
            .status(warp_status(HttpStatus::SEE_OTHER))
            .header(header::LOCATION, from.to_string())
            .body(Vec::new().into())
            .unwrap_or_else(|_| warp::http::Response::new(Vec::new().into()))
    }
}

/// Renders a semantic file result as a downloadable reply.
pub struct WarpFileOutput;

impl<File, Error> OutputAlg<(Result<File, Error>, String)> for WarpFileOutput
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
                if let Ok(value) = format!("attachment; filename=\"{name}\"").parse() {
                    answer.headers_mut().insert(header::CONTENT_DISPOSITION, value);
                }

                answer
            }
            Err(error) => {
                let mut answer = warp::http::Response::new(Vec::new().into());
                *answer.status_mut() = warp_status(error.http_status());

                answer
            }
        }
    }
}

/// Answers with a header the handler stated, beside the body it stated.
pub struct WarpHeaderOutput<Inner, Name>(PhantomData<fn(Inner, Name)>);

impl<Inner, Name, Value, Rest> OutputAlg<(Value, Rest)> for WarpHeaderOutput<Inner, Name>
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
pub struct WarpStatusOutput<Inner, const CODE: u16>(PhantomData<Inner>);

impl<Inner, From, const CODE: u16> OutputAlg<From> for WarpStatusOutput<Inner, CODE>
where
    Inner: OutputAlg<From, Output = Response>,
{
    type Output = Response;

    fn output(from: From) -> Self::Output {
        let mut answer = Inner::output(from);
        *answer.status_mut() = warp_status(HttpStatus::new(CODE));

        answer
    }
}

/// Answers with what a failure means when the handler failed, and with the body it states otherwise.
pub struct WarpResultOutput<Inner, Error>(PhantomData<fn(Inner, Error)>);

impl<Inner, Error, Value> OutputAlg<Result<Value, Error>> for WarpResultOutput<Inner, Error>
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

macro_rules! warp_outputs {
    ($($alg:ident => $selected:ident, $output:ty),+ $(,)?) => {
        $(
            impl<Context> $alg for WarpHandlerImpl<Context> {
                type $selected<From> = $output;
            }
        )+
    };
}

warp_outputs! {
    JsonOutAlg     => Json, WarpJsonOutput,
    FileOutAlg     => File, WarpFileOutput,
    TextOutAlg     => Text, WarpTextOutput,
    HtmlOutAlg     => Html, WarpHtmlOutput,
    BytesOutAlg    => Bytes, WarpBytesOutput,
    EmptyOutAlg    => Empty, WarpEmptyOutput,
    RedirectOutAlg => Redirect, WarpRedirectOutput,
}

impl<Context> HeaderOutAlg for WarpHandlerImpl<Context> {
    type Header<Inner, Name> = WarpHeaderOutput<Inner, Name>;
}

impl<Context> StatusOutAlg for WarpHandlerImpl<Context> {
    type Status<Inner, const CODE: u16> = WarpStatusOutput<Inner, CODE>;
}

impl<Context> ResultOutAlg for WarpHandlerImpl<Context> {
    type Result<Inner, Error> = WarpResultOutput<Inner, Error>;
}
