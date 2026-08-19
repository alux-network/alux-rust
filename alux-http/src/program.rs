use crate::{
    FileOut, HandlerEndpointAlg, HttpApiAlg, HttpInputAlg, HttpProgramAlg, HttpRouteAlg, JsonOut, RouteAlg, WithAlg,
};
use alux_ext::{ApplyAlg, HandlerContextAlg, OperationAlg};
use core::marker::PhantomData;

/// Compiles a first-order route program with a concrete interpreter.
pub trait CompileRouteProgram<Compiler> {
    /// The route representation produced by `Compiler`.
    type Route;

    /// Folds the complete first-order program through `compiler`.
    ///
    /// Endpoint construction happens here, after composition has preserved all
    /// handler, input, argument, and output-transform types.
    fn compile_route(self, compiler: &Compiler) -> Self::Route;
}

/// Represents the empty route program.
#[derive(Debug, Default)]
pub struct Empty;

/// Represents the categorical coproduct of two route programs.
#[derive(Debug)]
pub struct Merge<Left, Right> {
    left: Left,
    right: Right,
}

/// Represents a route program nested below an HTTP path prefix.
#[derive(Debug)]
pub struct Nest<Program> {
    prefix: String,
    program: Program,
}

/// Includes a separately named HTTP program in a route program.
#[derive(Debug)]
pub struct Named<Program>(Program);

/// Identifies a GET endpoint declaration.
#[derive(Debug)]
pub struct Get;

/// Identifies a POST endpoint declaration.
#[derive(Debug)]
pub struct Post;

/// Represents an endpoint without choosing an HTTP interpreter.
#[derive(Debug)]
pub struct Endpoint<Method, Handler, Inputs, Args, Transform> {
    path: String,
    handler: Handler,
    marker: PhantomData<fn(Method, Inputs, Args, Transform)>,
}

/// Carries a typed operation declaration as first-order data.
#[derive(Debug)]
pub struct Operation<Handler, Inputs = (), Args = (), Transform = ()> {
    handler: Handler,
    marker: PhantomData<fn(Inputs, Args, Transform)>,
}

/// Carries a typed route program during fluent composition.
#[derive(Debug)]
pub struct RouteProgram<Program>(Program);

/// Constructs neutral HTTP route programs.
#[derive(Debug, Default)]
pub struct HttpProgramBuilder;

/// Marks an input supplied directly by an interpreter.
pub struct Direct<Input>(PhantomData<Input>);

/// Marks an HTTP path input.
pub struct Path<Input>(PhantomData<Input>);

/// Marks an HTTP query input.
pub struct Query<Input>(PhantomData<Input>);

/// Marks an HTTP request-body input.
pub struct Body<Input>(PhantomData<Input>);

/// Marks an HTTP header input.
pub struct Header<Input>(PhantomData<Input>);

/// Marks an HTTP authentication input.
pub struct Auth<Input>(PhantomData<Input>);

/// Marks an endpoint-context input.
pub struct Context<Input>(PhantomData<Input>);

/// Maps neutral input roles to the input types selected by an interpreter.
pub trait InterpretInputsAlg<Compiler> {
    /// The extractor product understood by `Compiler`.
    type Inputs;
}

impl<Compiler> InterpretInputsAlg<Compiler> for () {
    type Inputs = ();
}

impl<Compiler, Input> InterpretInputsAlg<Compiler> for Direct<Input> {
    type Inputs = Input;
}

impl<Compiler, Input> InterpretInputsAlg<Compiler> for Path<Input>
where
    Compiler: HttpInputAlg,
{
    type Inputs = Compiler::Path<Input>;
}

impl<Compiler, Input> InterpretInputsAlg<Compiler> for Query<Input>
where
    Compiler: HttpInputAlg,
{
    type Inputs = Compiler::Query<Input>;
}

impl<Compiler, Input> InterpretInputsAlg<Compiler> for Body<Input>
where
    Compiler: HttpInputAlg,
{
    type Inputs = Compiler::Body<Input>;
}

impl<Compiler, Input> InterpretInputsAlg<Compiler> for Header<Input>
where
    Compiler: HttpInputAlg,
{
    type Inputs = Compiler::Header<Input>;
}

impl<Compiler, Input> InterpretInputsAlg<Compiler> for Auth<Input>
where
    Compiler: HttpInputAlg,
{
    type Inputs = Compiler::Auth<Input>;
}

impl<Compiler, Input> InterpretInputsAlg<Compiler> for Context<Input>
where
    Compiler: HttpInputAlg,
{
    type Inputs = Compiler::Context<Input>;
}

macro_rules! interpret_inputs {
    ($($input:ident),+ $(,)?) => {
        impl<Compiler, $($input),+> InterpretInputsAlg<Compiler> for ($($input,)+)
        where
            $($input: InterpretInputsAlg<Compiler>,)+
        {
            type Inputs = ($($input::Inputs,)+);
        }
    };
}

interpret_inputs!(I1);
interpret_inputs!(I1, I2);
interpret_inputs!(I1, I2, I3);
interpret_inputs!(I1, I2, I3, I4);
interpret_inputs!(I1, I2, I3, I4, I5);
interpret_inputs!(I1, I2, I3, I4, I5, I6);
interpret_inputs!(I1, I2, I3, I4, I5, I6, I7);
interpret_inputs!(I1, I2, I3, I4, I5, I6, I7, I8);

impl HttpProgramBuilder {
    /// Starts an empty, uninterpreted route program.
    ///
    /// Subsequent calls record route syntax without requiring any concrete
    /// framework capabilities.
    pub fn routes(&self) -> RouteProgram<Empty> {
        RouteProgram(Empty)
    }

    /// Wraps a first-order handler operation in a neutral declaration.
    ///
    /// Input roles and an output kind can then be attached while the handler's
    /// result type remains inferred through `ApplyAlg`.
    pub fn op<Handler>(&self, handler: Handler) -> Operation<Handler> {
        Operation { handler, marker: PhantomData }
    }

    /// Includes a named HTTP program as an uninterpreted composition node.
    ///
    /// The named program is compiled by the same interpreter as its enclosing
    /// route program when the complete tree is folded.
    pub fn program<Program>(&self, program: Program) -> RouteProgram<Named<Program>> {
        RouteProgram(Named(program))
    }
}

/// Carries an operation declaration with one additional typed input.
pub type WithInput<Handler, Inputs, Args, Transform, Extractor, Arg> =
    Operation<Handler, <Inputs as WithAlg>::With<Extractor>, <Args as WithAlg>::With<Arg>, Transform>;

/// Carries a route program with one additional typed endpoint.
pub type WithEndpoint<Program, Method, Handler, Inputs, Args, Transform> =
    RouteProgram<Merge<Program, Endpoint<Method, Handler, Inputs, Args, Transform>>>;

impl<Handler, Inputs, Args, Transform> Operation<Handler, Inputs, Args, Transform>
where
    Inputs: WithAlg,
    Args: WithAlg,
{
    // Changes only the declaration's phantom input and argument products while
    // preserving the first-order handler value.
    fn with_as<Input, Arg>(self) -> WithInput<Handler, Inputs, Args, Transform, Input, Arg> {
        Operation { handler: self.handler, marker: PhantomData }
    }

    /// Records an argument supplied directly in the interpreter's input product.
    pub fn with<Input>(self) -> WithInput<Handler, Inputs, Args, Transform, Direct<Input>, Input> {
        self.with_as::<Direct<Input>, Input>()
    }

    /// Records a path extractor whose value becomes the next handler argument.
    pub fn path<Input>(self) -> WithInput<Handler, Inputs, Args, Transform, Path<Input>, Input> {
        self.with_as::<Path<Input>, Input>()
    }

    /// Records a query extractor whose value becomes the next handler argument.
    pub fn query<Input>(self) -> WithInput<Handler, Inputs, Args, Transform, Query<Input>, Input> {
        self.with_as::<Query<Input>, Input>()
    }

    /// Records a request-body extractor whose value becomes the next handler argument.
    pub fn body<Input>(self) -> WithInput<Handler, Inputs, Args, Transform, Body<Input>, Input> {
        self.with_as::<Body<Input>, Input>()
    }

    /// Records a header extractor whose value becomes the next handler argument.
    pub fn header<Input>(self) -> WithInput<Handler, Inputs, Args, Transform, Header<Input>, Input> {
        self.with_as::<Header<Input>, Input>()
    }

    /// Records an authentication extractor whose value becomes the next handler argument.
    pub fn auth<Input>(self) -> WithInput<Handler, Inputs, Args, Transform, Auth<Input>, Input> {
        self.with_as::<Auth<Input>, Input>()
    }

    /// Records an endpoint-context extractor whose value becomes the next handler argument.
    pub fn context<Input>(self) -> WithInput<Handler, Inputs, Args, Transform, Context<Input>, Input> {
        self.with_as::<Context<Input>, Input>()
    }

    /// Replaces the declaration's output-kind marker without converting a value.
    ///
    /// The selected kind is interpreted only after the handler result type is
    /// known at the compilation boundary.
    pub fn out<NewTransform>(self) -> Operation<Handler, Inputs, Args, NewTransform> {
        Operation { handler: self.handler, marker: PhantomData }
    }

    /// Marks the inferred handler result for JSON interpretation.
    pub fn json(self) -> Operation<Handler, Inputs, Args, JsonOut> {
        self.out()
    }

    /// Marks the inferred handler result for streamed-file interpretation.
    pub fn file(self) -> Operation<Handler, Inputs, Args, FileOut> {
        self.out()
    }
}

impl<Program> RouteProgram<Program> {
    /// Records the categorical coproduct of two typed route programs.
    ///
    /// Neither side is interpreted, so both complete program types remain
    /// available to later folds.
    pub fn merge<Other>(self, other: RouteProgram<Other>) -> RouteProgram<Merge<Program, Other>> {
        RouteProgram(Merge { left: self.0, right: other.0 })
    }

    /// Records `other` under `prefix` and merges it into this program.
    ///
    /// The prefix is selector precomposition rather than a framework-specific
    /// router operation.
    pub fn nest<Other>(self, prefix: &str, other: RouteProgram<Other>) -> RouteProgram<Merge<Program, Nest<Other>>> {
        self.merge(RouteProgram(Nest { prefix: prefix.into(), program: other.0 }))
    }

    /// Records a GET selector and typed operation at an exact path.
    pub fn get<Handler, Inputs, Args, Transform>(
        self,
        path: &str,
        operation: Operation<Handler, Inputs, Args, Transform>,
    ) -> WithEndpoint<Program, Get, Handler, Inputs, Args, Transform> {
        self.merge(RouteProgram(Endpoint { path: path.into(), handler: operation.handler, marker: PhantomData }))
    }

    /// Records a POST selector and typed operation at an exact path.
    pub fn post<Handler, Inputs, Args, Transform>(
        self,
        path: &str,
        operation: Operation<Handler, Inputs, Args, Transform>,
    ) -> WithEndpoint<Program, Post, Handler, Inputs, Args, Transform> {
        self.merge(RouteProgram(Endpoint { path: path.into(), handler: operation.handler, marker: PhantomData }))
    }

    /// Removes the fluent wrapper and returns the first-order syntax tree.
    pub fn into_program(self) -> Program {
        self.0
    }
}

impl<Compiler> CompileRouteProgram<Compiler> for Empty
where
    Compiler: RouteAlg,
{
    type Route = Compiler::Route;

    fn compile_route(self, compiler: &Compiler) -> Self::Route {
        compiler.initial()
    }
}

impl<Compiler, Left, Right, Route> CompileRouteProgram<Compiler> for Merge<Left, Right>
where
    Compiler: RouteAlg<Route = Route>,
    Left: CompileRouteProgram<Compiler, Route = Route>,
    Right: CompileRouteProgram<Compiler, Route = Route>,
{
    type Route = Route;

    fn compile_route(self, compiler: &Compiler) -> Route {
        compiler.coproduct(self.left.compile_route(compiler), self.right.compile_route(compiler))
    }
}

impl<Compiler, Program, Route> CompileRouteProgram<Compiler> for Nest<Program>
where
    Compiler: HttpRouteAlg<Route = Route>,
    Program: CompileRouteProgram<Compiler, Route = Route>,
{
    type Route = Route;

    fn compile_route(self, compiler: &Compiler) -> Route {
        compiler.precompose(compiler.http_prefix(&self.prefix), self.program.compile_route(compiler))
    }
}

impl<Compiler, Program> CompileRouteProgram<Compiler> for Named<Program>
where
    Program: HttpProgramAlg<Compiler>,
{
    type Route = Program::Route;

    fn compile_route(self, compiler: &Compiler) -> Self::Route {
        self.0.compile_http(compiler)
    }
}

trait HttpMethodAlg<Compiler> {
    fn selector(compiler: &Compiler) -> <Compiler as RouteAlg>::Selector
    where
        Compiler: HttpApiAlg;
}

impl<Compiler> HttpMethodAlg<Compiler> for Get {
    fn selector(compiler: &Compiler) -> <Compiler as RouteAlg>::Selector
    where
        Compiler: HttpApiAlg,
    {
        compiler.http_get()
    }
}

impl<Compiler> HttpMethodAlg<Compiler> for Post {
    fn selector(compiler: &Compiler) -> <Compiler as RouteAlg>::Selector
    where
        Compiler: HttpApiAlg,
    {
        compiler.http_post()
    }
}

impl<Compiler, Method, Handler, Inputs, Args, Transform, Handle> CompileRouteProgram<Compiler>
    for Endpoint<Method, Handler, Inputs, Args, Transform>
where
    Compiler: HttpApiAlg
        + HandlerContextAlg<Handler::Context, Handle = Handle>
        + HandlerEndpointAlg<Handle, Inputs::Inputs, Args, Transform, Handler::Output>,
    Method: HttpMethodAlg<Compiler>,
    Handler: OperationAlg + ApplyAlg<Handle, Args> + Send + Sync + 'static,
    Inputs: InterpretInputsAlg<Compiler>,
{
    type Route = Compiler::Route;

    fn compile_route(self, compiler: &Compiler) -> Self::Route {
        let selector = compiler.compose(Method::selector(compiler), compiler.http_path(&self.path));
        let endpoint = compiler.finish_handler(self.handler);

        compiler.precompose(selector, compiler.lift(endpoint))
    }
}
