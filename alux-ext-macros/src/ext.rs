//! Implementation of the `ext` attribute.
//!
//! Expansion preserves the ordinary extension declaration and optionally
//! defunctionalizes its methods. Plain `defunc` reifies borrowed method
//! application through `ApplyAlg`; `defunc(via = backend)` delegates to an
//! attribute-macro backend while retaining the same extension surface.

use crate::http_program::http_program_defunc_internal;
use crate::syntax::{ExtensionImpl, operation_ident};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::visit_mut::VisitMut;
use syn::{
    ExprMethodCall, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Meta, Pat, Path, ReceiverKind, ReturnType, Token,
    Type, Visibility,
};

enum Defunc {
    None,
    Apply,
    Via(Path),
}

/// Separates the DD-specific flag from arguments forwarded to `extend::ext`.
struct ExtArgs {
    forwarded: Vec<Meta>,
    defunc: Defunc,
}

impl Parse for ExtArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let arguments = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        let mut forwarded = Vec::new();
        let mut defunc = Defunc::None;
        for argument in arguments {
            if argument.path().is_ident("defunc") {
                defunc = match argument {
                    Meta::Path(_) => Defunc::Apply,
                    Meta::List(list) => {
                        let mut via = None;
                        list.parse_nested_meta(|meta| {
                            if !meta.path.is_ident("via") {
                                return Err(meta.error("expected `via = path`"));
                            }
                            via = Some(meta.value()?.parse::<Path>()?);
                            Ok(())
                        })?;
                        Defunc::Via(via.ok_or_else(|| syn::Error::new_spanned(list, "expected `via = path`"))?)
                    }
                    Meta::NameValue(argument) => {
                        return Err(syn::Error::new_spanned(argument, "use `defunc` or `defunc(via = path)`"));
                    }
                };
            } else {
                forwarded.push(argument);
            }
        }
        Ok(Self { forwarded, defunc })
    }
}

struct RenameThis;

impl VisitMut for RenameThis {
    fn visit_ident_mut(&mut self, ident: &mut Ident) {
        if ident == "This" {
            *ident = format_ident!("Context");
        }
    }
}

/// Recognizes a block whose methods already construct first-order route programs.
fn is_direct_program(item: &ItemImpl) -> bool {
    struct RouteBody {
        routes: bool,
    }

    impl VisitMut for RouteBody {
        fn visit_expr_method_call_mut(&mut self, call: &mut ExprMethodCall) {
            self.routes |= call.method == "routes";
            syn::visit_mut::visit_expr_method_call_mut(self, call);
        }
    }

    let mut methods = item.items.iter().filter_map(|item| match item {
        ImplItem::Fn(method) => Some(method),
        _ => None,
    });
    let Some(first) = methods.next() else { return false };

    core::iter::once(first).chain(methods).all(|method| {
        if !matches!(method.sig.output, ReturnType::Default) {
            return false;
        }
        let mut method = method.clone();
        let mut body = RouteBody { routes: false };
        body.visit_block_mut(&mut method.block);
        body.routes
    })
}

/// Reifies one borrowed extension method as an operation and its application meaning.
fn defunctionalize(
    method: &ImplItemFn,
    visibility: &Visibility,
    where_clause: Option<&syn::WhereClause>,
) -> syn::Result<TokenStream> {
    let operation = operation_ident(&method.sig.ident);
    let mut output = match &method.sig.output {
        ReturnType::Type(_, output) => output.as_ref().clone(),
        ReturnType::Default => {
            return Err(syn::Error::new_spanned(&method.sig, "defunctionalized method needs a return type"));
        }
    };
    let context = format_ident!("__context");
    RenameThis.visit_type_mut(&mut output);
    let mut predicates = where_clause.map(|clause| clause.predicates.clone()).unwrap_or_default();
    for predicate in &mut predicates {
        RenameThis.visit_where_predicate_mut(predicate);
    }
    let mut arguments = Vec::new();
    let mut argument_types = Vec::<Type>::new();
    let mut mutable_receiver = false;
    for input in &method.sig.inputs {
        match input {
            FnArg::Receiver(receiver) => {
                let ReceiverKind::Reference(_, _, mutability) = &receiver.kind else {
                    return Err(syn::Error::new_spanned(receiver, "defunctionalized methods must borrow `self`"));
                };
                mutable_receiver = mutability.is_some();
            }
            FnArg::Typed(input) => {
                let Pat::Ident(pattern) = input.pat.as_ref() else {
                    return Err(syn::Error::new_spanned(&input.pat, "defunctionalized arguments must be identifiers"));
                };
                arguments.push(pattern.ident.clone());
                argument_types.push(input.ty.as_ref().clone());
            }
        }
    }
    let args = if argument_types.is_empty() { quote!(()) } else { quote!((#(#argument_types,)*)) };
    let destructure =
        if arguments.is_empty() { quote!(let () = __args;) } else { quote!(let (#(#arguments,)*) = __args;) };
    let method_name = &method.sig.ident;
    let context_bound = if mutable_receiver { quote!(AsMut<Context>) } else { quote!(AsRef<Context>) };
    let context_parameter = if mutable_receiver { quote!(mut #context) } else { quote!(#context) };
    let receiver = if mutable_receiver { quote!(#context.as_mut()) } else { quote!(#context.as_ref()) };
    let call = quote!(#receiver.#method_name(#(#arguments),*));
    let application = if method.sig.asyncness.is_some() { quote!(#call.await) } else { call };

    Ok(quote! {
        #[doc(hidden)]
        #visibility struct #operation<Context>(core::marker::PhantomData<fn() -> Context>);

        #[doc(hidden)]
        impl<Context> core::default::Default for #operation<Context> {
            fn default() -> Self {
                Self(core::marker::PhantomData)
            }
        }

        #[doc(hidden)]
        impl<Context> ::alux_ext::OperationAlg for #operation<Context> {
            type Context = Context;
            type Args = #args;

            const ARG_NAMES: &'static [&'static str] = &[#(stringify!(#arguments)),*];
        }

        #[doc(hidden)]
        impl<Context, Handle> ::alux_ext::ApplyAlg<Handle, #args> for #operation<Context>
        where
            Handle: #context_bound + Send,
            Context: Sync,
            #predicates
        {
            type Output = #output;

            fn apply(&self, #context_parameter: Handle, __args: #args) -> impl core::future::Future<Output = Self::Output> + Send {
                async move {
                    #destructure
                    #application
                }
            }
        }
    })
}

/// Expands the facade macro after converting compiler token streams into testable tokens.
pub(crate) fn ext_internal(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let arguments = syn::parse2::<ExtArgs>(attr)?;
    if let Defunc::Via(via) = &arguments.defunc {
        let forwarded = &arguments.forwarded;
        return Ok(quote! {
            #[#via(#(#forwarded),*)]
            #item
        });
    }
    let original = item.clone();
    let input = syn::parse2::<ExtensionImpl>(item)?;
    if matches!(arguments.defunc, Defunc::Apply) && is_direct_program(&input.item) {
        let forwarded = &arguments.forwarded;
        return http_program_defunc_internal(quote!(#(#forwarded),*), original);
    }
    let visibility = input.visibility.clone().unwrap_or(Visibility::Inherited);
    let operations = if matches!(arguments.defunc, Defunc::Apply) {
        let where_clause = input.item.generics.where_clause.clone();
        input
            .item
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(method) => Some(defunctionalize(method, &visibility, where_clause.as_ref())),
                _ => None,
            })
            .collect::<syn::Result<Vec<_>>>()?
    } else {
        Vec::new()
    };
    let forwarded = arguments.forwarded;
    let forwarded = input.forwarded(quote!(#(#forwarded),*));
    let item = input.item;
    Ok(quote! {
        #[::alux_ext::extend::ext(#forwarded)]
        #item
        #(#operations)*
    })
}

#[cfg(test)]
mod tests {
    use super::ext_internal;
    use quote::quote;

    #[test]
    fn preserves_an_ordinary_extension() {
        let output = ext_internal(
            quote!(name = ValueExt),
            quote! {
                impl<This> This {
                    fn value(&self) -> u32 { 1 }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("extend :: ext"));
        assert!(output.contains("name = ValueExt"));
        assert!(!output.contains("ValueOperation"));
    }

    #[test]
    fn defunctionalizes_each_method_into_an_applicable_operation() {
        let output = ext_internal(
            quote!(name = ValueExt, defunc),
            quote! {
                pub impl<This> This
                where
                    This: ValueAlg,
                {
                    async fn doubled(&self, value: u32) -> u32 { value * 2 }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("pub struct DoubledOperation"));
        assert!(output.contains("alux_ext :: OperationAlg"));
        assert!(output.contains("type Args = (u32 ,)"));
        assert!(output.contains("const ARG_NAMES"));
        assert!(output.contains("stringify ! (value)"));
        assert!(output.contains("alux_ext :: ApplyAlg"));
        assert!(output.contains("AsRef < Context >"));
    }

    #[test]
    fn names_an_already_first_order_route_program_without_a_backend_argument() {
        let output = ext_internal(
            quote!(name = DirectApiExt, defunc),
            quote! {
                pub impl<This> This
                where
                    This: HttpApiAlg + JsonOutAlg,
                {
                    fn direct_api<Alg>(&self)
                    where
                        Alg: ExampleAlg,
                    {
                        self.routes().get(
                            "/summary",
                            self.op(ExampleSummaryOperation::<Alg>::default()).json(),
                        )
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("struct DirectApiProgram"));
        assert!(output.contains("alux_http :: HttpProgramAlg"));
        assert!(output.contains("ExampleSummaryOperation :: < Alg > :: default"));
        assert!(!output.contains("struct DirectApiOperation"));
    }

    #[test]
    fn rejects_a_value_for_the_defunc_flag() {
        let error = ext_internal(
            quote!(defunc = true),
            quote!(
                impl<This> This {}
            ),
        )
        .unwrap_err();

        assert!(error.to_string().contains("use `defunc` or `defunc(via = path)`"));
    }

    #[test]
    fn delegates_defunctionalization_to_the_selected_backend() {
        let output = ext_internal(
            quote!(name = StatusRoutesExt, defunc(via = http)),
            quote! {
                impl<This> This
                where
                    This: HttpApiAlg + JsonOutAlg,
                {
                    fn status_routes<Alg>(&self)
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

        assert!(output.contains("http"));
        assert!(output.contains("name = StatusRoutesExt"));
        assert!(output.contains("fn status_routes"));
    }
}
