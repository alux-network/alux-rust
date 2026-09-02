use alux_ext::{ApplyAlg, ext};
use trait_set::trait_set;

/// Describes construction and composition of JSON-RPC method collections.
pub trait JsonRpcAlg {
    /// The interpreter's homogeneous method collection.
    type Methods;

    /// Returns the empty JSON-RPC method collection.
    fn jsonrpc_empty(&self) -> Self::Methods;

    /// Combines two JSON-RPC method collections.
    fn jsonrpc_merge(&self, left: Self::Methods, right: Self::Methods) -> Self::Methods;
}

/// Compiles a typed JSON-RPC method declaration supported by an interpreter.
pub trait JsonRpcMethodAlg<Context, Args, Output> {
    /// Registers a first-order handler decoded from positional parameters.
    ///
    /// The argument names are stated even though positional decoding reads by position, because they
    /// are part of what the method promises: an interpretation that describes or generates a client
    /// can name the parameters a caller passes, whichever way the wire carries them.
    fn finish_jsonrpc_positional_method<Handler>(
        &self,
        name: &'static str,
        arg_names: &'static [&'static str],
        handler: Handler,
    ) -> Self::Methods
    where
        Self: JsonRpcAlg,
        Handler: ApplyAlg<Context, Args, Output = Output> + Send + Sync + 'static;

    /// Registers a first-order handler decoded from named parameters.
    fn finish_jsonrpc_named_method<Handler>(
        &self,
        name: &'static str,
        arg_names: &'static [&'static str],
        handler: Handler,
    ) -> Self::Methods
    where
        Self: JsonRpcAlg,
        Handler: ApplyAlg<Context, Args, Output = Output> + Send + Sync + 'static;
}

/// Names the two halves of an outcome, so a bound can speak of a failure without spelling it.
///
/// A declaration states the conversion; this is what lets an interpretation name the value it answers
/// with and the failure it reports, given only the operation's output.
pub trait OutcomeAlg {
    /// The value a successful outcome carries.
    type Value;
    /// The failure an unsuccessful outcome states.
    type Error;

    /// Reads the outcome as one or the other.
    ///
    /// # Errors
    ///
    /// Answers with `Err` when the outcome is the failure the operation stated.
    fn outcome(self) -> Result<Self::Value, Self::Error>;
}

impl<Value, Error> OutcomeAlg for Result<Value, Error> {
    type Value = Value;
    type Error = Error;

    fn outcome(self) -> Self {
        self
    }
}

/// States what a domain failure denotes on a JSON-RPC surface.
///
/// A domain says this once for its own error type: the code the JSON-RPC specification carries and
/// the message the failure states. Nothing here names a transport library, so a specification can
/// state what its failures mean without depending on whichever interpreter answers the call.
pub trait RpcErrorAlg {
    /// The code the JSON-RPC specification carries for this failure.
    fn rpc_code(&self) -> i32;

    /// The message this failure states.
    fn rpc_message(&self) -> String;
}

/// An error that cannot be constructed converts to nothing, so a method carrying one answers only
/// with its value. This is how a method keeps the value path inside an ext that converts every error.
impl RpcErrorAlg for core::convert::Infallible {
    fn rpc_code(&self) -> i32 {
        match *self {}
    }

    fn rpc_message(&self) -> String {
        match *self {}
    }
}

/// Compiles a typed JSON-RPC method whose error answers as a protocol error.
///
/// A method registered here answers with its value or with a JSON-RPC error, so a domain that states
/// failure in its own vocabulary reaches a caller as a failed call rather than as a successful one
/// carrying an error-shaped value. [`OutcomeAlg`] names the two halves of the output and
/// [`RpcErrorAlg`] says what the failing half denotes.
pub trait JsonRpcFallibleAlg<Context, Args, Output> {
    /// Registers a fallible handler decoded from positional parameters.
    ///
    /// The argument names are stated for the same reason as on [`JsonRpcMethodAlg`].
    fn finish_jsonrpc_positional_fallible<Handler>(
        &self,
        name: &'static str,
        arg_names: &'static [&'static str],
        handler: Handler,
    ) -> Self::Methods
    where
        Self: JsonRpcAlg,
        Handler: ApplyAlg<Context, Args, Output = Output> + Send + Sync + 'static;

    /// Registers a fallible handler decoded from named parameters.
    fn finish_jsonrpc_named_fallible<Handler>(
        &self,
        name: &'static str,
        arg_names: &'static [&'static str],
        handler: Handler,
    ) -> Self::Methods
    where
        Self: JsonRpcAlg,
        Handler: ApplyAlg<Context, Args, Output = Output> + Send + Sync + 'static;
}

/// Interprets a named, defunctionalized JSON-RPC program with `Compiler`.
pub trait JsonRpcProgramAlg<Compiler> {
    /// The method collection produced by `Compiler`.
    type Methods;

    /// Compiles the program through the supplied interpreter.
    fn compile_jsonrpc(self, compiler: &Compiler) -> Self::Methods;
}

trait_set! {
    /// Combines the capabilities required to interpret JSON-RPC programs.
    ///
    /// Method registration is stated per operation signature, so composing a program surface needs
    /// only the empty collection and its merge.
    pub trait JsonRpcApiAlg = JsonRpcAlg;
}

/// Compiles defunctionalized JSON-RPC programs with an interpreter.
#[ext(name = JsonRpcProgramExt)]
pub impl<This> This {
    /// Compiles a named JSON-RPC program with this interpreter.
    fn compile_jsonrpc<Program>(&self, program: Program) -> Program::Methods
    where
        Program: JsonRpcProgramAlg<This>,
    {
        program.compile_jsonrpc(self)
    }
}

#[cfg(test)]
mod tests {
    use super::RpcErrorAlg;
    use core::convert::Infallible;

    #[test]
    fn an_error_that_cannot_be_constructed_states_the_conversion() {
        fn converts<Error: RpcErrorAlg>() {}

        // This is what keeps a total method on the value path inside an ext that converts.
        converts::<Infallible>();
    }
}
