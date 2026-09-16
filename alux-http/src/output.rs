//! States what an endpoint answers with, before any framework can convert it.

use core::marker::PhantomData;

/// Transforms an inferred handler result into its portable API output.
///
/// Converter families are selected from an endpoint's output kind. The handler
/// result supplies `From`, so API declarations never repeat it.
pub trait OutputAlg<From> {
    /// The transport value produced from the semantic handler result.
    type Output;

    /// Converts a handler result into the declared API output.
    fn output(from: From) -> Self::Output;
}

/// Resolves a portable output kind through an interpreter.
pub trait OutputKindAlg<Interpreter: ?Sized, From> {
    /// The concrete converter chosen by `Interpreter` for this output kind.
    ///
    /// That the converter reads `From` is required where the endpoint is compiled rather than
    /// here, so a kind may answer for some results and not others. An endpoint stating no body
    /// converts a handler that returns nothing, and an endpoint stating a failure converts a
    /// handler that can fail.
    type Transform;
}

macro_rules! output_kinds {
    ($($kind:ident => $alg:ident, $selected:ident, $meaning:literal),+ $(,)?) => {
        $(
            #[doc = concat!("Selects the converter used for ", $meaning, " API outputs.")]
            pub trait $alg {
                #[doc = concat!("The ", $meaning, " converter selected for `From`.")]
                type $selected<From>;
            }

            #[doc = concat!("Selects ", $meaning, " output semantics.")]
            #[derive(Debug, Default)]
            pub struct $kind;

            impl<Interpreter, From> OutputKindAlg<Interpreter, From> for $kind
            where
                Interpreter: $alg + ?Sized,
            {
                type Transform = Interpreter::$selected<From>;
            }
        )+
    };
}

output_kinds! {
    JsonOut     => JsonOutAlg, Json, "JSON",
    FileOut     => FileOutAlg, File, "streamed file",
    TextOut     => TextOutAlg, Text, "plain text",
    HtmlOut     => HtmlOutAlg, Html, "HTML",
    BytesOut    => BytesOutAlg, Bytes, "raw byte",
    EmptyOut    => EmptyOutAlg, Empty, "empty",
    RedirectOut => RedirectOutAlg, Redirect, "redirect",
    StreamOut   => StreamOutAlg, Stream, "streamed",
}

/// Selects the converter used to answer with a declared status.
pub trait StatusOutAlg {
    /// The converter answering with `CODE` and the body `Inner` states.
    type Status<Inner, const CODE: u16>;
}

/// Answers with `CODE` and the body `Kind` states.
///
/// The status is part of what an endpoint is declared to answer, so it is stated once in the
/// program rather than chosen inside a handler.
#[derive(Debug, Default)]
pub struct StatusOut<Kind, const CODE: u16>(PhantomData<Kind>);

impl<Interpreter, From, Kind, const CODE: u16> OutputKindAlg<Interpreter, From> for StatusOut<Kind, CODE>
where
    Interpreter: StatusOutAlg + ?Sized,
    Kind: OutputKindAlg<Interpreter, From>,
{
    type Transform = Interpreter::Status<Kind::Transform, CODE>;
}

/// Selects the converter used to answer with a header beside a body.
pub trait HeaderOutAlg {
    /// The converter writing `Name` beside the body `Inner` states.
    type Header<Inner, Name>;
}

/// Answers with a header the handler states, beside the body `Kind` states.
///
/// The handler answers with the header's value and the body, in that order, because a header whose
/// value an endpoint cannot know is a header only the handler can state. Which header it is, is
/// stated here: a name is all a header is to a program.
#[derive(Debug, Default)]
pub struct HeaderOut<Kind, Name>(PhantomData<fn(Kind, Name)>);

impl<Interpreter, Value, Rest, Kind, Name> OutputKindAlg<Interpreter, (Value, Rest)> for HeaderOut<Kind, Name>
where
    Interpreter: HeaderOutAlg + ?Sized,
    Kind: OutputKindAlg<Interpreter, Rest>,
{
    type Transform = Interpreter::Header<Kind::Transform, Name>;
}

/// Selects the converter used for a handler that can fail.
pub trait ResultOutAlg {
    /// The converter answering with `Inner` on success and with what `Error` means otherwise.
    type Result<Inner, Error>;
}

/// Answers with `Kind` when the handler succeeded, and with what its failure means otherwise.
///
/// A handler that can fail returns a `Result`, so this kind reads one. What a failure means is
/// stated by [`HttpErrorAlg`](crate::HttpErrorAlg) on the error itself.
#[derive(Debug, Default)]
pub struct ResultOut<Kind>(PhantomData<Kind>);

impl<Interpreter, Kind, Value, Error> OutputKindAlg<Interpreter, Result<Value, Error>> for ResultOut<Kind>
where
    Interpreter: ResultOutAlg + ?Sized,
    Kind: OutputKindAlg<Interpreter, Value>,
{
    type Transform = Interpreter::Result<Kind::Transform, Error>;
}
