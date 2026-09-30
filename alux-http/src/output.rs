//! States what an endpoint answers with, before any framework can convert it.

use crate::WithAlg;
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
    ($($declaration:ident => $kind:ident, $alg:ident, $selected:ident, $meaning:literal),+ $(,)?) => {
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

with_output_kinds!(output_kinds);

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

/// Selects the converter used to answer with the headers a named product states, beside a body.
pub trait HeadersOutAlg {
    /// The converter writing each value `Headers` states beside the body `Inner` states.
    type Headers<Inner, Headers>;
}

/// Answers with the headers a named product states, beside the body `Kind` states.
///
/// The output twin of reading headers into a product: each member is a header, named by the words
/// its member name states, so `cache_control` is `cache-control`. A member stating nothing is not
/// written, and a member stating many values writes one header for each, as `set-cookie` needs. The
/// handler answers with the product and the body, in that order.
#[derive(Debug, Default)]
pub struct HeadersOut<Kind, Headers>(PhantomData<fn(Kind, Headers)>);

impl<Interpreter, Headers, Rest, Kind> OutputKindAlg<Interpreter, (Headers, Rest)> for HeadersOut<Kind, Headers>
where
    Interpreter: HeadersOutAlg + ?Sized,
    Kind: OutputKindAlg<Interpreter, Rest>,
{
    type Transform = Interpreter::Headers<Kind::Transform, Headers>;
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

/// Holds the wrappers a declaration states before its kind, the outermost first.
///
/// A declaration reads from the outside in: `.out_header::<ETag>().result().json()` answers with an
/// `ETag` around a result around a JSON body, and its handler returns `(etag, Result<body, E>)`. The
/// kind closes the declaration, folding the wrappers held here around it.
///
/// Nothing wraps a declaration its kind has closed:
///
/// ```compile_fail,E0277
/// use alux_http::HttpProgramBuilder;
///
/// let syntax = HttpProgramBuilder;
/// let _ = syntax.op(()).json().result();
/// ```
#[derive(Debug, Default)]
pub struct Pending<Wrappers>(PhantomData<Wrappers>);

/// States one wrapper as the kind it makes of the kind inside it.
pub trait WrapAlg {
    /// The kind this wrapper makes around `Inner`.
    type Wrap<Inner>;
}

impl<const CODE: u16> WrapAlg for StatusOut<(), CODE> {
    type Wrap<Inner> = StatusOut<Inner, CODE>;
}

impl<Name> WrapAlg for HeaderOut<(), Name> {
    type Wrap<Inner> = HeaderOut<Inner, Name>;
}

impl<Headers> WrapAlg for HeadersOut<(), Headers> {
    type Wrap<Inner> = HeadersOut<Inner, Headers>;
}

impl WrapAlg for ResultOut<()> {
    type Wrap<Inner> = ResultOut<Inner>;
}

/// Reads a declaration still open to wrappers: one stating nothing yet, or wrappers and no kind.
#[diagnostic::on_unimplemented(
    message = "`{Self}` already states its output kind",
    note = "wrappers such as `.status()`, `.out_header()`, `.out_headers()`, and `.result()` come before the kind, which closes the declaration"
)]
pub trait OpenAlg {
    /// The declaration with `Wrapper` inside every wrapper already stated.
    type With<Wrapper>;
}

impl OpenAlg for () {
    type With<Wrapper> = Pending<(Wrapper,)>;
}

impl<Wrappers> OpenAlg for Pending<Wrappers>
where
    Wrappers: WithAlg,
{
    type With<Wrapper> = Pending<Wrappers::With<Wrapper>>;
}

/// Closes a declaration with its kind, folding the wrappers it states around that kind.
#[diagnostic::on_unimplemented(
    message = "`{Self}` already states its output kind",
    note = "a declaration states one kind, last, after the wrappers around it"
)]
pub trait CloseAlg<Kind> {
    /// The kind the whole declaration answers with.
    type Closed;
}

impl<Kind> CloseAlg<Kind> for () {
    type Closed = Kind;
}

impl<Kind, Wrappers> CloseAlg<Kind> for Pending<Wrappers>
where
    Wrappers: FoldAlg<Kind>,
{
    type Closed = Wrappers::Folded;
}

/// Folds wrappers around a kind, the first of them outermost.
pub trait FoldAlg<Kind> {
    /// The kind the wrappers make around `Kind`.
    type Folded;
}

macro_rules! fold_wrappers {
    () => {
        impl<Kind> FoldAlg<Kind> for () {
            type Folded = Kind;
        }
    };
    ($first:ident $(, $rest:ident)*) => {
        impl<Kind, $first $(, $rest)*> FoldAlg<Kind> for ($first, $($rest,)*)
        where
            $first: WrapAlg,
            ($($rest,)*): FoldAlg<Kind>,
        {
            type Folded = $first::Wrap<<($($rest,)*) as FoldAlg<Kind>>::Folded>;
        }

        fold_wrappers!($($rest),*);
    };
}

fold_wrappers!(W1, W2, W3, W4, W5, W6, W7, W8, W9, W10, W11, W12, W13, W14, W15, W16);
