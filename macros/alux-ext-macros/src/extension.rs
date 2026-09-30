//! Generates the trait an extension declares and the impl that carries its bodies.
//!
//! The trait declares each method as the future it answers, its arguments named and without `mut`.
//! The impl states each method as `async fn`.

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::{Async, Plus};
use syn::visit::Visit;
use syn::{
    Expr, FnArg, GenericArgument, Ident, ImplItem, ImplItemFn, ItemImpl, Meta, Pat, PathArguments, ReceiverKind,
    ReturnType, Signature, Stmt, TraitItemConst, TraitItemFn, Type, TypeImplTrait, TypeParamBound, Visibility,
    WherePredicate, parse_quote,
};

/// Carries what an extension's arguments state about its trait.
pub(crate) struct Stated {
    name: Ident,
    supertraits: Option<Punctuated<TypeParamBound, Plus>>,
}

/// Reads the `name` and `supertraits` an extension states, naming an unnamed trait after the block.
///
/// A block over a generic carrier is named after the carrier's first trait bound that is not a
/// marker, so `impl<This> This where This: ChunksAlg + Send` declares `ChunksAlgExt`. Any other
/// block is named after its carrier by the rule `extend::ext` states.
pub(crate) fn stated(arguments: &[Meta], authored: &ItemImpl) -> syn::Result<Stated> {
    let mut name = None;
    let mut supertraits = None;
    for argument in arguments {
        match argument {
            Meta::NameValue(argument) if argument.path.is_ident("name") => {
                name = Some(syn::parse2::<Ident>(argument.value.to_token_stream())?);
            }
            Meta::NameValue(argument) if argument.path.is_ident("supertraits") => {
                supertraits = Some(Punctuated::parse_terminated.parse2(argument.value.to_token_stream())?);
            }
            other => return Err(syn::Error::new_spanned(other, "an extension states `name` and `supertraits`")),
        }
    }
    let name = match name {
        Some(name) => name,
        None => match bound_name(authored) {
            Some(bound) => format_ident!("{bound}Ext"),
            None => format_ident!("{}Ext", carrier_name(&authored.self_ty)?),
        },
    };

    Ok(Stated { name, supertraits })
}

/// States the trait and the impl an extension block means.
pub(crate) fn extension(visibility: &Visibility, stated: Stated, item: &ItemImpl) -> syn::Result<TokenStream> {
    let ItemImpl { attrs, unsafety, generics, self_ty, items, trait_, .. } = item;
    if let Some((path, _)) = trait_ {
        return Err(syn::Error::new(path.span(), "a trait impl states no extension"));
    }
    let Stated { name, supertraits } = stated;

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let extends = supertraits.map(|supertraits| quote!(: #supertraits));
    let declared = items.iter().map(declaration).collect::<syn::Result<Vec<_>>>()?;
    let carried = items
        .iter()
        .map(|item| {
            let mut carried = item.clone();
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

/// Reads the trait bound a generic carrier is named after: its first bound that is not a basic Rust
/// trait, or its first bound when every one is.
///
/// Bounds written on the parameter are read before the `where` clause, each in written order.
fn bound_name(authored: &ItemImpl) -> Option<Ident> {
    const BASIC: [&str; 16] = [
        "Send",
        "Sync",
        "Sized",
        "Unpin",
        "Copy",
        "UnwindSafe",
        "RefUnwindSafe",
        "Clone",
        "Default",
        "Debug",
        "Display",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Any",
    ];

    let Type::Path(carrier) = &*authored.self_ty else { return None };
    let carrier = carrier.path.get_ident()?;
    let generics = &authored.generics;
    // A concrete carrier such as `Registry` is named after itself.
    let param = generics.type_params().find(|param| param.ident == *carrier)?;
    let clauses = generics
        .where_clause
        .iter()
        .flat_map(|clause| &clause.predicates)
        .filter_map(|predicate| match predicate {
            WherePredicate::Type(predicate) => Some(predicate),
            _ => None,
        })
        .filter(|predicate| matches!(&predicate.bounded_ty, Type::Path(bounded) if bounded.path.is_ident(carrier)))
        .flat_map(|predicate| &predicate.bounds);
    let traits = param
        .bounds
        .iter()
        .chain(clauses)
        .filter_map(|bound| match bound {
            TypeParamBound::Trait(bound) if bound.maybe.is_none() => Some(&bound.path.segments.last()?.ident),
            _ => None,
        })
        .collect::<Vec<_>>();

    traits
        .iter()
        .find(|named| !BASIC.iter().any(|basic| *named == basic))
        .or(traits.first())
        .map(|named| (*named).clone())
}

/// Names a trait after its carrier by the rule `extend::ext` states, so `impl<This> This` declares
/// `ThisExt` and `impl<T> &Vec<T>` declares `RefVecTExt`.
fn carrier_name(carrier: &Type) -> syn::Result<Ident> {
    Ok(match carrier {
        Type::Path(named) => {
            struct Idents(Vec<Ident>);

            impl Visit<'_> for Idents {
                fn visit_ident(&mut self, ident: &Ident) {
                    self.0.push(ident.clone());
                }
            }

            let mut idents = Idents(Vec::new());
            idents.visit_type_path(named);
            if idents.0.is_empty() {
                return Err(syn::Error::new(named.span(), "an empty type path names no extension"));
            }

            format_ident!("{}", idents.0.iter().map(Ident::to_string).collect::<String>())
        }
        Type::Reference(referred) if referred.mutability.is_some() => {
            format_ident!("RefMut{}", carrier_name(&referred.elem)?)
        }
        Type::Reference(referred) => format_ident!("Ref{}", carrier_name(&referred.elem)?),
        Type::Array(array) => format_ident!("ListOf{}", carrier_name(&array.elem)?),
        Type::Group(group) => format_ident!("Group{}", carrier_name(&group.elem)?),
        Type::Paren(paren) => format_ident!("Paren{}", carrier_name(&paren.elem)?),
        Type::Ptr(pointer) => format_ident!("PointerTo{}", carrier_name(&pointer.elem)?),
        Type::Slice(slice) => format_ident!("SliceOf{}", carrier_name(&slice.elem)?),
        Type::Tuple(tuple) => format_ident!(
            "TupleOf{}",
            tuple
                .elems
                .iter()
                .map(|elem| carrier_name(elem).map(|name| name.to_string()))
                .collect::<syn::Result<String>>()?
        ),
        Type::Never(_) => format_ident!("Never"),
        Type::FnPtr(function) => {
            let inputs = function
                .inputs
                .iter()
                .map(|input| carrier_name(&input.ty).map(|name| name.to_string()))
                .collect::<syn::Result<String>>()?;
            let output = match &function.output {
                ReturnType::Default => format_ident!("Unit"),
                ReturnType::Type(_, output) => carrier_name(output.as_ref())?,
            };

            format_ident!("BareFn{inputs}{output}")
        }
        Type::TraitObject(object) => {
            let bounds = object
                .bounds
                .iter()
                .map(|bound| match bound {
                    TypeParamBound::Trait(bound) => {
                        Ok(bound.path.segments.iter().map(|segment| segment.ident.to_string()).collect::<String>())
                    }
                    TypeParamBound::Lifetime(lifetime) => Ok(lifetime.ident.to_string()),
                    other => Err(syn::Error::new(other.span(), "this bound names no extension")),
                })
                .collect::<syn::Result<String>>()?;

            format_ident!("TraitObject{bounds}")
        }
        other => return Err(syn::Error::new(other.span(), "this kind of type names no extension; state a `name`")),
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
fn declaration(item: &ImplItem) -> syn::Result<TokenStream> {
    match item {
        ImplItem::Fn(method) => {
            // `inline` states how a body is compiled, which a declaration has no body to state.
            let attrs = method.attrs.iter().filter(|attribute| !attribute.path().is_ident("inline"));
            let sig = declared(&method.sig);
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

/// Declares one signature: the future an `async fn` answers, and its arguments by name.
///
/// `Send` is stated only where the block wrote the future out, since only a body satisfies it.
/// `mut` is dropped and a pattern becomes `_`, which rust-analyzer reports as E0130 otherwise.
fn declared(sig: &Signature) -> Signature {
    let mut declared = sig.clone();
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
