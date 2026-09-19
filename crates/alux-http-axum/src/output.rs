use crate::AxumHandlerImpl;
use alux_http::{
    BytesOutAlg, ChunksAlg, ChunksExt, EmptyOutAlg, FileOutAlg, HeaderNameAlg, HeaderOutAlg, HtmlOutAlg, HttpErrorAlg,
    HttpStatus, JsonOutAlg, OutputAlg, RedirectOutAlg, ResultOutAlg, StatusOutAlg, StreamOutAlg, TextOutAlg,
};
use axum::Json;
use axum::body::Body;
use axum::http::{HeaderName, HeaderValue, StatusCode, header};
use axum::response::{Html, IntoResponse, Redirect, Response};
use core::fmt::Display;
use core::marker::PhantomData;
use futures::{Stream, TryStreamExt};
use std::io::Error as IoError;

/// Interprets a portable status as the one axum answers with.
pub fn axum_status(status: HttpStatus) -> StatusCode {
    StatusCode::from_u16(status.code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Converts semantic results into axum JSON responses.
pub struct AxumJsonOutput;

impl<From> OutputAlg<From> for AxumJsonOutput {
    type Output = Json<From>;

    fn output(from: From) -> Self::Output {
        Json(from)
    }
}

/// Converts semantic results into axum plain-text responses.
pub struct AxumTextOutput;

impl<From> OutputAlg<From> for AxumTextOutput
where
    From: Display,
{
    type Output = String;

    fn output(from: From) -> Self::Output {
        from.to_string()
    }
}

/// Converts semantic results into axum HTML responses.
pub struct AxumHtmlOutput;

impl<From> OutputAlg<From> for AxumHtmlOutput {
    type Output = Html<From>;

    fn output(from: From) -> Self::Output {
        Html(from)
    }
}

/// Converts semantic results into axum raw-byte responses.
pub struct AxumBytesOutput;

impl<From> OutputAlg<From> for AxumBytesOutput
where
    From: Into<Vec<u8>>,
{
    type Output = Vec<u8>;

    fn output(from: From) -> Self::Output {
        from.into()
    }
}

/// Converts a handler that returns nothing into an answer with no body.
pub struct AxumEmptyOutput;

impl OutputAlg<()> for AxumEmptyOutput {
    type Output = StatusCode;

    fn output((): ()) -> Self::Output {
        StatusCode::NO_CONTENT
    }
}

/// Converts a semantic location into an axum redirect.
pub struct AxumRedirectOutput;

impl<From> OutputAlg<From> for AxumRedirectOutput
where
    From: Display,
{
    type Output = Redirect;

    fn output(from: From) -> Self::Output {
        Redirect::to(&from.to_string())
    }
}

/// Converts a semantic stream of chunks into an axum streamed body.
pub struct AxumStreamOutput;

/// Reads a body stated as chunks as the bytes this framework moves.
fn moving<Chunks>(chunks: Chunks) -> impl Stream<Item = Result<Vec<u8>, IoError>> + Send
where
    Chunks: ChunksAlg + Send + 'static,
    Chunks::Chunk: Into<Vec<u8>> + Send,
    Chunks::Error: Display,
{
    chunks.moving().map_ok(Into::into).map_err(|error| IoError::other(error.to_string()))
}

impl<From> OutputAlg<From> for AxumStreamOutput
where
    From: ChunksAlg + Send + 'static,
    From::Chunk: Into<Vec<u8>> + Send,
    From::Error: Display,
{
    type Output = Body;

    fn output(from: From) -> Self::Output {
        Body::from_stream(moving(from))
    }
}

/// Answers with a header the handler stated, beside the body it stated.
pub struct AxumHeaderOutput<Inner, Name>(PhantomData<fn(Inner, Name)>);

/// Carries a converted body and a header's value until axum writes them.
pub struct AxumCarrying<Output, Name> {
    body: Output,
    value: String,
    name: PhantomData<fn(Name)>,
}

impl<Inner, Name, Value, Rest> OutputAlg<(Value, Rest)> for AxumHeaderOutput<Inner, Name>
where
    Inner: OutputAlg<Rest>,
    Value: Display,
{
    type Output = AxumCarrying<Inner::Output, Name>;

    fn output((value, rest): (Value, Rest)) -> Self::Output {
        AxumCarrying { body: Inner::output(rest), value: value.to_string(), name: PhantomData }
    }
}

impl<Output, Name> IntoResponse for AxumCarrying<Output, Name>
where
    Output: IntoResponse,
    Name: HeaderNameAlg,
{
    fn into_response(self) -> Response {
        let mut response = self.body.into_response();
        if let Ok(value) = HeaderValue::from_str(&self.value) {
            response.headers_mut().insert(HeaderName::from_static(Name::HEADER_NAME), value);
        }

        response
    }
}

/// Answers with the status an endpoint declared, around the body it already states.
pub struct AxumStatusOutput<Inner, const CODE: u16>(PhantomData<Inner>);

/// Carries a converted body until axum gives it the declared status.
pub struct AxumStatusResponse<Output, const CODE: u16>(Output);

impl<Inner, From, const CODE: u16> OutputAlg<From> for AxumStatusOutput<Inner, CODE>
where
    Inner: OutputAlg<From>,
{
    type Output = AxumStatusResponse<Inner::Output, CODE>;

    fn output(from: From) -> Self::Output {
        AxumStatusResponse(Inner::output(from))
    }
}

impl<Output, const CODE: u16> IntoResponse for AxumStatusResponse<Output, CODE>
where
    Output: IntoResponse,
{
    fn into_response(self) -> Response {
        let mut response = self.0.into_response();
        *response.status_mut() = axum_status(HttpStatus::new(CODE));

        response
    }
}

/// Answers with what a failure means when the handler failed, and with the body it states otherwise.
pub struct AxumResultOutput<Inner, Error>(PhantomData<fn(Inner, Error)>);

/// Carries either a converted body or the meaning of a failure until axum answers with it.
pub struct AxumResultResponse<Output>(Result<Output, (HttpStatus, String)>);

impl<Inner, Error, Value> OutputAlg<Result<Value, Error>> for AxumResultOutput<Inner, Error>
where
    Inner: OutputAlg<Value>,
    Error: HttpErrorAlg,
{
    type Output = AxumResultResponse<Inner::Output>;

    fn output(from: Result<Value, Error>) -> Self::Output {
        AxumResultResponse(from.map(Inner::output).map_err(|error| (error.http_status(), error.http_message())))
    }
}

impl<Output> IntoResponse for AxumResultResponse<Output>
where
    Output: IntoResponse,
{
    fn into_response(self) -> Response {
        match self.0 {
            Ok(output) => output.into_response(),
            Err((status, message)) => (axum_status(status), message).into_response(),
        }
    }
}

/// Converts semantic file results into downloadable axum responses.
pub struct AxumFileOutput;

/// Carries a file result and filename until axum creates the response.
pub struct AxumFileResponse<From>(From);

impl<From> OutputAlg<From> for AxumFileOutput {
    type Output = AxumFileResponse<From>;

    fn output(from: From) -> Self::Output {
        AxumFileResponse(from)
    }
}

impl<File, Error> IntoResponse for AxumFileResponse<(Result<File, Error>, String)>
where
    File: IntoResponse,
    Error: HttpErrorAlg + Send,
{
    fn into_response(self) -> Response {
        let (file, name) = self.0;
        match file {
            Ok(file) => {
                let mut response = file.into_response();
                let name = name.replace(['\r', '\n', '"'], "_");
                if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{name}\"")) {
                    response.headers_mut().insert(header::CONTENT_DISPOSITION, value);
                }
                response
            }
            Err(error) => axum_status(error.http_status()).into_response(),
        }
    }
}

macro_rules! axum_outputs {
    ($($alg:ident => $selected:ident, $output:ty),+ $(,)?) => {
        $(
            impl<Context> $alg for AxumHandlerImpl<Context> {
                type $selected<From> = $output;
            }
        )+
    };
}

axum_outputs! {
    JsonOutAlg     => Json, AxumJsonOutput,
    FileOutAlg     => File, AxumFileOutput,
    TextOutAlg     => Text, AxumTextOutput,
    HtmlOutAlg     => Html, AxumHtmlOutput,
    BytesOutAlg    => Bytes, AxumBytesOutput,
    EmptyOutAlg    => Empty, AxumEmptyOutput,
    RedirectOutAlg => Redirect, AxumRedirectOutput,
    StreamOutAlg   => Stream, AxumStreamOutput,
}

impl<Context> HeaderOutAlg for AxumHandlerImpl<Context> {
    type Header<Inner, Name> = AxumHeaderOutput<Inner, Name>;
}

impl<Context> StatusOutAlg for AxumHandlerImpl<Context> {
    type Status<Inner, const CODE: u16> = AxumStatusOutput<Inner, CODE>;
}

impl<Context> ResultOutAlg for AxumHandlerImpl<Context> {
    type Result<Inner, Error> = AxumResultOutput<Inner, Error>;
}
