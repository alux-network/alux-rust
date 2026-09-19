//! States what a call answers with, in the vocabulary a caller writes.

use crate::TsHttpClient;
use alux_http::{
    BytesOutAlg, EmptyOutAlg, FileOutAlg, HeaderOutAlg, HtmlOutAlg, JsonOutAlg, OutputAlg, RedirectOutAlg,
    ResultOutAlg, StatusOutAlg, StreamOutAlg, TextOutAlg,
};
use alux_shape::{ShapeAlg, ShapeOf};
use alux_shape_typescript::{TsShape, TsType};
use core::marker::PhantomData;

/// States what one output kind answers a caller with.
///
/// A failure is a rejected call rather than a value, so what a call answers with is what it answers
/// with when it answers at all.
pub trait TsOutputAlg<From> {
    /// Reads the type a caller receives.
    fn answer(shapes: &TsShape) -> TsType;
}

/// Answers with whatever the handler's result states.
pub struct TsJsonOutput;

impl<From> OutputAlg<From> for TsJsonOutput {
    type Output = From;

    fn output(from: From) -> From {
        from
    }
}

impl<From> TsOutputAlg<From> for TsJsonOutput
where
    From: ShapeOf<TsShape, Shape = TsType>,
{
    fn answer(shapes: &TsShape) -> TsType {
        From::shape_of(shapes)
    }
}

macro_rules! ts_outputs {
    ($($output:ident => $answer:ident, $meaning:literal),+ $(,)?) => {
        $(
            #[doc = concat!("Answers a caller with what ", $meaning, " states.")]
            pub struct $output;

            impl<From> OutputAlg<From> for $output {
                type Output = From;

                fn output(from: From) -> From {
                    from
                }
            }

            impl<From> TsOutputAlg<From> for $output {
                fn answer(shapes: &TsShape) -> TsType {
                    shapes.$answer()
                }
            }
        )+
    };
}

ts_outputs! {
    TsTextOutput     => text, "plain text",
    TsHtmlOutput     => text, "HTML",
    TsRedirectOutput => unit, "a redirect",
    TsEmptyOutput    => unit, "an answer with no body",
}

macro_rules! ts_byte_outputs {
    ($($output:ident => $meaning:literal),+ $(,)?) => {
        $(
            #[doc = concat!("Answers a caller with the bytes ", $meaning, " states.")]
            pub struct $output;

            impl<From> OutputAlg<From> for $output {
                type Output = From;

                fn output(from: From) -> From {
                    from
                }
            }

            impl<From> TsOutputAlg<From> for $output {
                fn answer(shapes: &TsShape) -> TsType {
                    shapes.bytes(None)
                }
            }
        )+
    };
}

ts_byte_outputs! {
    TsBytesOutput  => "a raw-byte answer",
    TsFileOutput   => "a streamed file",
    TsStreamOutput => "a streamed answer",
}

/// Answers with what the kind beneath states, since a header reaches a caller beside it.
pub struct TsHeaderOutput<Inner, Name>(PhantomData<fn(Inner, Name)>);

impl<Inner, Name, From> OutputAlg<From> for TsHeaderOutput<Inner, Name> {
    type Output = From;

    fn output(from: From) -> From {
        from
    }
}

impl<Inner, Name, Value, Rest> TsOutputAlg<(Value, Rest)> for TsHeaderOutput<Inner, Name>
where
    Inner: TsOutputAlg<Rest>,
{
    fn answer(shapes: &TsShape) -> TsType {
        Inner::answer(shapes)
    }
}

impl HeaderOutAlg for TsHttpClient {
    type Header<Inner, Name> = TsHeaderOutput<Inner, Name>;
}

/// Answers with what the kind beneath states, whatever status the endpoint declared.
pub struct TsStatusOutput<Inner, const CODE: u16>(PhantomData<Inner>);

impl<Inner, From, const CODE: u16> OutputAlg<From> for TsStatusOutput<Inner, CODE> {
    type Output = From;

    fn output(from: From) -> From {
        from
    }
}

impl<Inner, From, const CODE: u16> TsOutputAlg<From> for TsStatusOutput<Inner, CODE>
where
    Inner: TsOutputAlg<From>,
{
    fn answer(shapes: &TsShape) -> TsType {
        Inner::answer(shapes)
    }
}

/// Answers with what the kind beneath states, since a failure rejects the call instead.
pub struct TsResultOutput<Inner, Error>(PhantomData<fn(Inner, Error)>);

impl<Inner, Error, From> OutputAlg<From> for TsResultOutput<Inner, Error> {
    type Output = From;

    fn output(from: From) -> From {
        from
    }
}

impl<Inner, Error, Value> TsOutputAlg<Result<Value, Error>> for TsResultOutput<Inner, Error>
where
    Inner: TsOutputAlg<Value>,
{
    fn answer(shapes: &TsShape) -> TsType {
        Inner::answer(shapes)
    }
}

macro_rules! ts_kinds {
    ($($alg:ident => $selected:ident, $output:ty),+ $(,)?) => {
        $(
            impl $alg for TsHttpClient {
                type $selected<From> = $output;
            }
        )+
    };
}

ts_kinds! {
    JsonOutAlg     => Json, TsJsonOutput,
    FileOutAlg     => File, TsFileOutput,
    TextOutAlg     => Text, TsTextOutput,
    HtmlOutAlg     => Html, TsHtmlOutput,
    BytesOutAlg    => Bytes, TsBytesOutput,
    EmptyOutAlg    => Empty, TsEmptyOutput,
    RedirectOutAlg => Redirect, TsRedirectOutput,
    StreamOutAlg   => Stream, TsStreamOutput,
}

impl StatusOutAlg for TsHttpClient {
    type Status<Inner, const CODE: u16> = TsStatusOutput<Inner, CODE>;
}

impl ResultOutAlg for TsHttpClient {
    type Result<Inner, Error> = TsResultOutput<Inner, Error>;
}
