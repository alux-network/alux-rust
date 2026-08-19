use alux_ext::{ApplyAlg, ext};

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
    fn finish_jsonrpc_positional_method<Handler>(&self, name: &'static str, handler: Handler) -> Self::Methods
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

/// Interprets a named, defunctionalized JSON-RPC program with `Compiler`.
pub trait JsonRpcProgramAlg<Compiler> {
    /// The method collection produced by `Compiler`.
    type Methods;

    /// Compiles the program through the supplied interpreter.
    fn compile_jsonrpc(self, compiler: &Compiler) -> Self::Methods;
}

/// Combines the capabilities required to interpret JSON-RPC programs.
///
/// Method registration is stated per operation signature, so composing a program surface needs only
/// the empty collection and its merge.
pub trait JsonRpcApiAlg: JsonRpcAlg {}

impl<This> JsonRpcApiAlg for This where This: JsonRpcAlg {}

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
