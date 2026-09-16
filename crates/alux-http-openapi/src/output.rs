//! States what each output kind answers with, in the vocabulary a document reads.

use crate::OpenApiHandlerImpl;
use alux_http::{
    BytesOutAlg, EmptyOutAlg, FileOutAlg, HeaderNameAlg, HeaderOutAlg, HtmlOutAlg, HttpErrorAlg, HttpStatus,
    JsonOutAlg, OutputAlg, RedirectOutAlg, ResultOutAlg, StatusOutAlg, StreamOutAlg, TextOutAlg,
};
use alux_shape::ShapeOf;
use alux_shape_jsonschema::{JsonSchema, JsonSchemaShape};
use core::marker::PhantomData;
use serde_json::Value;

/// One answer an endpoint states, as a document describes it.
#[derive(Debug, Clone)]
pub struct OpenApiAnswer {
    /// The status this answer carries.
    pub status: HttpStatus,
    /// Every header this answer carries beside its body.
    pub headers: Vec<&'static str>,
    /// The media type this answer is written as, where it has a body.
    pub content_type: Option<&'static str>,
    /// The schema this answer's body carries, where it has one.
    pub schema: Option<Value>,
}

impl OpenApiAnswer {
    /// States an answer carrying a body of a stated media type.
    pub fn content(status: HttpStatus, content_type: &'static str, schema: Value) -> Self {
        Self { status, headers: Vec::new(), content_type: Some(content_type), schema: Some(schema) }
    }

    /// States an answer carrying no body.
    pub fn bodiless(status: HttpStatus) -> Self {
        Self { status, headers: Vec::new(), content_type: None, schema: None }
    }
}

/// States how a document describes what one output kind answers with.
///
/// An interpretation that runs converts a value it was given. This one has no value, so what an
/// endpoint answers with has to be read from the kind's type.
pub trait OpenApiOutputAlg<From> {
    /// Describes every answer this kind states, naming the shapes it mentions along the way.
    fn answers(schema: &JsonSchemaShape) -> Vec<OpenApiAnswer>;
}

macro_rules! openapi_outputs {
    ($($output:ident => $content_type:literal, $meaning:literal),+ $(,)?) => {
        $(
            #[doc = concat!("Describes ", $meaning, " answers in a document.")]
            pub struct $output;

            impl<From> OutputAlg<From> for $output {
                type Output = From;

                fn output(from: From) -> From {
                    from
                }
            }

            impl<From> OpenApiOutputAlg<From> for $output
            where
                From: ShapeOf<JsonSchemaShape, Shape = JsonSchema>,
            {
                fn answers(schema: &JsonSchemaShape) -> Vec<OpenApiAnswer> {
                    vec![OpenApiAnswer::content(HttpStatus::OK, $content_type, From::shape_of(schema).into_value())]
                }
            }
        )+
    };
}

openapi_outputs! {
    OpenApiJsonOutput   => "application/json", "JSON",
    OpenApiTextOutput   => "text/plain", "plain-text",
    OpenApiHtmlOutput   => "text/html", "HTML",
    OpenApiBytesOutput  => "application/octet-stream", "raw-byte",
    OpenApiFileOutput   => "application/octet-stream", "streamed-file",
    OpenApiStreamOutput => "application/octet-stream", "streamed",
}

/// Describes an answer with no body in a document.
pub struct OpenApiEmptyOutput;

impl<From> OutputAlg<From> for OpenApiEmptyOutput {
    type Output = From;

    fn output(from: From) -> From {
        from
    }
}

impl OpenApiOutputAlg<()> for OpenApiEmptyOutput {
    fn answers(_schema: &JsonSchemaShape) -> Vec<OpenApiAnswer> {
        vec![OpenApiAnswer::bodiless(HttpStatus::NO_CONTENT)]
    }
}

/// Describes a redirect in a document.
pub struct OpenApiRedirectOutput;

impl<From> OutputAlg<From> for OpenApiRedirectOutput {
    type Output = From;

    fn output(from: From) -> From {
        from
    }
}

impl<From> OpenApiOutputAlg<From> for OpenApiRedirectOutput {
    fn answers(_schema: &JsonSchemaShape) -> Vec<OpenApiAnswer> {
        vec![OpenApiAnswer::bodiless(HttpStatus::SEE_OTHER)]
    }
}

/// Describes a header an answer carries, beside the body it states.
pub struct OpenApiHeaderOutput<Inner, Name>(PhantomData<fn(Inner, Name)>);

impl<Inner, Name, From> OutputAlg<From> for OpenApiHeaderOutput<Inner, Name> {
    type Output = From;

    fn output(from: From) -> From {
        from
    }
}

impl<Inner, Name, Value, Rest> OpenApiOutputAlg<(Value, Rest)> for OpenApiHeaderOutput<Inner, Name>
where
    Inner: OpenApiOutputAlg<Rest>,
    Name: HeaderNameAlg,
{
    fn answers(schema: &JsonSchemaShape) -> Vec<OpenApiAnswer> {
        Inner::answers(schema)
            .into_iter()
            .map(|answer| {
                let mut carried = answer;
                if carried.status.is_success() {
                    carried.headers.push(Name::HEADER_NAME);
                }

                carried
            })
            .collect()
    }
}

impl<Context> HeaderOutAlg for OpenApiHandlerImpl<Context> {
    type Header<Inner, Name> = OpenApiHeaderOutput<Inner, Name>;
}

/// Describes a declared status around the answer a kind already states.
pub struct OpenApiStatusOutput<Inner, const CODE: u16>(PhantomData<Inner>);

impl<Inner, From, const CODE: u16> OutputAlg<From> for OpenApiStatusOutput<Inner, CODE> {
    type Output = From;

    fn output(from: From) -> From {
        from
    }
}

impl<Inner, From, const CODE: u16> OpenApiOutputAlg<From> for OpenApiStatusOutput<Inner, CODE>
where
    Inner: OpenApiOutputAlg<From>,
{
    fn answers(schema: &JsonSchemaShape) -> Vec<OpenApiAnswer> {
        Inner::answers(schema)
            .into_iter()
            .map(|answer| OpenApiAnswer { status: HttpStatus::new(CODE), ..answer })
            .collect()
    }
}

/// Describes both what an endpoint answers with and what its failures answer with.
///
/// A failure states its statuses on its type, which is the only place a document can read them: it
/// folds a program rather than running one, so it never holds a failure to ask.
pub struct OpenApiResultOutput<Inner, Error>(PhantomData<fn(Inner, Error)>);

impl<Inner, Error, From> OutputAlg<From> for OpenApiResultOutput<Inner, Error> {
    type Output = From;

    fn output(from: From) -> From {
        from
    }
}

impl<Inner, Error, Value> OpenApiOutputAlg<Result<Value, Error>> for OpenApiResultOutput<Inner, Error>
where
    Inner: OpenApiOutputAlg<Value>,
    Error: HttpErrorAlg,
{
    fn answers(schema: &JsonSchemaShape) -> Vec<OpenApiAnswer> {
        let failures = Error::HTTP_STATUSES.iter().map(|status| OpenApiAnswer::content(*status, "text/plain", text()));

        Inner::answers(schema).into_iter().chain(failures).collect()
    }
}

/// The schema a stated failure carries, which is the message it answers with.
fn text() -> Value {
    serde_json::json!({ "type": "string" })
}

macro_rules! openapi_kinds {
    ($($alg:ident => $selected:ident, $output:ty),+ $(,)?) => {
        $(
            impl<Context> $alg for OpenApiHandlerImpl<Context> {
                type $selected<From> = $output;
            }
        )+
    };
}

openapi_kinds! {
    JsonOutAlg     => Json, OpenApiJsonOutput,
    FileOutAlg     => File, OpenApiFileOutput,
    TextOutAlg     => Text, OpenApiTextOutput,
    HtmlOutAlg     => Html, OpenApiHtmlOutput,
    BytesOutAlg    => Bytes, OpenApiBytesOutput,
    EmptyOutAlg    => Empty, OpenApiEmptyOutput,
    RedirectOutAlg => Redirect, OpenApiRedirectOutput,
    StreamOutAlg   => Stream, OpenApiStreamOutput,
}

impl<Context> StatusOutAlg for OpenApiHandlerImpl<Context> {
    type Status<Inner, const CODE: u16> = OpenApiStatusOutput<Inner, CODE>;
}

impl<Context> ResultOutAlg for OpenApiHandlerImpl<Context> {
    type Result<Inner, Error> = OpenApiResultOutput<Inner, Error>;
}
