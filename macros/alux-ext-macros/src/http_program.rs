//! Reifies HTTP declarations and derives what each asks of an interpretation.
//!
//! An endpoint asks for an `HttpOperationAlg` over its operation, input roles, and output kind, so a
//! declaration states only the route algebra and its domain.

use crate::lower::{Chain, LoweredProgram, ProgramBackendAlg, expand_program};
use crate::syntax::{Reified, lift_operation};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, ExprMethodCall, GenericArgument, Ident, ImplItemFn, Type, parse_quote_spanned};

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
    let mut calls = Vec::new();
    let mut current = declaration;
    loop {
        let Expr::MethodCall(call) = current else { return None };
        if call.method == "op" {
            break;
        }
        calls.push(call);
        current = &call.receiver;
    }
    let mut inputs = Vec::new();
    let mut transform = None;
    for call in calls.into_iter().rev() {
        let name = call.method.to_string();
        if let Some(kind) = output_kind(&name) {
            transform = Some(quote!(::alux_http::#kind));
            continue;
        }
        let type_argument = || {
            call.turbofish.as_ref()?.args.iter().find_map(|argument| match argument {
                GenericArgument::Type(ty) => Some(ty.clone()),
                _ => None,
            })
        };
        match name.as_str() {
            "out" => {
                let kind = type_argument()?;
                transform = Some(quote!(#kind));
            }
            "status" => {
                let code = call.turbofish.as_ref()?.args.iter().next()?;
                let inner = transform?;
                transform = Some(quote!(::alux_http::StatusOut<#inner, #code>));
            }
            "out_header" => {
                let header = type_argument()?;
                let inner = transform?;
                transform = Some(quote!(::alux_http::HeaderOut<#inner, #header>));
            }
            "result" => {
                let inner = transform?;
                transform = Some(quote!(::alux_http::ResultOut<#inner>));
            }
            "with" | "path" | "query" | "body" | "form" | "raw_body" | "multipart" | "in_header" | "cookie"
            | "auth" | "context" => {
                inputs.push((call.method.clone(), type_argument()?));
            }
            _ => {}
        }
    }
    Some((inputs, transform?))
}

/// Lifts one endpoint declaration into the requirement it places on an interpreter.
fn lift_route(declaration: &mut Expr) -> Option<(Reified, Vec<InputDeclaration>, TokenStream)> {
    let (inputs, transform) = endpoint_roles(declaration)?;
    let reified = lift_operation(declaration)?;

    Some((reified, inputs, transform))
}

impl ProgramBackendAlg for HttpBackend {
    type Defaults = ();

    const NESTED_SUFFIX: &'static str = "_api";
    const REJECTED_PARAM: &'static str = "HTTP programs currently support type parameters only";

    fn prepare_declarations(method: &mut ImplItemFn, (): &Self::Defaults) {
        let mut requirements = Vec::new();
        Routes(&mut requirements).visit_block_mut(&mut method.block);
        let clause = method.sig.generics.make_where_clause();
        for (Reified { operation, .. }, inputs, kind, written) in requirements {
            let roles = inputs.iter().map(|(role, input)| {
                let marker = match role.to_string().as_str() {
                    "with" => "Direct",
                    "path" => "Path",
                    "query" => "Query",
                    "body" => "Body",
                    "form" => "Form",
                    "raw_body" => "RawBody",
                    "multipart" => "Multipart",
                    "in_header" => "Header",
                    "cookie" => "Cookie",
                    "auth" => "Auth",
                    "context" => "Context",
                    _ => unreachable!(),
                };
                let marker = format_ident!("{marker}");
                quote!(::alux_http::#marker<#input>)
            });
            let roles = quote!((#(#roles,)*));
            clause.predicates.push(parse_quote_spanned! { written =>
                This: ::alux_http::HttpOperationAlg<#operation, #roles, #kind,
                    Endpoint = <This as ::alux_http::RouteAlg>::Endpoint>
            });
        }
    }

    fn require_subprogram(program: &TokenStream) -> TokenStream {
        quote! {
            #program: ::alux_http::HttpProgramAlg<This, Route = <This as ::alux_http::RouteAlg>::Route>
        }
    }

    fn read_link(_call: &ExprMethodCall) -> Option<TokenStream> {
        None
    }

    fn compile_program(lowered: &LoweredProgram) -> TokenStream {
        let LoweredProgram { program_type, compiler_params, predicates, chain } = lowered;
        let Chain { leading, root, .. } = chain;
        quote! {
            impl<This, #compiler_params> ::alux_http::HttpProgramAlg<This> for #program_type
            where
                #(#predicates,)*
            {
                type Route = <This as ::alux_http::RouteAlg>::Route;

                fn compile_http(self, compiler: &This) -> Self::Route {
                    #[allow(unused_imports)]
                    use ::alux_http::{RouteAlgExt as _, HttpOperationExt as _, HttpProgramExt as _};
                    let builder = compiler;
                    #(#leading)*
                    (#root).into_route()
                }
            }
        }
    }
}

/// Expands an HTTP declaration into its program, stating each endpoint's `HttpOperationAlg` bound.
pub(crate) fn http_program_defunc_internal(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    expand_program::<HttpBackend>(attr, item, &())
}

#[cfg(test)]
mod tests {
    use super::http_program_defunc_internal;
    use quote::quote;

    #[test]
    fn preserves_authored_bounds_and_output_expressions() {
        let output = http_program_defunc_internal(
            quote!(name = StatusApiExt),
            quote! {
                impl<This> This where This: HttpRouteAlg {
                    fn status_api<Alg>(&self)
                    where
                        Alg: StatusAlg,
                        This: HttpOperationAlg<StatusCurrentOperation<Alg>, (), LoginOut,
                            Endpoint = <This as RouteAlg>::Endpoint>,
                    {
                        self.routes().get("/status", self.op(Alg::status_current).out::<LoginOut>())
                    }
                }
            },
        )
        .unwrap()
        .to_string();
        assert!(output.contains("struct StatusApiProgram"));
        assert!(output.contains("StatusCurrentOperation < Alg >"));
        assert!(output.contains("out :: < LoginOut >"));
        assert!(output.contains("let builder = compiler"));
        assert!(!output.contains("HttpProgramBuilder"));
        assert!(!output.contains("HandlerContextAlg"));
        assert!(!output.contains("HandlerEndpointAlg"));
        assert!(!output.contains("ApplyAlg"));
    }

    #[test]
    fn states_the_endpoint_and_nested_program_bounds() {
        let output = http_program_defunc_internal(
            quote!(name = RootApiExt),
            quote! {
                impl<This> This where This: HttpRouteAlg {
                    fn root_api<Alg>(&self) {
                        self.routes()
                            .get("/status", self.op(Alg::status_current).json())
                            .merge(self.other_api::<Alg>())
                    }
                }
            },
        )
        .unwrap()
        .to_string();
        // The `.json()` endpoint states its capability, and `other_api`, named by its `_api` suffix,
        // states the program it nests.
        assert!(output.contains(
            "This : :: alux_http :: HttpOperationAlg < StatusCurrentOperation < Alg > , () , :: alux_http :: JsonOut"
        ));
        assert!(output.contains("OtherApiProgram < Alg > : :: alux_http :: HttpProgramAlg < This"));
        assert!(output.contains(". json ()"));
        assert!(output.contains(". program ("));
    }

    #[test]
    fn retains_composed_output_calls_and_inline_bounds() {
        let output = http_program_defunc_internal(
            quote!(name = CustomApiExt),
            quote! {
                impl<This: HttpRouteAlg> This {
                    fn custom_api<Domain: DomainAlg>(&self)
                    where
                        This: CustomEndpointAlg<Domain>,
                    {
                        self.routes().post("/", self.op(Domain::submit)
                            .form::<Params>().out::<CustomOut>().status::<202>().out_header::<Header>())
                    }
                }
            },
        )
        .unwrap()
        .to_string();
        assert!(output.contains("SubmitOperation < Domain >"));
        assert!(output.contains("This : HttpRouteAlg"));
        assert!(output.contains("Domain : DomainAlg"));
        assert!(output.contains("This : CustomEndpointAlg < Domain >"));
        assert!(output.contains(". out :: < CustomOut > () . status :: < 202 > () . out_header :: < Header > ()"));
        assert!(!output.contains("OutputKindAlg"));
        assert!(!output.contains("HandlerEndpointAlg"));
    }

    #[test]
    fn rejects_non_type_program_parameters() {
        let error = http_program_defunc_internal(
            quote!(name = InvalidApiExt),
            quote! { impl<This> This { fn invalid_api<const N: usize>(&self) { self.routes() } } },
        )
        .unwrap_err();
        assert!(error.to_string().contains("type parameters only"));
    }
}
