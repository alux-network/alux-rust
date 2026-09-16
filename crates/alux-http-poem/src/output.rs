use crate::PoemHandlerImpl;
use alux_http::{
    BytesOutAlg, ChunksAlg, ChunksExt, EmptyOutAlg, FileOutAlg, HeaderNameAlg, HeaderOutAlg, HtmlOutAlg, HttpErrorAlg,
    HttpStatus, JsonOutAlg, OutputAlg, RedirectOutAlg, ResultOutAlg, StatusOutAlg, StreamOutAlg, TextOutAlg,
};
use core::fmt::Display;
use core::marker::PhantomData;
use futures::{Stream, TryStreamExt};
use poem::http::{HeaderName, HeaderValue, StatusCode, header};
use poem::web::{Html, Json, Redirect};
use poem::{Body, IntoResponse, Response};
use std::io::Error as IoError;

/// Interprets a portable status as the one Poem answers with.
pub fn poem_status(status: HttpStatus) -> StatusCode {
    StatusCode::from_u16(status.code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

/// Converts semantic results into Poem JSON responses.
pub struct PoemJsonOutput;

impl<From> OutputAlg<From> for PoemJsonOutput {
    type Output = Json<From>;

    fn output(from: From) -> Self::Output {
        Json(from)
    }
}

/// Converts semantic results into Poem plain-text responses.
pub struct PoemTextOutput;

impl<From> OutputAlg<From> for PoemTextOutput
where
    From: Display,
{
    type Output = String;

    fn output(from: From) -> Self::Output {
        from.to_string()
    }
}

/// Converts semantic results into Poem HTML responses.
pub struct PoemHtmlOutput;

impl<From> OutputAlg<From> for PoemHtmlOutput {
    type Output = Html<From>;

    fn output(from: From) -> Self::Output {
        Html(from)
    }
}

/// Converts semantic results into Poem raw-byte responses.
pub struct PoemBytesOutput;

impl<From> OutputAlg<From> for PoemBytesOutput
where
    From: Into<Vec<u8>>,
{
    type Output = Vec<u8>;

    fn output(from: From) -> Self::Output {
        from.into()
    }
}

/// Converts a handler that returns nothing into an answer with no body.
pub struct PoemEmptyOutput;

impl OutputAlg<()> for PoemEmptyOutput {
    type Output = StatusCode;

    fn output((): ()) -> Self::Output {
        StatusCode::NO_CONTENT
    }
}

/// Converts a semantic location into a Poem redirect.
pub struct PoemRedirectOutput;

impl<From> OutputAlg<From> for PoemRedirectOutput
where
    From: Display,
{
    type Output = Redirect;

    fn output(from: From) -> Self::Output {
        Redirect::see_other(from)
    }
}

/// Converts a semantic stream of chunks into a Poem streamed body.
pub struct PoemStreamOutput;

/// Reads a body stated as chunks as the bytes this framework moves.
fn moving<Chunks>(chunks: Chunks) -> impl Stream<Item = Result<Vec<u8>, IoError>> + Send
where
    Chunks: ChunksAlg + Send + 'static,
    Chunks::Chunk: Into<Vec<u8>> + Send,
    Chunks::Error: Display,
{
    chunks.moving().map_ok(Into::into).map_err(|error| IoError::other(error.to_string()))
}

impl<From> OutputAlg<From> for PoemStreamOutput
where
    From: ChunksAlg + Send + 'static,
    From::Chunk: Into<Vec<u8>> + Send,
    From::Error: Display,
{
    type Output = Body;

    fn output(from: From) -> Self::Output {
        Body::from_bytes_stream(moving(from))
    }
}

/// Answers with a header the handler stated, beside the body it stated.
pub struct PoemHeaderOutput<Inner, Name>(PhantomData<fn(Inner, Name)>);

/// Carries a converted body and a header's value until Poem writes them.
pub struct PoemCarrying<Output, Name> {
    body: Output,
    value: String,
    name: PhantomData<fn(Name)>,
}

impl<Inner, Name, Value, Rest> OutputAlg<(Value, Rest)> for PoemHeaderOutput<Inner, Name>
where
    Inner: OutputAlg<Rest>,
    Value: Display,
{
    type Output = PoemCarrying<Inner::Output, Name>;

    fn output((value, rest): (Value, Rest)) -> Self::Output {
        PoemCarrying { body: Inner::output(rest), value: value.to_string(), name: PhantomData }
    }
}

impl<Output, Name> IntoResponse for PoemCarrying<Output, Name>
where
    Output: IntoResponse,
    Name: HeaderNameAlg + Send,
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
pub struct PoemStatusOutput<Inner, const CODE: u16>(PhantomData<Inner>);

/// Carries a converted body until Poem gives it the declared status.
pub struct PoemStatusResponse<Output, const CODE: u16>(Output);

impl<Inner, From, const CODE: u16> OutputAlg<From> for PoemStatusOutput<Inner, CODE>
where
    Inner: OutputAlg<From>,
{
    type Output = PoemStatusResponse<Inner::Output, CODE>;

    fn output(from: From) -> Self::Output {
        PoemStatusResponse(Inner::output(from))
    }
}

impl<Output, const CODE: u16> IntoResponse for PoemStatusResponse<Output, CODE>
where
    Output: IntoResponse,
{
    fn into_response(self) -> Response {
        let mut response = self.0.into_response();
        response.set_status(poem_status(HttpStatus::new(CODE)));

        response
    }
}

/// Answers with what a failure means when the handler failed, and with the body it states otherwise.
pub struct PoemResultOutput<Inner, Error>(PhantomData<fn(Inner, Error)>);

/// Carries either a converted body or the meaning of a failure until Poem answers with it.
pub struct PoemResultResponse<Output>(Result<Output, (HttpStatus, String)>);

impl<Inner, Error, Value> OutputAlg<Result<Value, Error>> for PoemResultOutput<Inner, Error>
where
    Inner: OutputAlg<Value>,
    Error: HttpErrorAlg,
{
    type Output = PoemResultResponse<Inner::Output>;

    fn output(from: Result<Value, Error>) -> Self::Output {
        PoemResultResponse(from.map(Inner::output).map_err(|error| (error.http_status(), error.http_message())))
    }
}

impl<Output> IntoResponse for PoemResultResponse<Output>
where
    Output: IntoResponse,
{
    fn into_response(self) -> Response {
        match self.0 {
            Ok(output) => output.into_response(),
            Err((status, message)) => Response::builder().status(poem_status(status)).body(message),
        }
    }
}

/// Converts semantic file results into downloadable Poem responses.
pub struct PoemFileOutput;

/// Carries a file result and filename until Poem creates the response.
pub struct PoemFileResponse<From>(From);

impl<From> OutputAlg<From> for PoemFileOutput {
    type Output = PoemFileResponse<From>;

    fn output(from: From) -> Self::Output {
        PoemFileResponse(from)
    }
}

impl<File, Error> IntoResponse for PoemFileResponse<(Result<File, Error>, String)>
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
            Err(error) => Response::builder().status(poem_status(error.http_status())).finish(),
        }
    }
}

macro_rules! poem_outputs {
    ($($alg:ident => $selected:ident, $output:ty),+ $(,)?) => {
        $(
            impl<Context> $alg for PoemHandlerImpl<Context> {
                type $selected<From> = $output;
            }
        )+
    };
}

poem_outputs! {
    JsonOutAlg     => Json, PoemJsonOutput,
    FileOutAlg     => File, PoemFileOutput,
    TextOutAlg     => Text, PoemTextOutput,
    HtmlOutAlg     => Html, PoemHtmlOutput,
    BytesOutAlg    => Bytes, PoemBytesOutput,
    EmptyOutAlg    => Empty, PoemEmptyOutput,
    RedirectOutAlg => Redirect, PoemRedirectOutput,
    StreamOutAlg   => Stream, PoemStreamOutput,
}

impl<Context> HeaderOutAlg for PoemHandlerImpl<Context> {
    type Header<Inner, Name> = PoemHeaderOutput<Inner, Name>;
}

impl<Context> StatusOutAlg for PoemHandlerImpl<Context> {
    type Status<Inner, const CODE: u16> = PoemStatusOutput<Inner, CODE>;
}

impl<Context> ResultOutAlg for PoemHandlerImpl<Context> {
    type Result<Inner, Error> = PoemResultOutput<Inner, Error>;
}

#[cfg(test)]
mod tests {
    use super::{PoemFileOutput, PoemResultOutput, PoemStatusOutput};
    use alux_http::OutputAlg;
    use poem::http::{StatusCode, header};
    use poem::web::Json;
    use poem::{Body, IntoResponse, Response};
    use std::io::{Error as IoError, ErrorKind};

    fn download(file: Result<Body, IoError>) -> Response {
        PoemFileOutput::output((file, "data.bin".to_owned())).into_response()
    }

    #[tokio::test]
    async fn names_the_file_in_a_download_response() {
        let mut response = download(Ok(Body::from_bytes((&b"data"[..]).into())));
        let disposition = response.headers().get(header::CONTENT_DISPOSITION).unwrap().to_str().unwrap().to_owned();

        assert!(response.status().is_success());
        assert_eq!(disposition, "attachment; filename=\"data.bin\"");
        assert_eq!(response.take_body().into_string().await.unwrap(), "data");
    }

    #[test]
    fn maps_a_missing_file_to_its_response_status() {
        let response = download(Err(IoError::new(ErrorKind::NotFound, "gone")));

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn answers_with_the_status_the_endpoint_declared() {
        type Created = PoemStatusOutput<super::PoemJsonOutput, 201>;

        let mut response = <Created as OutputAlg<u32>>::output(7).into_response();

        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(response.take_body().into_string().await.unwrap(), "7");
    }

    #[tokio::test]
    async fn answers_with_what_a_failure_means() {
        type Fallible = PoemResultOutput<super::PoemJsonOutput, IoError>;

        let failed =
            <Fallible as OutputAlg<Result<u32, IoError>>>::output(Err(IoError::new(ErrorKind::NotFound, "no")));
        let mut response = failed.into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.take_body().into_string().await.unwrap(), "no");

        let answered = <Fallible as OutputAlg<Result<u32, IoError>>>::output(Ok(7));
        let mut response = answered.into_response();

        assert!(response.status().is_success());
        assert_eq!(response.take_body().into_string().await.unwrap(), Json(7_u32).0.to_string());
    }
}
