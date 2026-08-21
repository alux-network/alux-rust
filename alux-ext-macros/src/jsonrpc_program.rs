//! JSON-RPC program backend for extension defunctionalization.
//!
//! The backend states what a JSON-RPC declaration means: each method handler is reified as a typed
//! operation whose argument product and output become interpreter evidence, and the method surface is
//! compiled through `JsonRpcProgramAlg`. Everything shared with other transports lives in
//! [`crate::lower`] and [`crate::syntax`].

use crate::lower::{LoweredProgram, ProgramBackendAlg, expand_program};
use crate::syntax::{Reified, lift_operation};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, ExprMethodCall, Ident, ImplItemFn, Meta, Token, parse_quote};

/// Interprets the shared lowering as a JSON-RPC method program.
struct JsonRpcBackend;

/// What a JSON-RPC program states once about every method it declares.
#[derive(Clone, Copy, Default)]
pub(crate) struct MethodDefaults {
    /// The program converts every method's error, so every declaration is read as fallible.
    fallible: bool,
}

/// Separates the program-level arguments this backend owns from those `extend::ext` reads.
fn split_arguments(attr: TokenStream) -> syn::Result<(MethodDefaults, TokenStream)> {
    if attr.is_empty() {
        return Ok((MethodDefaults::default(), attr));
    }
    let mut defaults = MethodDefaults::default();
    let mut forwarded = Punctuated::<Meta, Token![,]>::new();
    for argument in Punctuated::<Meta, Token![,]>::parse_terminated.parse2(attr)? {
        if matches!(&argument, Meta::Path(path) if path.is_ident("fallible")) {
            defaults.fallible = true;
        } else {
            forwarded.push(argument);
        }
    }

    Ok((defaults, quote!(#forwarded)))
}

/// Carries one method's operation and whether its error answers as a protocol error.
struct MethodRequirement {
    reified: Reified,
    fallible: bool,
}

/// Reads whether a declaration selects the failure mode.
fn is_fallible(declaration: &Expr) -> bool {
    let mut current = declaration;
    loop {
        let Expr::MethodCall(call) = current else { return false };
        if call.method == "fallible" {
            return true;
        }
        current = &call.receiver;
    }
}

/// Finds the method declarations of a JSON-RPC program, reading each under the program's defaults.
struct Methods<'a> {
    requirements: &'a mut Vec<MethodRequirement>,
    defaults: MethodDefaults,
}

impl VisitMut for Methods<'_> {
    fn visit_expr_method_call_mut(&mut self, call: &mut ExprMethodCall) {
        if call.method == "method"
            && let Some(declaration) = call.args.iter_mut().nth(1)
        {
            let declared = is_fallible(declaration);
            if let Some(reified) = lift_operation(declaration) {
                // A program that states the conversion states it for the declarations staying silent.
                if self.defaults.fallible && !declared {
                    let total = declaration.clone();
                    *declaration = parse_quote!(#total.fallible());
                }
                let fallible = declared || self.defaults.fallible;
                self.requirements.push(MethodRequirement { reified, fallible });
            }
        }
        visit_mut::visit_expr_method_call_mut(self, call);
    }
}

impl ProgramBackendAlg for JsonRpcBackend {
    /// A JSON-RPC program states whether its methods' errors answer as protocol errors.
    type Defaults = MethodDefaults;

    const NESTED_SUFFIX: &'static str = "_rpc";
    const REJECTED_PARAM: &'static str = "JSON-RPC programs currently support type parameters only";

    fn require_declarations(method: &mut ImplItemFn, defaults: &Self::Defaults) {
        let mut operations = Vec::new();
        Methods { requirements: &mut operations, defaults: *defaults }.visit_block_mut(&mut method.block);
        let where_clause = method.sig.generics.make_where_clause();
        // One handle obligation per distinct domain, however many operations name it.
        let mut carriers: Vec<Ident> = Vec::new();
        for requirement in &operations {
            let carrier = &requirement.reified.carrier;
            if !carriers.contains(carrier) {
                carriers.push(carrier.clone());
            }
        }
        for carrier in carriers {
            where_clause.predicates.push(parse_quote!(This: ::alux_ext::HandlerContextAlg<#carrier>));
        }
        for MethodRequirement { reified: Reified { operation, carrier }, fallible } in operations {
            where_clause.predicates.push(parse_quote! {
                #operation: ::alux_ext::OperationAlg<Context = #carrier>
                    + ::alux_ext::ApplyAlg<
                        <This as ::alux_ext::HandlerContextAlg<#carrier>>::Handle,
                        <#operation as ::alux_ext::OperationAlg>::Args,
                    >
                    + Send + Sync + 'static
            });
            // A fallible declaration answers with a value or a protocol error, so it needs the
            // registration that can report one; every other method answers with a value only.
            let registration = if fallible {
                quote!(::alux_jsonrpc::JsonRpcFallibleAlg)
            } else {
                quote!(::alux_jsonrpc::JsonRpcMethodAlg)
            };
            where_clause.predicates.push(parse_quote! {
                This: #registration<
                    <This as ::alux_ext::HandlerContextAlg<#carrier>>::Handle,
                    <#operation as ::alux_ext::OperationAlg>::Args,
                    <#operation as ::alux_ext::ApplyAlg<
                        <This as ::alux_ext::HandlerContextAlg<#carrier>>::Handle,
                        <#operation as ::alux_ext::OperationAlg>::Args,
                    >>::Output
                >
            });
        }
    }

    fn require_subprogram(program: &TokenStream) -> TokenStream {
        quote! {
            #program: ::alux_jsonrpc::JsonRpcProgramAlg<This, Methods = <This as ::alux_jsonrpc::JsonRpcAlg>::Methods>
        }
    }

    fn compile_program(lowered: &LoweredProgram) -> TokenStream {
        let LoweredProgram { program_type, compiler_params, predicates, body } = lowered;
        quote! {
            impl<This, #compiler_params> ::alux_jsonrpc::JsonRpcProgramAlg<This> for #program_type
            where
                #(#predicates,)*
            {
                type Methods = <This as ::alux_jsonrpc::JsonRpcAlg>::Methods;

                fn compile_jsonrpc(self, compiler: &This) -> Self::Methods {
                    let _ = self;
                    let builder = ::alux_jsonrpc::JsonRpcProgramBuilder;
                    let program = (#body).into_program();

                    ::alux_jsonrpc::CompileJsonRpcProgram::compile_jsonrpc_program(program, compiler)
                }
            }
        }
    }
}

/// Expands the facade macro after converting compiler token streams into testable tokens.
pub(crate) fn jsonrpc_program_defunc_internal(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let (defaults, forwarded) = split_arguments(attr)?;

    expand_program::<JsonRpcBackend>(forwarded, item, &defaults)
}

#[cfg(test)]
mod tests {
    use super::jsonrpc_program_defunc_internal;
    use quote::quote;

    #[test]
    fn generates_a_composable_jsonrpc_program() {
        let output = jsonrpc_program_defunc_internal(
            quote!(name = StatusRpcExt),
            quote! {
                pub impl<This> This
                where
                    This: JsonRpcApiAlg,
                {
                    fn status_rpc<Alg>(&self)
                    where
                        Alg: StatusAlg,
                    {
                        self.methods()
                            .method("status", self.op(Alg::status_current))
                            .method("status_for", self.op(Alg::status_for_path))
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("struct StatusRpcProgram"));
        assert!(output.contains("alux_jsonrpc :: JsonRpcProgramAlg"));
        assert!(output.contains("StatusCurrentOperation < Alg >"));
        assert!(output.contains("StatusForPathOperation < Alg >"));
        assert!(output.contains("CompileJsonRpcProgram :: compile_jsonrpc_program"));
    }

    #[test]
    fn reads_every_declaration_of_a_fallible_program_as_fallible() {
        let output = jsonrpc_program_defunc_internal(
            quote!(name = StatusRpcExt, fallible),
            quote! {
                pub impl<This> This
                where
                    This: JsonRpcApiAlg,
                {
                    fn status_rpc<Alg>(&self)
                    where
                        Alg: StatusAlg,
                    {
                        self.methods()
                            .method("status", self.op(Alg::status_current))
                            .method("status_for", self.op(Alg::status_for_path).named())
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        // The program said it once, so each silent declaration is read as fallible.
        assert_eq!(output.matches("JsonRpcFallibleAlg").count(), 2);
        assert!(!output.contains("JsonRpcMethodAlg"));
        assert_eq!(output.matches(". fallible ()").count(), 2);
        // The flag is this backend's own argument and reaches no other macro.
        assert!(!output.contains("fallible)"), "the flag leaked into a forwarded attribute");
    }

    #[test]
    fn keeps_a_declaration_that_states_its_own_mode() {
        let output = jsonrpc_program_defunc_internal(
            quote!(name = StatusRpcExt, fallible),
            quote! {
                pub impl<This> This {
                    fn status_rpc<Alg>(&self) {
                        self.methods().method("status", self.op(Alg::status_current).fallible())
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        // Saying it twice means the same as saying it once.
        assert_eq!(output.matches(". fallible ()").count(), 1);
        assert_eq!(output.matches("JsonRpcFallibleAlg").count(), 1);
    }

    #[test]
    fn leaves_a_silent_program_on_the_value_path() {
        let output = jsonrpc_program_defunc_internal(
            quote!(name = StatusRpcExt),
            quote! {
                pub impl<This> This {
                    fn status_rpc<Alg>(&self) {
                        self.methods().method("status", self.op(Alg::status_current))
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("JsonRpcMethodAlg"));
        assert!(!output.contains("fallible"));
    }

    #[test]
    fn composes_named_jsonrpc_programs() {
        let output = jsonrpc_program_defunc_internal(
            quote!(name = ExampleRpcExt),
            quote! {
                pub impl<This> This
                where
                    This: JsonRpcApiAlg,
                {
                    fn example_rpc<Alg>(&self)
                    where
                        Alg: StatusAlg + ItemsAlg,
                    {
                        self.methods().merge(self.status_rpc::<Alg>()).merge(self.items_rpc::<Alg>())
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("StatusRpcProgram < Alg >"));
        assert!(output.contains("ItemsRpcProgram < Alg >"));
        assert!(output.contains("builder . program"));
    }
}
