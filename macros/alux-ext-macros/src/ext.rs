//! Implementation of the `ext` attribute.
//!
//! Expansion preserves the ordinary extension declaration and optionally
//! defunctionalizes its methods. Plain `defunc` reifies borrowed method
//! application through `ApplyAlg`; `defunc(via = backend)` delegates to an
//! attribute-macro backend while retaining the same extension surface.

use crate::extension::{extension, stated};
use crate::http_program::http_program_defunc_internal;
use crate::syntax::{ExtensionImpl, arg_names_ident, doc_text, documentation, operation_ident};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::visit_mut::VisitMut;
use syn::{
    ExprMethodCall, FnArg, GenericParam, Generics, Ident, ImplItem, ImplItemFn, ItemImpl, Meta, Pat, Path,
    ReceiverKind, ReturnType, Token, Type, Visibility, WherePredicate, parse_quote,
};

enum Defunc {
    None,
    Apply,
    /// Every backend the block is lowered through, in authored order.
    Via(Vec<Path>),
}

/// Separates the DD-specific flag from the arguments the extension itself reads.
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
                        let mut via = Vec::new();
                        list.parse_nested_meta(|meta| {
                            if !meta.path.is_ident("via") {
                                return Err(meta.error("expected `via = path`"));
                            }
                            via.push(meta.value()?.parse::<Path>()?);
                            Ok(())
                        })?;
                        if via.is_empty() {
                            return Err(syn::Error::new_spanned(list, "expected `via = path`"));
                        }
                        Defunc::Via(via)
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

/// Names every elided lifetime of an argument type with the operation's borrow lifetime.
struct NameBorrow {
    lifetime: syn::Lifetime,
    named: bool,
    /// The types borrowed for the lifetime, each of which must outlive it.
    referents: Vec<Type>,
}

impl VisitMut for NameBorrow {
    fn visit_type_reference_mut(&mut self, reference: &mut syn::TypeReference) {
        if reference.lifetime.as_ref().is_none_or(|lifetime| lifetime.ident == "_") {
            reference.lifetime = Some(self.lifetime.clone());
            self.named = true;
        }
        syn::visit_mut::visit_type_reference_mut(self, reference);
        if reference.lifetime.as_ref() == Some(&self.lifetime) {
            self.referents.push(reference.elem.as_ref().clone());
        }
    }

    fn visit_lifetime_mut(&mut self, lifetime: &mut syn::Lifetime) {
        if lifetime.ident == "_" {
            *lifetime = self.lifetime.clone();
            self.named = true;
        }
    }
}

/// States the borrow lifetime as `'static`, the shape a signature names its borrows with.
struct StaticBorrow<'a>(&'a syn::Lifetime);

impl VisitMut for StaticBorrow<'_> {
    fn visit_lifetime_mut(&mut self, lifetime: &mut syn::Lifetime) {
        if lifetime == self.0 {
            *lifetime = syn::Lifetime::new("'static", lifetime.span());
        }
    }
}

/// Says whether a type borrows, which an output cannot: the handle it would borrow from is
/// consumed by the application that answers it.
fn borrows(ty: &Type) -> bool {
    struct Borrows(bool);
    impl<'ast> syn::visit::Visit<'ast> for Borrows {
        fn visit_type_reference(&mut self, reference: &'ast syn::TypeReference) {
            self.0 |= reference.lifetime.as_ref().is_none_or(|lifetime| lifetime.ident != "static");
            syn::visit::visit_type_reference(self, reference);
        }
        fn visit_lifetime(&mut self, lifetime: &'ast syn::Lifetime) {
            self.0 |= lifetime.ident != "static";
        }
    }
    let mut visitor = Borrows(false);
    syn::visit::Visit::visit_type(&mut visitor, ty);
    visitor.0
}

/// Says whether tokens name the context, which the renamed `This` of a signature is.
fn mentions_context(tokens: TokenStream) -> bool {
    tokens.into_iter().any(|token| match token {
        proc_macro2::TokenTree::Ident(ident) => ident == "Context",
        proc_macro2::TokenTree::Group(group) => mentions_context(group.stream()),
        _ => false,
    })
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

/// Reads the type parameters a method selects, and the bounds it states on them.
///
/// They are selected where the operation is named, so they are parameters of the operation type,
/// and their bounds belong to both of its implementations.
fn selected_parameters(method: &ImplItemFn) -> syn::Result<(Vec<Ident>, Punctuated<WherePredicate, Token![,]>)> {
    let mut selected = Vec::new();
    let mut method_predicates = Punctuated::<WherePredicate, Token![,]>::new();
    for param in &method.sig.generics.params {
        let GenericParam::Type(param) = param else {
            return Err(syn::Error::new_spanned(
                param,
                "a defunctionalized method selects type parameters only; a lifetime or const parameter has \
                 no first-order operation to select",
            ));
        };
        let ident = &param.ident;
        selected.push(ident.clone());
        if !param.bounds.is_empty() {
            let mut bounds = param.bounds.clone();
            for bound in &mut bounds {
                RenameThis.visit_type_param_bound_mut(bound);
            }
            method_predicates.push(parse_quote!(#ident: #bounds));
        }
    }
    if let Some(clause) = &method.sig.generics.where_clause {
        for predicate in &clause.predicates {
            let mut predicate = predicate.clone();
            RenameThis.visit_where_predicate_mut(&mut predicate);
            method_predicates.push(predicate);
        }
    }
    Ok((selected, method_predicates))
}

/// Reads a method's arguments, their types with `This` as the context and every elided lifetime
/// named as the operation's borrow, and whether its receiver is mutable.
fn signature_arguments(method: &ImplItemFn, borrow: &mut NameBorrow) -> syn::Result<(Vec<Ident>, Vec<Type>, bool)> {
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
                let mut argument_type = input.ty.as_ref().clone();
                RenameThis.visit_type_mut(&mut argument_type);
                borrow.visit_type_mut(&mut argument_type);
                argument_types.push(argument_type);
            }
        }
    }
    Ok((arguments, argument_types, mutable_receiver))
}

/// Chooses the handle an application takes, how it reaches the receiver, and what it requires.
///
/// An owned output is answered to any handle that lends the context; a borrowed output only to a
/// borrow of the context for the output's lifetime.
fn handle(
    context: &Ident,
    lifetime_ref: &syn::Lifetime,
    lends_output: bool,
    mutable_receiver: bool,
) -> (TokenStream, TokenStream, Option<TokenStream>, TokenStream, TokenStream) {
    // `Borrow` reaches the context through `&`, `&mut`, `Arc`, `Box`, or the context itself, sized or not.
    let context_bound =
        if mutable_receiver { quote!(core::borrow::BorrowMut<Context>) } else { quote!(core::borrow::Borrow<Context>) };
    let context_parameter = if mutable_receiver && !lends_output { quote!(mut #context) } else { quote!(#context) };
    let receiver = match (lends_output, mutable_receiver) {
        (true, _) => quote!(#context),
        (false, true) => quote!(<Handle as core::borrow::BorrowMut<Context>>::borrow_mut(&mut #context)),
        (false, false) => quote!(<Handle as core::borrow::Borrow<Context>>::borrow(&#context)),
    };
    let (handle_param, handle, handle_bounds) = match (lends_output, mutable_receiver) {
        (true, false) => (None, quote!(&#lifetime_ref Context), quote!(Context: Sync,)),
        (true, true) => (None, quote!(&#lifetime_ref mut Context), quote!(Context: Send,)),
        (false, _) => (Some(quote!(Handle,)), quote!(Handle), quote!(Handle: #context_bound + Send, Context: Sync,)),
    };
    (context_parameter, receiver, handle_param, handle, handle_bounds)
}

/// Reads the parameters a block names besides `This`, its bounds with `This` as the context, and
/// whether it admits an unsized context.
fn block_parameters(generics: &Generics) -> (Vec<GenericParam>, Vec<WherePredicate>, bool) {
    // The parameters the block names besides its receiver are fixed by the context through its
    // bounds, so the operation keeps them on its implementations and not on its type.
    let named = generics
        .params
        .iter()
        .filter(|param| !matches!(param, GenericParam::Type(param) if param.ident == "This"))
        .cloned()
        .map(|mut param| {
            RenameThis.visit_generic_param_mut(&mut param);
            param
        })
        .collect::<Vec<_>>();
    let mut predicates = generics.where_clause.as_ref().map(|clause| clause.predicates.clone()).unwrap_or_default();
    for predicate in &mut predicates {
        RenameThis.visit_where_predicate_mut(predicate);
    }
    // A block that admits an unsized receiver admits an unsized context, on every item generated.
    let unsized_context = predicates.iter().any(|predicate| {
        let predicate = quote!(#predicate).to_string();
        predicate.starts_with("Context :") && predicate.contains("? Sized")
    });
    // The relaxation is stated once, on the parameter, so the bounds keep only what they require.
    let predicates = predicates
        .into_iter()
        .filter_map(|mut predicate| {
            if let WherePredicate::Type(bound) = &mut predicate {
                bound.bounds =
                    bound.bounds.iter().filter(|bound| quote!(#bound).to_string() != "? Sized").cloned().collect();
                if bound.bounds.is_empty() {
                    return None;
                }
            }
            Some(predicate)
        })
        .collect::<Vec<_>>();
    (named, predicates, unsized_context)
}

/// States a method's argument names and stated types on an item no context changes, so a program
/// can check the sources it binds where it is declared.
fn arg_names_item(
    method: &ImplItemFn,
    visibility: &Visibility,
    generics: &Generics,
    arguments: &[Ident],
) -> TokenStream {
    let arg_names = arg_names_ident(&method.sig.ident);
    let stated_args = stated_arg_types(method, generics);
    quote! {
        /// Carries the method's argument names, which no context or selected type changes, so a
        /// program can check names it binds where it is declared.
        #[doc(hidden)]
        #visibility struct #arg_names;

        #[doc(hidden)]
        impl #arg_names {
            /// The method's argument names, in order.
            pub const ARG_NAMES: &'static [&'static str] = &[#(stringify!(#arguments)),*];
            #(#stated_args)*
        }

    }
}

/// States each argument's type on a constant `ARG_<index>`, or that the context decides it.
///
/// A type naming the carrier, a type parameter, or a lifetime depends on what the operation is
/// applied to, so only an interpretation can check a source against it.
fn stated_arg_types(method: &ImplItemFn, generics: &Generics) -> Vec<TokenStream> {
    let parameters = generics
        .params
        .iter()
        .chain(&method.sig.generics.params)
        .filter_map(|param| match param {
            GenericParam::Type(param) => Some(param.ident.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    method
        .sig
        .inputs
        .iter()
        .filter_map(|input| match input {
            FnArg::Typed(input) => Some(input.ty.as_ref()),
            FnArg::Receiver(_) => None,
        })
        .enumerate()
        .map(|(index, argument_type)| {
            let constant = format_ident!("ARG_{index}");
            let mut dependence = ContextDependence { parameters: &parameters, dependent: false };
            dependence.visit_type(argument_type);
            if dependence.dependent {
                quote!(pub const #constant: ::alux_ext::ContextArg = ::alux_ext::ContextArg;)
            } else {
                quote!(pub const #constant: ::alux_ext::StatedArg<#argument_type> = ::alux_ext::StatedArg::new();)
            }
        })
        .collect()
}

/// Finds whether a type names the carrier, a type parameter, a lifetime, or an opaque type.
struct ContextDependence<'a> {
    parameters: &'a [Ident],
    dependent: bool,
}

impl<'ast> Visit<'ast> for ContextDependence<'_> {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        self.dependent |= ident == "This" || ident == "Self" || self.parameters.contains(ident);
    }

    fn visit_lifetime(&mut self, _: &'ast syn::Lifetime) {
        self.dependent = true;
    }

    fn visit_type_reference(&mut self, _: &'ast syn::TypeReference) {
        self.dependent = true;
    }

    fn visit_type_impl_trait(&mut self, _: &'ast syn::TypeImplTrait) {
        self.dependent = true;
    }

    fn visit_type_infer(&mut self, _: &'ast syn::TypeInfer) {
        self.dependent = true;
    }
}

/// Reifies one borrowed extension method as an operation and its application meaning.
fn defunctionalize(method: &ImplItemFn, visibility: &Visibility, generics: &Generics) -> syn::Result<TokenStream> {
    let operation = operation_ident(&method.sig.ident);
    // A method that writes no return type returns nothing, which is a result like any other: it is
    // what an endpoint stating no body answers with.
    let mut output = match &method.sig.output {
        ReturnType::Type(_, output) => output.as_ref().clone(),
        ReturnType::Default => parse_quote!(()),
    };
    let context = format_ident!("__context");
    RenameThis.visit_type_mut(&mut output);
    let (named, predicates, unsized_context) = block_parameters(generics);
    let context_param = if unsized_context { quote!(Context: ?Sized) } else { quote!(Context) };
    let (selected, method_predicates) = selected_parameters(method)?;
    // Borrowed arguments share one lifetime, a parameter of the operation: an application lends
    // them for as long as its future runs.
    let mut borrow = NameBorrow {
        lifetime: syn::Lifetime::new("'__alux_borrow", proc_macro2::Span::call_site()),
        named: false,
        referents: Vec::new(),
    };
    // An output that borrows borrows the context, so it is answered only to a handle that is itself
    // a borrow of the context for the same lifetime, never to an owned handle the application drops.
    let lends_output = borrows(&output);
    borrow.visit_type_mut(&mut output);
    let (arguments, argument_types, mutable_receiver) = signature_arguments(method, &mut borrow)?;
    let args = if argument_types.is_empty() { quote!(()) } else { quote!((#(#argument_types,)*)) };
    // The borrow lifetime belongs to the application, which is stated for every lifetime the
    // arguments are lent for; the operation type names no lifetime, so every program names it alike.
    let lifetimes = borrow.named.then(|| {
        let lifetime = &borrow.lifetime;
        quote!(#lifetime,)
    });
    // The signature states each borrow at `'static`: its shape, independent of any one loan.
    let mut signature_types = argument_types.clone();
    for argument_type in &mut signature_types {
        StaticBorrow(&borrow.lifetime).visit_type_mut(argument_type);
    }
    let signature_args = if signature_types.is_empty() { quote!(()) } else { quote!((#(#signature_types,)*)) };
    // The context is named only behind a reference, which keeps the operation `Send` and `Sync`
    // and admits an unsized context.
    let phantom = quote!(core::marker::PhantomData<(fn(&Context), fn() -> (#(#selected),*))>);
    let lifetime_ref = &borrow.lifetime;
    let outlives = borrow.referents.iter().map(|referent| quote!(#referent: #lifetime_ref)).collect::<Vec<_>>();
    let static_referents = borrow
        .referents
        .iter()
        .map(|referent| {
            let mut referent = referent.clone();
            StaticBorrow(&borrow.lifetime).visit_type_mut(&mut referent);
            quote!(#referent: 'static)
        })
        .collect::<Vec<_>>();
    // A signature names the context's sorts only through the block's bounds, so it keeps them
    // wherever it mentions the context; one that does not states itself for every context.
    let signature = quote!(#args #output #method_predicates);
    let kept = !named.is_empty() || mentions_context(signature);
    let signature_predicates = predicates.iter().filter(|_| kept).chain(&method_predicates).collect::<Vec<_>>();
    let application_predicates = predicates.iter().chain(&method_predicates).collect::<Vec<_>>();
    let destructure =
        if arguments.is_empty() { quote!(let () = __args;) } else { quote!(let (#(#arguments,)*) = __args;) };
    let method_name = &method.sig.ident;
    let (context_parameter, receiver, handle_param, handle, handle_bounds) =
        handle(&context, lifetime_ref, lends_output, mutable_receiver);
    let call = quote!(#receiver.#method_name(#(#arguments),*));
    let application = if method.sig.asyncness.is_some() { quote!(#call.await) } else { call };
    let documentation = documentation(&method.attrs);
    let doc = doc_text(&method.attrs);
    let arg_names_item = arg_names_item(method, visibility, generics, &arguments);

    Ok(quote! {
        #(#documentation)*
        #[doc(hidden)]
        #visibility struct #operation<Context: ?Sized #(, #selected)*>(#phantom);

        #arg_names_item

        #[doc(hidden)]
        impl<Context: ?Sized #(, #selected)*> core::default::Default for #operation<Context #(, #selected)*> {
            fn default() -> Self {
                Self(core::marker::PhantomData)
            }
        }

        #[doc(hidden)]
        impl<#context_param, #(#selected,)* #(#named),*> ::alux_ext::OperationAlg for #operation<Context #(, #selected)*>
        where
            #(#signature_predicates,)*
            #(#static_referents,)*
        {
            type Context = Context;
            type Args = #signature_args;

            const DOC: &'static str = #doc;
            const NAME: &'static str = stringify!(#method_name);
            const ARG_NAMES: &'static [&'static str] = &[#(stringify!(#arguments)),*];
        }

        #[doc(hidden)]
        impl<#lifetimes #context_param, #handle_param #(#selected,)* #(#named),*> ::alux_ext::ApplyAlg<#handle, #args> for #operation<Context #(, #selected)*>
        where
            #handle_bounds
            // The application's future owns each argument until it runs, so it is `Send` where they are.
            #(#argument_types: Send,)*
            #(#application_predicates,)*
            #(#outlives,)*
        {
            type Output = #output;

            fn apply(&self, #context_parameter: #handle, __args: #args) -> impl core::future::Future<Output = Self::Output> + Send {
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
        // The backend lowers the methods first and then generates the trait and the impl the same way.
        let forwarded = &arguments.forwarded;
        // One backend lowers the whole block. Several are cumulative: each lowers the declarations
        // written in its own vocabulary, and the last one states the extension.
        return Ok(match via.as_slice() {
            [only] => quote! {
                #[#only(#(#forwarded),*)]
                #item
            },
            [first, rest @ ..] => quote! {
                #[#first(#(#forwarded,)* __alux_then(#(#rest),*))]
                #item
            },
            [] => unreachable!("`defunc(...)` states at least one backend"),
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
        let generics = &input.item.generics;
        input
            .item
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Fn(method) => Some(defunctionalize(method, &visibility, generics)),
                _ => None,
            })
            .collect::<syn::Result<Vec<_>>>()?
    } else {
        Vec::new()
    };
    let extension = extension(&visibility, stated(&arguments.forwarded, &input.item)?, &input.item)?;

    Ok(quote! {
        #extension
        #(#operations)*
    })
}

#[cfg(test)]
mod tests {
    use super::ext_internal;
    use quote::quote;

    #[test]
    fn keeps_self_as_the_block_wrote_it() {
        let output = ext_internal(
            quote!(name = ValueExt),
            quote! {
                impl<This> This
                where
                    This: ValueAlg,
                {
                    fn value(&self) -> Self::Value { self.value() }
                }
            },
        )
        .unwrap()
        .to_string();

        // Both halves state `Self` as written, as `extend::ext` does. No supertrait is added.
        assert_eq!(output.matches("Self :: Value").count(), 2, "{output}");
        assert!(!output.contains("< This > :: Value"), "{output}");
        assert!(!output.contains("trait ValueExt < This > :"), "{output}");
    }

    #[test]
    fn keeps_the_supertraits_a_block_states() {
        let output = ext_internal(
            quote!(name = ValueExt, supertraits = Sorts),
            quote! {
                impl<This> This
                where
                    This: ValueAlg,
                {
                    fn value(&self) -> u32 { 1 }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("trait ValueExt < This > : Sorts where"), "{output}");
    }

    #[test]
    fn states_no_supertraits_for_an_unbounded_carrier() {
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

        assert!(!output.contains("trait ValueExt < This > :"), "{output}");
    }

    #[test]
    fn declares_without_the_bindings_a_body_asks_for() {
        let output = ext_internal(
            quote!(name = ValueExt),
            quote! {
                impl<This> This {
                    fn value(mut self, mut increment: u32) -> u32 { increment += 1; increment }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("fn value (self , increment : u32) -> u32 ;"), "{output}");
        assert!(output.contains("fn value (mut self , mut increment : u32) -> u32 {"), "{output}");
    }

    #[test]
    fn states_the_future_an_async_method_answers() {
        let output = ext_internal(
            quote!(name = ValueExt),
            quote! {
                impl<This> This {
                    async fn value(&self) -> u32 { 1 }
                }
            },
        )
        .unwrap()
        .to_string();

        // The declaration states the future, the body stays what the author wrote.
        assert!(
            output.contains("fn value (& self) -> impl :: core :: future :: Future < Output = u32 > ;"),
            "{output}"
        );
        assert!(output.contains("async fn value (& self) -> u32"), "{output}");
    }

    #[test]
    fn expands_a_block_stating_no_name_as_one_stating_its_bound_name() {
        let block = quote! {
            impl<This> This
            where
                This: ValueAlg,
            {
                async fn value(mut self, mut increment: u32) -> This::Value { self.value() }
            }
        };
        let unnamed = ext_internal(quote!(), block.clone()).unwrap().to_string();
        let named = ext_internal(quote!(name = ValueAlgExt), block).unwrap().to_string();

        // The name is the only thing a block leaves to its bound.
        assert_eq!(unnamed, named);
    }

    #[test]
    fn names_a_trait_after_the_first_bound_that_is_not_a_basic_trait() {
        let name = |block| {
            let output = ext_internal(quote!(), block).unwrap().to_string();
            output.split_whitespace().skip_while(|token| *token != "trait").nth(1).unwrap().to_owned()
        };

        assert_eq!(
            name(quote!(
                impl<This> This where This: Send + ChunksAlg + FieldAlg {}
            )),
            "ChunksAlgExt"
        );
        assert_eq!(
            name(quote!(
                impl<This: ?Sized + 'static + alux::ShapeAlg> This where This: FieldAlg {}
            )),
            "ShapeAlgExt"
        );
        assert_eq!(
            name(quote!(
                impl<This: HandlerContextAlg<Context>> This {}
            )),
            "HandlerContextAlgExt"
        );
        assert_eq!(
            name(quote!(
                impl<This> This where This: Send {}
            )),
            "SendExt"
        );
        assert_eq!(
            name(quote!(
                impl<This> This where This: Clone + Debug + Copy + core::panic::RefUnwindSafe + Sync + ChunksAlg {}
            )),
            "ChunksAlgExt"
        );
        assert_eq!(
            name(quote!(
                impl<This: ?Sized + 'static> This where This: Clone + Default {}
            )),
            "CloneExt"
        );
        assert_eq!(
            name(quote!(
                impl<This> This {}
            )),
            "ThisExt"
        );
    }

    #[test]
    fn names_a_trait_after_its_carrier() {
        let output = ext_internal(
            quote!(),
            quote! {
                impl<T> &mut Vec<T> {
                    fn first_one(&self) -> u32 { 1 }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("trait RefMutVecTExt < T >"), "{output}");
    }

    #[test]
    fn carries_a_written_out_future_as_an_async_body() {
        let output = ext_internal(
            quote!(name = ValueExt),
            quote! {
                impl<This> This {
                    fn value(&self) -> impl Future<Output = u32> + Send { async move { 1 } }
                }
            },
        )
        .unwrap()
        .to_string();

        // The declaration states the future and what bounds it; the body is the `async fn` it is.
        assert!(output.contains("fn value (& self) -> impl Future < Output = u32 > + Send ;"), "{output}");
        assert!(output.contains("async fn value (& self) -> u32 { 1 }"), "{output}");
    }

    #[test]
    fn leaves_a_future_it_did_not_build_alone() {
        let output = ext_internal(
            quote!(name = ValueExt),
            quote! {
                impl<This> This {
                    fn value(&self) -> impl Future<Output = u32> + Send { answered(self) }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(!output.contains("async fn"), "{output}");
    }

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

        assert!(output.contains("trait ValueExt < This >"), "{output}");
        assert!(output.contains("impl < This > ValueExt < This > for This"), "{output}");
        assert!(!output.contains("ValueOperation"), "{output}");
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
        assert!(output.contains("Handle : core :: borrow :: Borrow < Context > + Send"), "{output}");
    }

    #[test]
    fn keeps_named_parameters_on_the_operation_implementations() {
        let output = ext_internal(
            quote!(name = ReadExt, defunc),
            quote! {
                impl<This, Item> This
                where
                    This: ReadAlg<Item = Item>,
                    Item: Send,
                {
                    async fn echo(&self, item: Item, key: This::Key) -> Item { item }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("struct EchoOperation < Context : ? Sized >"), "{output}");
        assert!(output.contains("impl < Context , Item > :: alux_ext :: OperationAlg"), "{output}");
        assert!(output.contains("impl < Context , Handle , Item > :: alux_ext :: ApplyAlg"), "{output}");
        assert!(output.contains("type Args = (Item , Context :: Key ,)"), "{output}");
        assert!(output.contains("type Output = Item"), "{output}");
        assert_eq!(output.matches("Context : ReadAlg < Item = Item >").count(), 2, "{output}");
        assert!(output.matches("Item : Send").count() >= 2, "{output}");
    }

    #[test]
    fn states_argument_names_on_an_item_no_context_changes() {
        let output = ext_internal(
            quote!(name = NodeExt, defunc),
            quote! {
                impl<This> This
                where
                    This: Node,
                {
                    fn start_node(&self, name: String, base_path: This::Path) {}
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("struct StartNodeArgNames ;"), "{output}");
        assert!(
            output.contains(
                "pub const ARG_NAMES : & 'static [& 'static str] = & [stringify ! (name) , stringify ! (base_path)]"
            ),
            "{output}"
        );
        // A type the method states is known without a context; one naming the carrier is not.
        assert!(
            output.contains(
                "pub const ARG_0 : :: alux_ext :: StatedArg < String > = :: alux_ext :: StatedArg :: new () ;"
            ),
            "{output}"
        );
        assert!(
            output.contains("pub const ARG_1 : :: alux_ext :: ContextArg = :: alux_ext :: ContextArg ;"),
            "{output}"
        );
    }

    #[test]
    fn keeps_the_capability_a_signature_names_a_sort_of() {
        let output = ext_internal(
            quote!(name = LeavesExt, defunc),
            quote! {
                impl<This> This
                where
                    This: Naming,
                {
                    fn leaves(&self, names: Vec<This::Name>) -> Vec<This::Name> { names }
                }
            },
        )
        .unwrap()
        .to_string();

        // Description and application both name `Context::Name`, so both keep `Context: Naming`.
        assert_eq!(output.matches("Context : Naming").count(), 2, "{output}");
        // Only the application owns the arguments in a `Send` future.
        assert_eq!(output.matches("Vec < Context :: Name > : Send").count(), 1, "{output}");
    }

    #[test]
    fn selects_a_methods_type_parameters_on_the_operation() {
        let output = ext_internal(
            quote!(name = SelectExt, defunc),
            quote! {
                impl<This> This
                where
                    This: AsRef<[u8]>,
                {
                    fn select<Predicate: Copy>(&self, predicate: Predicate) -> Vec<u8>
                    where
                        Predicate: Fn(u8) -> bool,
                    {
                        Vec::new()
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("struct SelectOperation < Context : ? Sized , Predicate >"), "{output}");
        assert!(output.contains(":: alux_ext :: OperationAlg for SelectOperation < Context , Predicate >"), "{output}");
        // Description and application both state the method's bounds on what it selects.
        assert!(output.contains("where Predicate : Copy , Predicate : Fn (u8) -> bool , { type Context"), "{output}");
        assert!(output.contains("Predicate : Copy , Predicate : Fn (u8) -> bool , { type Output"), "{output}");
    }

    #[test]
    fn lends_borrowed_arguments_for_one_named_lifetime() {
        let output = ext_internal(
            quote!(name = BytesExt, defunc),
            quote! {
                impl<This> This
                where
                    This: Naming,
                {
                    fn append(&mut self, bytes: &[u8], names: &[This::Name]) {}
                }
            },
        )
        .unwrap()
        .to_string();

        // The type names no lifetime; the application is stated for every loan, the signature at `'static`.
        assert!(output.contains("struct AppendOperation < Context : ? Sized >"), "{output}");
        assert!(output.contains("type Args = (& 'static [u8] , & 'static [Context :: Name] ,)"), "{output}");
        assert!(
            output.contains("ApplyAlg < Handle , (& '__alux_borrow [u8] , & '__alux_borrow [Context :: Name] ,) >"),
            "{output}"
        );
        assert!(output.contains("[Context :: Name] : '__alux_borrow"), "{output}");
        assert!(output.contains("[Context :: Name] : 'static"), "{output}");
    }

    #[test]
    fn admits_an_unsized_context_where_the_block_admits_an_unsized_receiver() {
        let output = ext_internal(
            quote!(name = PrefixExt, defunc),
            quote! {
                impl<This> This
                where
                    This: AsRef<[u8]> + ?Sized,
                {
                    fn length(&self) -> usize { self.as_ref().len() }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("struct LengthOperation < Context : ? Sized >"), "{output}");
        assert!(output.contains("PhantomData < (fn (& Context) , fn () -> ()) >"), "{output}");
        assert_eq!(output.matches("impl < Context : ? Sized").count(), 3, "{output}");
        assert!(output.contains("Context : AsRef < [u8] > ,"), "{output}");
    }

    #[test]
    fn answers_a_borrowed_output_only_to_a_borrowed_handle() {
        let output = ext_internal(
            quote!(name = PrefixExt, defunc),
            quote! {
                impl<This> This
                where
                    This: AsRef<[u8]>,
                {
                    fn prefix(&self, count: usize) -> &[u8] { &self.as_ref()[..count] }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("ApplyAlg < & '__alux_borrow Context , (usize ,) >"), "{output}");
        assert!(output.contains("type Output = & '__alux_borrow [u8]"), "{output}");
        assert!(!output.contains("Handle"), "{output}");
    }

    #[test]
    fn names_an_already_first_order_route_program_without_a_backend_argument() {
        let output = ext_internal(
            quote!(name = DirectApiExt, defunc),
            quote! {
                pub impl<This> This
                where
                    This: HttpApiAlg,
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
    fn carries_the_authored_documentation_onto_the_operation() {
        let output = ext_internal(
            quote!(name = ValueExt, defunc),
            quote! {
                impl<This> This {
                    /// Returns the value as it stands.
                    async fn value(&self) -> u32 {
                        1
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        // A reader landing on the generated type sees what the method says.
        assert!(
            output.contains(r#"# [doc = r" Returns the value as it stands."] # [doc (hidden)] struct ValueOperation"#)
        );
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
    fn chains_several_backends_in_authored_order() {
        let output = ext_internal(
            quote!(name = SurfacesExt, defunc(via = cli, via = jsonrpc, via = http)),
            quote!(
                impl<This> This {}
            ),
        )
        .unwrap()
        .to_string();

        // The first backend runs now and hands the rest on; each one adds its own declarations.
        assert!(output.contains("# [cli (name = SurfacesExt , __alux_then (jsonrpc , http))]"), "{output}");
    }

    #[test]
    fn delegates_defunctionalization_to_the_selected_backend() {
        let output = ext_internal(
            quote!(name = StatusRoutesExt, defunc(via = http)),
            quote! {
                impl<This> This
                where
                    This: HttpApiAlg,
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
