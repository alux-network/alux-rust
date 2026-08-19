//! JSON-RPC program backend for extension defunctionalization.
//!
//! The backend states what a JSON-RPC declaration means: each method handler is reified as a typed
//! operation whose argument product and output become interpreter evidence, and the method surface is
//! compiled through `JsonRpcProgramAlg`. Everything shared with other transports lives in
//! [`crate::lower`] and [`crate::syntax`].

use crate::lower::{LoweredProgram, ProgramBackendAlg, expand_program};
use crate::syntax::lift_operation;
use proc_macro2::TokenStream;
use quote::quote;
use syn::visit_mut::{self, VisitMut};
use syn::{ExprMethodCall, ImplItemFn, Type, parse_quote};

/// Interprets the shared lowering as a JSON-RPC method program.
struct JsonRpcBackend;

/// Finds the method declarations of a JSON-RPC program.
struct Methods<'a>(&'a mut Vec<Type>);

impl VisitMut for Methods<'_> {
    fn visit_expr_method_call_mut(&mut self, call: &mut ExprMethodCall) {
        if call.method == "method"
            && let Some(declaration) = call.args.iter_mut().nth(1)
            && let Some(operation) = lift_operation(declaration)
        {
            self.0.push(operation);
        }
        visit_mut::visit_expr_method_call_mut(self, call);
    }
}

impl ProgramBackendAlg for JsonRpcBackend {
    const NESTED_SUFFIX: &'static str = "_rpc";
    const REJECTED_PARAM: &'static str = "JSON-RPC programs currently support type parameters only";

    fn require_declarations(method: &mut ImplItemFn) {
        let mut operations = Vec::new();
        Methods(&mut operations).visit_block_mut(&mut method.block);
        let where_clause = method.sig.generics.make_where_clause();
        if !operations.is_empty() {
            where_clause.predicates.push(parse_quote!(This: ::alux_ext::HandlerContextAlg<Alg>));
        }
        for operation in operations {
            where_clause.predicates.push(parse_quote! {
                #operation: ::alux_ext::OperationAlg<Context = Alg>
                    + ::alux_ext::ApplyAlg<
                        <This as ::alux_ext::HandlerContextAlg<Alg>>::Handle,
                        <#operation as ::alux_ext::OperationAlg>::Args,
                    >
                    + Send + Sync + 'static
            });
            where_clause.predicates.push(parse_quote! {
                This: ::alux_jsonrpc::JsonRpcMethodAlg<
                    <This as ::alux_ext::HandlerContextAlg<Alg>>::Handle,
                    <#operation as ::alux_ext::OperationAlg>::Args,
                    <#operation as ::alux_ext::ApplyAlg<
                        <This as ::alux_ext::HandlerContextAlg<Alg>>::Handle,
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
    expand_program::<JsonRpcBackend>(attr, item)
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
