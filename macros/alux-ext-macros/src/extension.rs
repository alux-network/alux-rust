//! Generates the trait an extension declares and the impl that carries its bodies.
//!
//! The trait declares each method as the future it answers, its arguments named and without `mut`.
//! The impl states each method as `async fn`.

use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Async, Plus};
use syn::visit_mut::{self, VisitMut};
use syn::{
    Expr, FnArg, GenericArgument, Ident, ImplItem, ImplItemFn, ItemImpl, Pat, PathArguments, ReceiverKind, ReturnType,
    Signature, Stmt, TraitItemConst, TraitItemFn, Type, TypeImplTrait, TypeParamBound, Visibility, parse_quote,
};

/// States the trait and the impl an extension block means.
pub(crate) fn extension(
    visibility: &Visibility,
    name: &Ident,
    supertraits: Option<Punctuated<TypeParamBound, Plus>>,
    item: &ItemImpl,
) -> syn::Result<TokenStream> {
    let ItemImpl { attrs, unsafety, generics, self_ty, items, trait_, .. } = item;
    if let Some((path, _)) = trait_ {
        return Err(syn::Error::new(path.span(), "a trait impl states no extension"));
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let extends = supertraits.map(|supertraits| quote!(: #supertraits));
    let declared = items.iter().map(|item| declaration(item, self_ty)).collect::<syn::Result<Vec<_>>>()?;
    let carried = items
        .iter()
        .map(|item| {
            let mut carried = item.clone();
            AsCarrier { carrier: self_ty }.visit_impl_item_mut(&mut carried);
            if let ImplItem::Fn(method) = &mut carried {
                as_async(method);
            }

            carried
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        #(#attrs)*
        #[allow(non_camel_case_types)]
        #visibility #unsafety trait #name #impl_generics #extends #where_clause {
            #(#declared)*
        }

        #(#attrs)*
        impl #impl_generics #name #ty_generics for #self_ty #where_clause {
            #(#carried)*
        }
    })
}

/// States a method answering a future as `async fn`, with the body its `async` block carries.
///
/// A body that is not one `async` block, such as one forwarding another call, is left as written.
fn as_async(method: &mut ImplItemFn) {
    let ReturnType::Type(_, answered) = &method.sig.output else {
        return;
    };
    let Type::ImplTrait(answered) = &**answered else {
        return;
    };
    let Some(output) = future_output(answered) else {
        return;
    };
    let [Stmt::Expr(Expr::Async(awaited), None)] = method.block.stmts.as_slice() else {
        return;
    };

    method.sig.asyncness = Some(Async::default());
    method.sig.output = parse_quote!(-> #output);
    method.block = awaited.block.clone();
}

/// Reads `T` of an `impl Future<Output = T>`.
fn future_output(answered: &TypeImplTrait) -> Option<Type> {
    answered.bounds.iter().find_map(|bound| {
        let TypeParamBound::Trait(bound) = bound else {
            return None;
        };
        let stated = bound.path.segments.last()?;
        if stated.ident != "Future" {
            return None;
        }
        let PathArguments::AngleBracketed(arguments) = &stated.arguments else {
            return None;
        };

        arguments.args.iter().find_map(|argument| match argument {
            GenericArgument::AssocType(assoc) if assoc.ident == "Output" => Some(assoc.ty.clone()),
            _ => None,
        })
    })
}

/// States one item of the block as the declaration the trait carries.
fn declaration(item: &ImplItem, carrier: &Type) -> syn::Result<TokenStream> {
    match item {
        ImplItem::Fn(method) => {
            // `inline` states how a body is compiled, which a declaration has no body to state.
            let attrs = method.attrs.iter().filter(|attribute| !attribute.path().is_ident("inline"));
            let sig = declared(&method.sig, carrier);
            let declared: TraitItemFn = parse_quote!(#(#attrs)* #sig;);

            Ok(quote!(#[allow(unused_attributes)] #declared))
        }
        ImplItem::Const(stated) => {
            let (attrs, ident, ty) = (&stated.attrs, &stated.ident, &stated.ty);
            let declared: TraitItemConst = parse_quote!(#(#attrs)* const #ident: #ty;);

            Ok(quote!(#declared))
        }
        other => Err(syn::Error::new(other.span(), "an extension states methods and consts")),
    }
}

/// Names the carrier where the block wrote `Self`.
///
/// The trait is generic over the carrier, so `This::Chunk` resolves from the bounds the block
/// states, while `Self::Chunk` needs a bound on `Self`. A nested item keeps its own `Self`.
struct AsCarrier<'a> {
    carrier: &'a Type,
}

impl VisitMut for AsCarrier<'_> {
    fn visit_item_mut(&mut self, _: &mut syn::Item) {}

    fn visit_type_mut(&mut self, ty: &mut Type) {
        if let Type::Path(named) = ty
            && named.qself.is_none()
            && named.path.segments.first().is_some_and(|segment| segment.ident == "Self")
        {
            let carrier = self.carrier;
            let rest = named.path.segments.iter().skip(1).collect::<Vec<_>>();
            *ty = if rest.is_empty() { carrier.clone() } else { parse_quote!(<#carrier> #(:: #rest)*) };
        }

        visit_mut::visit_type_mut(self, ty);
    }
}

/// Declares one signature: the future an `async fn` answers, and its arguments by name.
///
/// `Send` is stated only where the block wrote the future out, since only a body satisfies it.
/// `mut` is dropped and a pattern becomes `_`, which rust-analyzer reports as E0130 otherwise.
fn declared(sig: &Signature, carrier: &Type) -> Signature {
    let mut declared = sig.clone();
    AsCarrier { carrier }.visit_signature_mut(&mut declared);
    declared.inputs = declared
        .inputs
        .into_iter()
        .map(|argument| match argument {
            FnArg::Receiver(mut received) => {
                // `mut self`, which a declaration has no use for. `&mut self` states a borrow.
                if matches!(received.kind, ReceiverKind::Value) {
                    received.mutability = None;
                }

                FnArg::Receiver(received)
            }
            FnArg::Typed(mut taken) => {
                taken.pat = Box::new(match *taken.pat {
                    Pat::Ident(mut named) if named.subpat.is_none() => {
                        named.mutability = None;
                        named.by_ref = None;

                        Pat::Ident(named)
                    }
                    _ => parse_quote!(_),
                });

                FnArg::Typed(taken)
            }
        })
        .collect();

    if declared.asyncness.take().is_some() {
        let answered = match &declared.output {
            ReturnType::Default => parse_quote!(()),
            ReturnType::Type(_, answered) => answered.clone(),
        };
        declared.output = parse_quote!(-> impl ::core::future::Future<Output = #answered>);
    }

    declared
}
