//! HTTP program backend for extension defunctionalization.
//!
//! The backend states what an HTTP declaration means: each route handler is reified as a typed
//! operation, its input roles and output kind become interpreter evidence, and the route tree is
//! compiled through `HttpProgramAlg`. Everything shared with other transports lives in
//! [`crate::lower`] and [`crate::syntax`].

use crate::lower::{LoweredProgram, ProgramBackendAlg, expand_program};
use crate::syntax::{Reified, lift_operation};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, ExprMethodCall, GenericArgument, Ident, ImplItemFn, Type, parse_quote};

/// Interprets the shared lowering as an HTTP route program.
struct HttpBackend;

/// Carries one route's operation, ordered input roles, and output kind.
type RouteRequirement = (Reified, Vec<InputDeclaration>, Ident);

/// Carries one input role together with the argument type it supplies.
type InputDeclaration = (Ident, Type);

/// Finds the endpoint declarations of a route program.
struct Routes<'a>(&'a mut Vec<RouteRequirement>);

impl VisitMut for Routes<'_> {
    fn visit_expr_method_call_mut(&mut self, call: &mut ExprMethodCall) {
        if matches!(call.method.to_string().as_str(), "get" | "post")
            && let Some(declaration) = call.args.iter_mut().nth(1)
            && let Some(route) = lift_route(declaration)
        {
            self.0.push(route);
        }
        visit_mut::visit_expr_method_call_mut(self, call);
    }
}

/// Reads the input roles and output kind an endpoint declaration selects.
///
/// A declaration without an output kind selects nothing an interpreter could convert, so it denotes
/// no endpoint and is left as authored.
fn endpoint_roles(declaration: &Expr) -> Option<(Vec<InputDeclaration>, Ident)> {
    let mut inputs = Vec::new();
    let mut transform = None;
    let mut current = declaration;
    loop {
        let Expr::MethodCall(call) = current else { return None };
        match call.method.to_string().as_str() {
            "op" => {
                inputs.reverse();
                return Some((inputs, transform?));
            }
            "json" => transform = Some(format_ident!("JsonOut")),
            "file" => transform = Some(format_ident!("FileOut")),
            "with" | "path" | "query" | "body" | "header" | "auth" | "context" => {
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
fn lift_route(declaration: &mut Expr) -> Option<RouteRequirement> {
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
        for (Reified { operation, carrier }, inputs, transform) in requirements {
            let input_types = inputs.iter().map(|(_, input)| input);
            let args = if inputs.is_empty() { quote!(()) } else { quote!((#(#input_types,)*)) };
            let roles = inputs.iter().map(|(role, input)| match role.to_string().as_str() {
                "with" => quote!(#input),
                "path" => quote!(<This as ::alux_http::HttpInputAlg>::Path<#input>),
                "query" => quote!(<This as ::alux_http::HttpInputAlg>::Query<#input>),
                "body" => quote!(<This as ::alux_http::HttpInputAlg>::Body<#input>),
                "header" => quote!(<This as ::alux_http::HttpInputAlg>::Header<#input>),
                "auth" => quote!(<This as ::alux_http::HttpInputAlg>::Auth<#input>),
                "context" => quote!(<This as ::alux_http::HttpInputAlg>::Context<#input>),
                _ => unreachable!(),
            });
            let roles = if inputs.is_empty() { quote!(()) } else { quote!((#(#roles,)*)) };
            where_clause.predicates.push(parse_quote! {
                #operation: ::alux_ext::ApplyAlg<<This as ::alux_ext::HandlerContextAlg<#carrier>>::Handle, #args>
                    + Send + Sync + 'static
            });
            where_clause.predicates.push(parse_quote! {
                This: ::alux_http::HandlerEndpointAlg<
                    <This as ::alux_ext::HandlerContextAlg<#carrier>>::Handle,
                    #roles,
                    #args,
                    ::alux_http::#transform,
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

    fn compile_program(lowered: &LoweredProgram) -> TokenStream {
        let LoweredProgram { program_type, compiler_params, predicates, body } = lowered;
        quote! {
            impl<This, #compiler_params> ::alux_http::HttpProgramAlg<This> for #program_type
            where
                #(#predicates,)*
            {
                type Route = <This as ::alux_http::RouteAlg>::Route;

                fn compile_http(self, compiler: &This) -> Self::Route {
                    let _ = self;
                    let builder = ::alux_http::HttpProgramBuilder;
                    let program = (#body).into_program();

                    ::alux_http::CompileRouteProgram::compile_route(program, compiler)
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
