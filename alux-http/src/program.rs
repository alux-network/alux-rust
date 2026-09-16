use crate::{
    BytesOut, Connect, Delete, EmptyOut, FileOut, Get, HandlerEndpointAlg, Head, HeaderOut, HtmlOut, HttpApiAlg,
    HttpInputAlg, HttpMethodAlg, HttpProgramAlg, HttpRouteAlg, JsonOut, NamedValuesAlg, Options, Patch, Post, Put,
    RedirectOut, ResultOut, RouteAlg, RoutePath, StatusOut, StreamOut, TextOut, Trace, WithAlg,
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
    prefix: RoutePath,
    program: Program,
}

/// Includes a separately named HTTP program in a route program.
#[derive(Debug)]
pub struct Named<Program>(Program);

/// Represents an endpoint without choosing an HTTP interpreter.
#[derive(Debug)]
pub struct Endpoint<Method, Handler, Inputs, Args, Transform> {
    path: RoutePath,
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

/// Marks a form-encoded HTTP request-body input.
pub struct Form<Input>(PhantomData<Input>);

/// Marks an HTTP request body taken as it arrived.
pub struct RawBody<Input>(PhantomData<Input>);

/// Marks an HTTP header input.
pub struct Header<Input>(PhantomData<Input>);

/// Marks an input read from the cookies a caller sent.
pub struct Cookie<Input>(PhantomData<Input>);

/// Marks an input read from a request body arriving as parts.
pub struct Multipart<Input>(PhantomData<Input>);

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

impl<Compiler, Input> InterpretInputsAlg<Compiler> for Form<Input>
where
    Compiler: HttpInputAlg,
{
    type Inputs = Compiler::Form<Input>;
}

impl<Compiler, Input> InterpretInputsAlg<Compiler> for RawBody<Input>
where
    Compiler: HttpInputAlg,
{
    type Inputs = Compiler::RawBody<Input>;
}

impl<Compiler, Input> InterpretInputsAlg<Compiler> for Header<Input>
where
    Compiler: HttpInputAlg,
{
    type Inputs = Compiler::Header<Input>;
}

impl<Compiler, Input> InterpretInputsAlg<Compiler> for Cookie<Input>
where
    Compiler: HttpInputAlg,
{
    type Inputs = Compiler::Cookie<Input>;
}

impl<Compiler, Input> InterpretInputsAlg<Compiler> for Multipart<Input>
where
    Compiler: HttpInputAlg,
{
    type Inputs = Compiler::Multipart<Input>;
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
interpret_inputs!(I1, I2, I3, I4, I5, I6, I7, I8, I9);
interpret_inputs!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10);
interpret_inputs!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11);
interpret_inputs!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11, I12);
interpret_inputs!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11, I12, I13);
interpret_inputs!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11, I12, I13, I14);
interpret_inputs!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11, I12, I13, I14, I15);
interpret_inputs!(I1, I2, I3, I4, I5, I6, I7, I8, I9, I10, I11, I12, I13, I14, I15, I16);

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

macro_rules! output_methods {
    ($($method:ident => $kind:ident, $meaning:literal),+ $(,)?) => {
        $(
            #[doc = concat!("Marks the inferred handler result for ", $meaning, " interpretation.")]
            pub fn $method(self) -> Operation<Handler, Inputs, Args, $kind> {
                self.out()
            }
        )+
    };
}

/// States the nine method declarations on both a program and a standalone operation.
///
/// An author writes the program form; a declaration read by the `http` macro becomes the operation
/// form. Emitting both from one list is what keeps the two spellings of a method in step.
macro_rules! route_methods {
    ($($method:ident => $marker:ident, $label:literal),+ $(,)?) => {
        impl<Program> RouteProgram<Program> {
            $(
                #[doc = concat!("Records a `", $label, "` selector and typed operation at an exact path.")]
                pub fn $method<Handler, Inputs, Args, Transform>(
                    self,
                    path: &str,
                    operation: Operation<Handler, Inputs, Args, Transform>,
                ) -> WithEndpoint<Program, $marker, Handler, Inputs, Args, Transform> {
                    self.method(path, operation)
                }
            )+
        }

        impl<Handler, Inputs, Args, Transform> Operation<Handler, Inputs, Args, Transform> {
            $(
                #[doc = concat!("Declares this operation at an exact path, answered under `", $label, "`.")]
                ///
                /// This is the same thing the declaration of that name on `RouteProgram` states,
                /// for one endpoint standing on its own rather than one inside a composition.
                pub fn $method(self, path: &str) -> Endpoint<$marker, Handler, Inputs, Args, Transform> {
                    self.declare::<$marker>(path)
                }
            )+
        }
    };
}

impl<Handler, Inputs, Args, Transform> Operation<Handler, Inputs, Args, Transform> {
    /// Declares this operation at one path under one method selector, with no program around it.
    ///
    /// A program that states many endpoints composes their routes rather than their types, so it
    /// needs each endpoint on its own. The named declarations below select one method each; this
    /// states the same thing for a method held as a type parameter.
    pub fn declare<Method>(self, path: &str) -> Endpoint<Method, Handler, Inputs, Args, Transform> {
        Endpoint { path: RoutePath::parse(path), handler: self.handler, marker: PhantomData }
    }
}

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
    pub fn query<Input>(self) -> WithInput<Handler, Inputs, Args, Transform, Query<Input>, Input>
    where
        Input: NamedValuesAlg,
    {
        self.with_as::<Query<Input>, Input>()
    }

    /// Records a request-body extractor whose value becomes the next handler argument.
    pub fn body<Input>(self) -> WithInput<Handler, Inputs, Args, Transform, Body<Input>, Input> {
        self.with_as::<Body<Input>, Input>()
    }

    /// Records a form-encoded request-body extractor whose value becomes the next handler argument.
    pub fn form<Input>(self) -> WithInput<Handler, Inputs, Args, Transform, Form<Input>, Input> {
        self.with_as::<Form<Input>, Input>()
    }

    /// Records the request body as it arrived, becoming the next handler argument.
    pub fn raw_body<Input>(self) -> WithInput<Handler, Inputs, Args, Transform, RawBody<Input>, Input> {
        self.with_as::<RawBody<Input>, Input>()
    }

    /// Records an incoming header extractor whose value becomes the next handler argument.
    pub fn in_header<Input>(self) -> WithInput<Handler, Inputs, Args, Transform, Header<Input>, Input>
    where
        Input: NamedValuesAlg,
    {
        self.with_as::<Header<Input>, Input>()
    }

    /// Records a cookie extractor whose value becomes the next handler argument.
    pub fn cookie<Input>(self) -> WithInput<Handler, Inputs, Args, Transform, Cookie<Input>, Input>
    where
        Input: NamedValuesAlg,
    {
        self.with_as::<Cookie<Input>, Input>()
    }

    /// Records a body arriving as parts, read into the next handler argument.
    pub fn multipart<Input>(self) -> WithInput<Handler, Inputs, Args, Transform, Multipart<Input>, Input> {
        self.with_as::<Multipart<Input>, Input>()
    }

    /// Records an authentication extractor whose value becomes the next handler argument.
    pub fn auth<Input>(self) -> WithInput<Handler, Inputs, Args, Transform, Auth<Input>, Input>
    where
        Input: NamedValuesAlg,
    {
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

    output_methods! {
        json     => JsonOut, "JSON",
        file     => FileOut, "streamed-file",
        text     => TextOut, "plain-text",
        html     => HtmlOut, "HTML",
        bytes    => BytesOut, "raw-byte",
        empty    => EmptyOut, "empty",
        redirect => RedirectOut, "redirect",
        stream   => StreamOut, "streamed",
    }

    /// Answers with `CODE` and the body already stated.
    pub fn status<const CODE: u16>(self) -> Operation<Handler, Inputs, Args, StatusOut<Transform, CODE>> {
        self.out()
    }

    /// Answers with an outgoing response header the handler states, beside the body already stated.
    ///
    /// The handler answers with the header's value and the body, so a value an endpoint cannot know
    /// is one the handler still states.
    pub fn out_header<Name>(self) -> Operation<Handler, Inputs, Args, HeaderOut<Transform, Name>> {
        self.out()
    }

    /// Answers with what the handler's failure means when it fails.
    ///
    /// The kind already stated answers the successful result, so `.json().result()` states JSON on
    /// success and the meaning of the failure otherwise.
    pub fn result(self) -> Operation<Handler, Inputs, Args, ResultOut<Transform>> {
        self.out()
    }
}

impl<Program> RouteProgram<Program> {
    /// Reads this program below an HTTP path prefix, with no program around it.
    ///
    /// A program that states many nestings composes their routes rather than their types, so it
    /// needs each nesting on its own. `RouteProgram::nest` states the same thing inside a
    /// composition, and is what an author writes.
    pub fn under(self, prefix: &str) -> Nest<Program> {
        Nest { prefix: RoutePath::parse(prefix), program: self.0 }
    }

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
        self.merge(RouteProgram(Nest { prefix: RoutePath::parse(prefix), program: other.0 }))
    }

    /// Records a method selector and typed operation at an exact path.
    ///
    /// The named declarations below select one method each and are what an author writes; this
    /// states the same thing for a method held as a type parameter.
    pub fn method<Method, Handler, Inputs, Args, Transform>(
        self,
        path: &str,
        operation: Operation<Handler, Inputs, Args, Transform>,
    ) -> WithEndpoint<Program, Method, Handler, Inputs, Args, Transform> {
        self.merge(RouteProgram(Endpoint {
            path: RoutePath::parse(path),
            handler: operation.handler,
            marker: PhantomData,
        }))
    }

    /// Removes the fluent wrapper and returns the first-order syntax tree.
    pub fn into_program(self) -> Program {
        self.0
    }
}

route_methods! {
    get     => Get, "GET",
    post    => Post, "POST",
    put     => Put, "PUT",
    patch   => Patch, "PATCH",
    delete  => Delete, "DELETE",
    head    => Head, "HEAD",
    options => Options, "OPTIONS",
    trace   => Trace, "TRACE",
    connect => Connect, "CONNECT",
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

impl<Compiler, Method, Handler, Inputs, Args, Transform, Handle> CompileRouteProgram<Compiler>
    for Endpoint<Method, Handler, Inputs, Args, Transform>
where
    Compiler: HttpApiAlg
        + HandlerContextAlg<Handler::Context, Handle = Handle>
        + HandlerEndpointAlg<Handle, Inputs::Inputs, Args, Transform, Handler::Output>,
    Method: HttpMethodAlg,
    Handler: OperationAlg + ApplyAlg<Handle, Args> + Send + Sync + 'static,
    Inputs: InterpretInputsAlg<Compiler>,
{
    type Route = Compiler::Route;

    fn compile_route(self, compiler: &Compiler) -> Self::Route {
        let selector = compiler.compose(compiler.http_method(Method::METHOD), compiler.http_path(&self.path));
        let endpoint = compiler.finish_handler(self.handler);

        compiler.precompose(selector, compiler.lift(endpoint))
    }
}
