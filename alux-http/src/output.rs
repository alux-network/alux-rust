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

/// Selects the converter used for JSON API outputs.
pub trait JsonOutAlg {
    /// The JSON converter selected for `From`.
    type Json<From>: OutputAlg<From>;
}

/// Selects the converter used for streamed file API outputs.
pub trait FileOutAlg {
    /// The streamed-file converter selected for `From`.
    type File<From>: OutputAlg<From>;
}

/// Resolves a portable output kind through an interpreter.
pub trait OutputKindAlg<Interpreter: ?Sized, From> {
    /// The concrete converter chosen by `Interpreter` for this output kind.
    type Transform: OutputAlg<From>;
}

/// Selects JSON output semantics.
pub struct JsonOut;

impl<Interpreter, From> OutputKindAlg<Interpreter, From> for JsonOut
where
    Interpreter: JsonOutAlg + ?Sized,
{
    type Transform = Interpreter::Json<From>;
}

/// Selects streamed-file output semantics.
pub struct FileOut;

impl<Interpreter, From> OutputKindAlg<Interpreter, From> for FileOut
where
    Interpreter: FileOutAlg + ?Sized,
{
    type Transform = Interpreter::File<From>;
}
