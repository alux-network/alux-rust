use crate::{JsonRpcAlg, JsonRpcApiAlg, JsonRpcFallibleAlg, JsonRpcMethodAlg, JsonRpcProgramAlg};
use alux_ext::{ApplyAlg, HandlerContextAlg, OperationAlg};
use core::marker::PhantomData;

/// Compiles a first-order JSON-RPC program with a concrete interpreter.
pub trait CompileJsonRpcProgram<Compiler> {
    /// The method collection produced by `Compiler`.
    type Methods;

    /// Folds the complete first-order program through `compiler`.
    fn compile_jsonrpc_program(self, compiler: &Compiler) -> Self::Methods;
}

/// Represents the empty JSON-RPC program.
#[derive(Debug, Default)]
pub struct Empty;

/// Represents the composition of two JSON-RPC programs.
#[derive(Debug)]
pub struct Merge<Left, Right> {
    left: Left,
    right: Right,
}

/// Includes a separately named JSON-RPC program.
#[derive(Debug)]
pub struct Named<Program>(Program);

/// Selects positional JSON-RPC parameter decoding.
#[derive(Debug, Default)]
pub struct Positional;

/// Selects named JSON-RPC parameter decoding.
#[derive(Debug, Default)]
pub struct NamedParams;

/// Selects positional decoding for a method whose error answers as a protocol error.
#[derive(Debug, Default)]
pub struct FalliblePositional;

/// Selects named decoding for a method whose error answers as a protocol error.
#[derive(Debug, Default)]
pub struct FallibleNamed;

/// Represents one named JSON-RPC method without choosing an interpreter.
#[derive(Debug)]
pub struct Method<Handler, Params = Positional> {
    name: &'static str,
    handler: Handler,
    marker: PhantomData<fn(Params)>,
}

/// Carries a typed operation declaration as first-order data.
#[derive(Debug)]
pub struct Operation<Handler, Params = Positional> {
    handler: Handler,
    marker: PhantomData<fn(Params)>,
}

/// Carries a typed JSON-RPC program during fluent composition.
#[derive(Debug)]
pub struct JsonRpcProgram<Program>(Program);

/// Constructs neutral JSON-RPC programs.
#[derive(Debug, Default)]
pub struct JsonRpcProgramBuilder;

impl JsonRpcProgramBuilder {
    /// Starts an empty, uninterpreted JSON-RPC program.
    pub fn methods(&self) -> JsonRpcProgram<Empty> {
        JsonRpcProgram(Empty)
    }

    /// Wraps a first-order handler operation in a neutral declaration.
    pub fn op<Handler>(&self, handler: Handler) -> Operation<Handler> {
        Operation { handler, marker: PhantomData }
    }

    /// Includes a named JSON-RPC program in another program.
    pub fn program<Program>(&self, program: Program) -> JsonRpcProgram<Named<Program>> {
        JsonRpcProgram(Named(program))
    }
}

impl<Program> JsonRpcProgram<Program> {
    /// Records the composition of two typed JSON-RPC programs.
    pub fn merge<Other>(self, other: JsonRpcProgram<Other>) -> JsonRpcProgram<Merge<Program, Other>> {
        JsonRpcProgram(Merge { left: self.0, right: other.0 })
    }

    /// Registers a typed operation under a JSON-RPC method name.
    pub fn method<Handler, Params>(
        self,
        name: &'static str,
        operation: Operation<Handler, Params>,
    ) -> JsonRpcProgram<Merge<Program, Method<Handler, Params>>> {
        self.merge(JsonRpcProgram(Method { name, handler: operation.handler, marker: PhantomData }))
    }

    /// Removes the fluent wrapper and returns the first-order syntax tree.
    pub fn into_program(self) -> Program {
        self.0
    }
}

impl<Handler, Params> Operation<Handler, Params> {
    /// Reads this declaration under another parameter or failure mode.
    fn retyped<Mode>(self) -> Operation<Handler, Mode> {
        Operation { handler: self.handler, marker: PhantomData }
    }
}

impl<Handler> Operation<Handler, Positional> {
    /// Selects positional parameter decoding for this operation declaration.
    ///
    /// Positional decoding is already the default, so this modifier is useful
    /// when a declaration benefits from making the wire shape explicit.
    #[must_use]
    pub fn positional(self) -> Self {
        self
    }

    /// Selects named parameter decoding using the defunctionalized operation's
    /// source argument names.
    pub fn named(self) -> Operation<Handler, NamedParams> {
        self.retyped()
    }

    /// Converts the operation's error into a JSON-RPC protocol error.
    pub fn fallible(self) -> Operation<Handler, FalliblePositional> {
        self.retyped()
    }
}

impl<Handler> Operation<Handler, NamedParams> {
    /// Converts the operation's error into a JSON-RPC protocol error.
    pub fn fallible(self) -> Operation<Handler, FallibleNamed> {
        self.retyped()
    }
}

impl<Handler> Operation<Handler, FalliblePositional> {
    /// Selects named parameter decoding, keeping the failure mode.
    pub fn named(self) -> Operation<Handler, FallibleNamed> {
        self.retyped()
    }
}

impl<Compiler> CompileJsonRpcProgram<Compiler> for Empty
where
    Compiler: JsonRpcAlg,
{
    type Methods = Compiler::Methods;

    fn compile_jsonrpc_program(self, compiler: &Compiler) -> Self::Methods {
        compiler.jsonrpc_empty()
    }
}

impl<Compiler, Left, Right, Methods> CompileJsonRpcProgram<Compiler> for Merge<Left, Right>
where
    Compiler: JsonRpcAlg<Methods = Methods>,
    Left: CompileJsonRpcProgram<Compiler, Methods = Methods>,
    Right: CompileJsonRpcProgram<Compiler, Methods = Methods>,
{
    type Methods = Methods;

    fn compile_jsonrpc_program(self, compiler: &Compiler) -> Self::Methods {
        compiler
            .jsonrpc_merge(self.left.compile_jsonrpc_program(compiler), self.right.compile_jsonrpc_program(compiler))
    }
}

impl<Compiler, Program> CompileJsonRpcProgram<Compiler> for Named<Program>
where
    Program: JsonRpcProgramAlg<Compiler>,
{
    type Methods = Program::Methods;

    fn compile_jsonrpc_program(self, compiler: &Compiler) -> Self::Methods {
        self.0.compile_jsonrpc(compiler)
    }
}

// One implementation per mode, so each states the capability its registration needs and no mode is
// reachable through another's bounds.
impl<Compiler, Handler, Handle> CompileJsonRpcProgram<Compiler> for Method<Handler, Positional>
where
    Compiler: JsonRpcApiAlg
        + HandlerContextAlg<Handler::Context, Handle = Handle>
        + JsonRpcMethodAlg<Handle, Handler::Args, Handler::Output>,
    Handler: OperationAlg + ApplyAlg<Handle, Handler::Args> + Send + Sync + 'static,
{
    type Methods = Compiler::Methods;

    fn compile_jsonrpc_program(self, compiler: &Compiler) -> Self::Methods {
        compiler.finish_jsonrpc_positional_method(self.name, Handler::ARG_NAMES, self.handler)
    }
}

impl<Compiler, Handler, Handle> CompileJsonRpcProgram<Compiler> for Method<Handler, NamedParams>
where
    Compiler: JsonRpcApiAlg
        + HandlerContextAlg<Handler::Context, Handle = Handle>
        + JsonRpcMethodAlg<Handle, Handler::Args, Handler::Output>,
    Handler: OperationAlg + ApplyAlg<Handle, Handler::Args> + Send + Sync + 'static,
{
    type Methods = Compiler::Methods;

    fn compile_jsonrpc_program(self, compiler: &Compiler) -> Self::Methods {
        compiler.finish_jsonrpc_named_method(self.name, Handler::ARG_NAMES, self.handler)
    }
}

impl<Compiler, Handler, Handle> CompileJsonRpcProgram<Compiler> for Method<Handler, FalliblePositional>
where
    Compiler: JsonRpcApiAlg
        + HandlerContextAlg<Handler::Context, Handle = Handle>
        + JsonRpcFallibleAlg<Handle, Handler::Args, Handler::Output>,
    Handler: OperationAlg + ApplyAlg<Handle, Handler::Args> + Send + Sync + 'static,
{
    type Methods = Compiler::Methods;

    fn compile_jsonrpc_program(self, compiler: &Compiler) -> Self::Methods {
        compiler.finish_jsonrpc_positional_fallible(self.name, Handler::ARG_NAMES, self.handler)
    }
}

impl<Compiler, Handler, Handle> CompileJsonRpcProgram<Compiler> for Method<Handler, FallibleNamed>
where
    Compiler: JsonRpcApiAlg
        + HandlerContextAlg<Handler::Context, Handle = Handle>
        + JsonRpcFallibleAlg<Handle, Handler::Args, Handler::Output>,
    Handler: OperationAlg + ApplyAlg<Handle, Handler::Args> + Send + Sync + 'static,
{
    type Methods = Compiler::Methods;

    fn compile_jsonrpc_program(self, compiler: &Compiler) -> Self::Methods {
        compiler.finish_jsonrpc_named_fallible(self.name, Handler::ARG_NAMES, self.handler)
    }
}
