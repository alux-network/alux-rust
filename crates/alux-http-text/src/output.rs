//! Names the output conversion an endpoint selects, without converting anything.

use crate::TextHandlerImpl;
use alux_http::{
    BytesOutAlg, EmptyOutAlg, FileOutAlg, HeaderOutAlg, HtmlOutAlg, HttpErrorAlg, HttpStatus, JsonOutAlg, OutputAlg,
    RedirectOutAlg, ResultOutAlg, StatusOutAlg, StreamOutAlg, TextOutAlg,
};
use core::marker::PhantomData;

macro_rules! text_outputs {
    ($($output:ident => $alg:ident, $selected:ident, $meaning:literal),+ $(,)?) => {
        $(
            #[doc = concat!("Interprets ", $meaning, " output selection in text descriptions.")]
            pub struct $output;

            impl<From> OutputAlg<From> for $output {
                type Output = From;

                fn output(from: From) -> From {
                    from
                }
            }

            impl $alg for TextHandlerImpl {
                type $selected<From> = $output;
            }
        )+
    };
}

text_outputs! {
    TextJsonOutput     => JsonOutAlg, Json, "JSON",
    TextFileOutput     => FileOutAlg, File, "streamed-file",
    TextTextOutput     => TextOutAlg, Text, "plain-text",
    TextHtmlOutput     => HtmlOutAlg, Html, "HTML",
    TextBytesOutput    => BytesOutAlg, Bytes, "raw-byte",
    TextRedirectOutput => RedirectOutAlg, Redirect, "redirect",
    TextStreamOutput   => StreamOutAlg, Stream, "streamed",
}

/// Interprets empty output selection in text descriptions.
///
/// An endpoint stating no body converts a handler that returns nothing, so this reads `()` and
/// nothing else. That is the partiality the description witnesses on every interpreter's behalf.
pub struct TextEmptyOutput;

impl OutputAlg<()> for TextEmptyOutput {
    type Output = ();

    fn output((): ()) {}
}

impl EmptyOutAlg for TextHandlerImpl {
    type Empty<From> = TextEmptyOutput;
}

/// Interprets a header an answer carries, in text descriptions.
pub struct TextHeaderOutput<Inner, Name>(PhantomData<fn(Inner, Name)>);

impl<Inner, Name, Value, Rest> OutputAlg<(Value, Rest)> for TextHeaderOutput<Inner, Name>
where
    Inner: OutputAlg<Rest>,
{
    type Output = Inner::Output;

    fn output((_value, rest): (Value, Rest)) -> Self::Output {
        Inner::output(rest)
    }
}

impl HeaderOutAlg for TextHandlerImpl {
    type Header<Inner, Name> = TextHeaderOutput<Inner, Name>;
}

/// Interprets a declared status in text descriptions.
pub struct TextStatusOutput<Inner, const CODE: u16>(PhantomData<Inner>);

impl<Inner, From, const CODE: u16> OutputAlg<From> for TextStatusOutput<Inner, CODE>
where
    Inner: OutputAlg<From>,
{
    type Output = Inner::Output;

    fn output(from: From) -> Self::Output {
        Inner::output(from)
    }
}

impl StatusOutAlg for TextHandlerImpl {
    type Status<Inner, const CODE: u16> = TextStatusOutput<Inner, CODE>;
}

/// Interprets a failing handler in text descriptions.
///
/// The description keeps the status a failure is answered with and discards the message, because
/// what a surface states is that the failure has a status, not what it reads like.
pub struct TextResultOutput<Inner, Error>(PhantomData<fn(Inner, Error)>);

impl<Inner, Error, Value> OutputAlg<Result<Value, Error>> for TextResultOutput<Inner, Error>
where
    Inner: OutputAlg<Value>,
    Error: HttpErrorAlg,
{
    type Output = Result<Inner::Output, HttpStatus>;

    fn output(from: Result<Value, Error>) -> Self::Output {
        from.map(Inner::output).map_err(|error| error.http_status())
    }
}

impl ResultOutAlg for TextHandlerImpl {
    type Result<Inner, Error> = TextResultOutput<Inner, Error>;
}
