//! HTTP program backend for extension defunctionalization.
//!
//! The backend states what an HTTP declaration means: each route handler is reified as a typed
//! operation, its input roles and output kind become interpreter evidence, and the route tree is
//! compiled through `HttpProgramAlg`. Everything shared with other transports lives in
//! [`crate::lower`] and [`crate::syntax`].

use crate::lower::{Chain, LoweredProgram, ProgramBackendAlg, expand_program};
use crate::syntax::{Reified, lift_operation};
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, ExprMethodCall, GenericArgument, Ident, ImplItemFn, Type, parse_quote, parse_quote_spanned};

/// Interprets the shared lowering as an HTTP route program.
struct HttpBackend;

/// Names each authored declaration method and the method marker it denotes.
///
/// The table is the whole of what a method contributes to lowering, so a method the specification
/// names is one row here rather than a case in the visitor and another in the reader.
const METHODS: &[(&str, &str)] = &[
    ("get", "Get"),
    ("post", "Post"),
    ("put", "Put"),
    ("patch", "Patch"),
    ("delete", "Delete"),
    ("head", "Head"),
    ("options", "Options"),
    ("trace", "Trace"),
    ("connect", "Connect"),
];

/// Reads a declaration method name as the marker type its endpoint is declared under.
fn method_marker(name: &str) -> Option<Ident> {
    METHODS.iter().find(|(method, _)| *method == name).map(|(_, marker)| format_ident!("{marker}"))
}

/// Names each authored output declaration and the output kind it selects.
const OUTPUTS: &[(&str, &str)] = &[
    ("json", "JsonOut"),
    ("file", "FileOut"),
    ("text", "TextOut"),
    ("html", "HtmlOut"),
    ("bytes", "BytesOut"),
    ("empty", "EmptyOut"),
    ("redirect", "RedirectOut"),
    ("stream", "StreamOut"),
];

/// Reads an output declaration name as the kind it selects.
fn output_kind(name: &str) -> Option<Ident> {
    OUTPUTS.iter().find(|(output, _)| *output == name).map(|(_, kind)| format_ident!("{kind}"))
}

/// Records one output kind stated around the kind an endpoint already selected.
enum OutputWrapper {
    /// States the status the endpoint answers with.
    Status(TokenStream),
    /// States a header the endpoint answers with beside its body.
    ResponseHeader(TokenStream),
    /// States that the handler can fail.
    Result,
}

/// Carries one route's operation, ordered input roles, output kind, and where it was written.
///
/// The span is where the endpoint itself was written, so an obligation it fails to meet names that
/// endpoint rather than the whole declaration. It is deliberately the endpoint rather than the
/// method call around it, because the method call is what an editor resolves and stating a bound at
/// it would hover as the bound instead.
type RouteRequirement = (Reified, Vec<InputDeclaration>, TokenStream, Span);

/// Carries one input role together with the argument type it supplies.
type InputDeclaration = (Ident, Type);

/// Finds the endpoint declarations of a route program.
struct Routes<'a>(&'a mut Vec<RouteRequirement>);

impl VisitMut for Routes<'_> {
    fn visit_expr_method_call_mut(&mut self, call: &mut ExprMethodCall) {
        if method_marker(&call.method.to_string()).is_some()
            && let Some(declaration) = call.args.iter_mut().nth(1)
        {
            let written = declaration.span();
            if let Some((reified, inputs, transform)) = lift_route(declaration) {
                self.0.push((reified, inputs, transform, written));
            }
        }
        visit_mut::visit_expr_method_call_mut(self, call);
    }
}

/// Reads the input roles and output kind an endpoint declaration selects.
///
/// A declaration without an output kind selects nothing an interpreter could convert, so it denotes
/// no endpoint and is left as authored.
fn endpoint_roles(declaration: &Expr) -> Option<(Vec<InputDeclaration>, TokenStream)> {
    let mut inputs = Vec::new();
    let mut transform = None;
    let mut wrappers: Vec<OutputWrapper> = Vec::new();
    let mut current = declaration;
    loop {
        let Expr::MethodCall(call) = current else { return None };
        let name = call.method.to_string();
        if let Some(kind) = output_kind(&name) {
            transform = Some(quote!(::alux_http::#kind));
            current = &call.receiver;
            continue;
        }
        match name.as_str() {
            "op" => {
                inputs.reverse();
                // The declaration was read from the outside in, so the kind it states is wrapped
                // from the inside out.
                let transform = wrappers.iter().rev().fold(transform?, |inner, wrapper| match wrapper {
                    OutputWrapper::Status(code) => quote!(::alux_http::StatusOut<#inner, #code>),
                    OutputWrapper::ResponseHeader(name) => quote!(::alux_http::HeaderOut<#inner, #name>),
                    OutputWrapper::Result => quote!(::alux_http::ResultOut<#inner>),
                });

                return Some((inputs, transform));
            }
            "status" => {
                let arguments = call.turbofish.as_ref()?;
                let code = arguments.args.iter().find_map(|argument| match argument {
                    GenericArgument::Const(code) => Some(code.clone()),
                    _ => None,
                })?;
                wrappers.push(OutputWrapper::Status(quote!(#code)));
            }
            "out_header" => {
                let arguments = call.turbofish.as_ref()?;
                let name = arguments.args.iter().find_map(|argument| match argument {
                    GenericArgument::Type(name) => Some(name.clone()),
                    _ => None,
                })?;
                wrappers.push(OutputWrapper::ResponseHeader(quote!(#name)));
            }
            "result" => wrappers.push(OutputWrapper::Result),
            "with" | "path" | "query" | "body" | "form" | "raw_body" | "multipart" | "in_header" | "cookie"
            | "auth" | "context" => {
                let arguments = call.turbofish.as_ref()?;
                let input = arguments.args.iter().find_map(|argument| match argument {
                    GenericArgument::Type(input) => Some(input.clone()),
                    _ => None,
                })?;
                inputs.push((call.method.clone(), input));
            }
            _ => {}
        }
        current = &call.receiver;
    }
}

/// Lifts one endpoint declaration into the requirement it places on an interpreter.
fn lift_route(declaration: &mut Expr) -> Option<(Reified, Vec<InputDeclaration>, TokenStream)> {
    let (inputs, transform) = endpoint_roles(declaration)?;
    let reified = lift_operation(declaration)?;

    Some((reified, inputs, transform))
}

impl ProgramBackendAlg for HttpBackend {
    /// An HTTP program states nothing once for all of its routes.
    type Defaults = ();

    const NESTED_SUFFIX: &'static str = "_api";
    const REJECTED_PARAM: &'static str = "HTTP programs currently support type parameters only";

    fn require_declarations(method: &mut ImplItemFn, (): &Self::Defaults) {
        let mut requirements = Vec::new();
        Routes(&mut requirements).visit_block_mut(&mut method.block);
        let where_clause = method.sig.generics.make_where_clause();
        // One handle obligation per distinct domain, however many operations name it.
        let mut carriers: Vec<Ident> = Vec::new();
        for reified in &requirements {
            let carrier = &reified.0.carrier;
            if !carriers.contains(carrier) {
                carriers.push(carrier.clone());
            }
        }
        for carrier in carriers {
            where_clause.predicates.push(parse_quote!(This: ::alux_ext::HandlerContextAlg<#carrier>));
        }
        for (Reified { operation, carrier }, inputs, transform, written) in requirements {
            let input_types = inputs.iter().map(|(_, input)| input);
            let args = if inputs.is_empty() { quote!(()) } else { quote!((#(#input_types,)*)) };
            let roles = inputs.iter().map(|(role, input)| match role.to_string().as_str() {
                "with" => quote!(#input),
                "path" => quote!(<This as ::alux_http::HttpInputAlg>::Path<#input>),
                "query" => quote!(<This as ::alux_http::HttpInputAlg>::Query<#input>),
                "body" => quote!(<This as ::alux_http::HttpInputAlg>::Body<#input>),
                "form" => quote!(<This as ::alux_http::HttpInputAlg>::Form<#input>),
                "multipart" => quote!(<This as ::alux_http::HttpInputAlg>::Multipart<#input>),
                "raw_body" => quote!(<This as ::alux_http::HttpInputAlg>::RawBody<#input>),
                "in_header" => quote!(<This as ::alux_http::HttpInputAlg>::Header<#input>),
                "cookie" => quote!(<This as ::alux_http::HttpInputAlg>::Cookie<#input>),
                "auth" => quote!(<This as ::alux_http::HttpInputAlg>::Auth<#input>),
                "context" => quote!(<This as ::alux_http::HttpInputAlg>::Context<#input>),
                _ => unreachable!(),
            });
            let roles = if inputs.is_empty() { quote!(()) } else { quote!((#(#roles,)*)) };
            where_clause.predicates.push(parse_quote_spanned! { written =>
                #operation: ::alux_ext::ApplyAlg<<This as ::alux_ext::HandlerContextAlg<#carrier>>::Handle, #args>
                    + Send + Sync + 'static
            });
            where_clause.predicates.push(parse_quote_spanned! { written =>
                This: ::alux_http::HandlerEndpointAlg<
                    <This as ::alux_ext::HandlerContextAlg<#carrier>>::Handle,
                    #roles,
                    #args,
                    #transform,
                    <#operation as ::alux_ext::ApplyAlg<
                        <This as ::alux_ext::HandlerContextAlg<#carrier>>::Handle,
                        #args,
                    >>::Output
                >
            });
        }
    }

    fn require_subprogram(program: &TokenStream) -> TokenStream {
        quote! {
            #program: ::alux_http::HttpProgramAlg<This, Route = <This as ::alux_http::RouteAlg>::Route>
        }
    }

    fn read_link(call: &ExprMethodCall) -> Option<TokenStream> {
        let name = call.method.to_string();
        let arguments = call.args.iter().collect::<Vec<_>>();
        // A link the declaration writes is read rather than kept, so what replaces it is spanned
        // where it was written. An editor then resolves the authored call to what it denotes.
        let written = call.method.span();
        // A declared endpoint is one endpoint on its own, at the path and method it answers on. It
        // is stated under the same name the declaration wrote, so an editor resolves that name to a
        // declaration of the method it names rather than to the general one it delegates to.
        if let [path, operation] = arguments.as_slice()
            && method_marker(&name).is_some()
        {
            let method = Ident::new(&name, written);

            return Some(quote_spanned!(written => #operation.#method(#path)));
        }

        match (name.as_str(), arguments.as_slice()) {
            // A nested program is one program below a prefix, and a merged one states itself.
            ("nest", [prefix, program]) => Some(quote_spanned!(written => #program.under(#prefix))),
            ("merge", [program]) => Some(quote_spanned!(written => #program.into_program())),
            _ => None,
        }
    }

    fn compile_program(lowered: &LoweredProgram) -> TokenStream {
        let LoweredProgram { program_type, compiler_params, predicates, chain } = lowered;
        let Chain { leading, root, leaves } = chain;
        quote! {
            impl<This, #compiler_params> ::alux_http::HttpProgramAlg<This> for #program_type
            where
                #(#predicates,)*
            {
                type Route = <This as ::alux_http::RouteAlg>::Route;

                fn compile_http(self, compiler: &This) -> Self::Route {
                    let _ = self;
                    let builder = ::alux_http::HttpProgramBuilder;
                    #(#leading)*
                    // Every endpoint joins the one route the interpreter states, so no type here
                    // grows with how many endpoints the declaration has.
                    let route =
                        ::alux_http::CompileRouteProgram::compile_route((#root).into_program(), compiler);
                    #(
                        let route = ::alux_http::RouteAlg::coproduct(
                            compiler,
                            route,
                            ::alux_http::CompileRouteProgram::compile_route(#leaves, compiler),
                        );
                    )*

                    route
                }
            }
        }
    }
}

/// Expands the facade macro after converting compiler token streams into testable tokens.
pub(crate) fn http_program_defunc_internal(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    expand_program::<HttpBackend>(attr, item, &())
}

#[cfg(test)]
mod tests {
    use super::http_program_defunc_internal;
    use quote::quote;

    #[test]
    fn generates_a_program_and_its_interpreter_evidence() {
        let output = http_program_defunc_internal(
            quote!(name = StatusApiExt),
            quote! {
                impl<This> This
                where
                    This: HttpApiAlg + JsonOutAlg,
                {
                    fn status_api<Alg>(&self) -> Routes<'_, This>
                    where
                        Alg: StatusAlg,
                    {
                        self.routes().get("/status", self.op(Alg::status_current).json())
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("struct StatusApiProgram"));
        assert!(output.contains("alux_http :: HttpProgramAlg"));
        assert!(output.contains("StatusCurrentOperation < Alg >"));
        assert!(output.contains("default"));
        assert!(output.contains("CompileRouteProgram :: compile_route"));
        // One route per endpoint, joined into the interpreter's route, and no nested program type.
        assert_eq!(output.matches("coproduct").count(), 1);
        assert_eq!(output.matches(". get (\"/status\")").count(), 1);
        assert!(!output.contains("let program ="), "the endpoints were left as one nested type");
    }

    #[test]
    fn declares_an_endpoint_under_the_marker_its_method_names() {
        let output = http_program_defunc_internal(
            quote!(name = StatusApiExt),
            quote! {
                impl<This> This
                where
                    This: HttpApiAlg + JsonOutAlg,
                {
                    fn status_api<Alg>(&self)
                    where
                        Alg: StatusAlg,
                    {
                        self.routes()
                            .put("/status", self.op(Alg::status_replaced).body::<u32>().json())
                            .patch("/status", self.op(Alg::status_moved).body::<i32>().json())
                            .delete("/status/:id", self.op(Alg::status_cleared).path::<u32>().json())
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        // Every method reaches the same lowering, so each states one endpoint and its evidence.
        for method in ["put", "patch", "delete"] {
            assert_eq!(output.matches(&format!(". {method} (")).count(), 1);
        }
        // One route per endpoint joined into the interpreter's route, however many methods.
        assert_eq!(output.matches("coproduct").count(), 3);
        assert!(output.contains("HttpInputAlg > :: Body < u32 >"));
        assert!(output.contains("HttpInputAlg > :: Path < u32 >"));
    }

    #[test]
    fn states_the_output_a_declaration_wraps_around_the_kind_it_selects() {
        let output = http_program_defunc_internal(
            quote!(name = ReportApiExt),
            quote! {
                impl<This> This
                where
                    This: HttpApiAlg + JsonOutAlg,
                {
                    fn report_api<Alg>(&self)
                    where
                        Alg: ReportAlg,
                    {
                        self.routes()
                            .post("/record", self.op(Alg::report_record).body::<u32>().json().status::<201>())
                            .get("/find", self.op(Alg::report_find).json().result())
                            .delete("/record", self.op(Alg::report_forget).empty())
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        // A declaration is read from the outside in and the kind it states is wrapped inside out.
        assert!(output.contains(":: alux_http :: StatusOut < :: alux_http :: JsonOut , 201 >"));
        assert!(output.contains(":: alux_http :: ResultOut < :: alux_http :: JsonOut >"));
        assert!(output.contains(":: alux_http :: EmptyOut"));
    }

    #[test]
    fn names_the_domain_the_author_named() {
        let output = http_program_defunc_internal(
            quote!(name = StatusApiExt),
            quote! {
                impl<This> This
                where
                    This: HttpApiAlg + JsonOutAlg,
                {
                    fn status_api<Domain>(&self)
                    where
                        Domain: StatusAlg,
                    {
                        self.routes().get("/status", self.op(Domain::status_current).json())
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        // The expansion reuses the authored parameter rather than inventing one.
        assert!(output.contains("StatusCurrentOperation < Domain >"));
        assert!(output.contains("HandlerContextAlg < Domain >"));
        assert!(!output.contains("< Alg >"), "a hardcoded `Alg` leaked into the expansion");
    }

    #[test]
    fn reads_bounds_on_the_generic_parameter_as_a_where_clause() {
        let expand = |item| http_program_defunc_internal(quote!(name = StatusApiExt), item).unwrap().to_string();
        let on_parameter = expand(quote! {
            impl<This: HttpApiAlg + JsonOutAlg> This {
                fn status_api<Alg: StatusAlg>(&self) {
                    self.routes().get("/status", self.op(Alg::status_current).json())
                }
            }
        });
        let in_where_clause = expand(quote! {
            impl<This> This
            where
                This: HttpApiAlg + JsonOutAlg,
            {
                fn status_api<Alg>(&self)
                where
                    Alg: StatusAlg,
                {
                    self.routes().get("/status", self.op(Alg::status_current).json())
                }
            }
        });

        for output in [&on_parameter, &in_where_clause] {
            // Both spellings state the same obligations on the interpretation.
            assert!(output.contains("where This : HttpApiAlg + JsonOutAlg , Alg : StatusAlg ,"));
            // Neither states them on the program type, which needs no algebra to exist.
            assert!(output.contains("struct StatusApiProgram < Alg > (core :: marker :: PhantomData"));
        }
    }

    #[test]
    fn composes_any_method_declared_by_the_same_extension() {
        let output = http_program_defunc_internal(
            quote!(name = RootApiExt),
            quote! {
                impl<This> This
                where
                    This: HttpApiAlg,
                {
                    fn health_routes(&self) -> Routes<'_, This> {
                        self.routes()
                    }

                    fn root_routes(&self) -> Routes<'_, This> {
                        self.routes().nest("/api", self.health_routes())
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("struct HealthRoutesProgram"));
        assert!(output.contains("struct RootRoutesProgram"));
        assert!(output.contains("builder . program (builder . health_routes"));
    }

    #[test]
    fn rejects_non_type_program_parameters() {
        let error = http_program_defunc_internal(
            quote!(name = InvalidApiExt),
            quote! {
                impl<This> This {
                    fn invalid_api<const N: usize>(&self) -> Routes<'_, This> {
                        self.routes()
                    }
                }
            },
        )
        .unwrap_err();

        assert!(error.to_string().contains("type parameters only"));
    }
}
