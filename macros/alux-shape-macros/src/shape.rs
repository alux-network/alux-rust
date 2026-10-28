//! Reading a shape out of a layout.
//!
//! A type that already exists states its shape through this derive rather than having one written
//! beside it. What it reads are the `serde` attributes already present, so the shape and the
//! serialization are two readings of one annotation: a rename states which words a member is named
//! by, `flatten` states a merge, and a tagging attribute states which encoding of a choice is meant.
//! No attribute survives into the term.
//!
//! A spelling is not read, because a term carries no spelling: `rename_all` says how names are
//! written, which is the interpretation's statement and not the type's.

use crate::construct;
use crate::words;
use crate::written::written_of;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Error, Fields, LitStr, Result, Variant, parse_quote};

/// How a choice states which alternative a value carries.
enum Tagging {
    External,
    Internal(Vec<String>),
    Adjacent(Vec<String>, Vec<String>),
    Untagged,
}

/// States `ShapeOf` for the type this derive was written on.
pub(crate) fn expand(input: DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let words = renamed(&input.attrs)?.unwrap_or_else(|| words::of_camel(&name.to_string()));
    let words = literal(&words);

    let mut members = Vec::new();
    let (body, construction) = match &input.data {
        Data::Struct(data) => {
            let body = product(&data.fields, &mut members)?;
            let Fields::Named(named) = &data.fields else { unreachable!("a product is read from named members") };
            (body, construct::product(&input, named)?)
        }
        Data::Enum(data) => (choice(&input.attrs, &data.variants, &mut members)?, construct::sum(&input, data)?),
        Data::Union(_) => {
            return Err(Error::new(input.span(), "a union states no shape"));
        }
    };

    // Each member's type states its own shape, which a type parameter does only where it is bound to.
    let mut generics = input.generics.clone();
    generics.params.push(parse_quote!(Alg));
    let predicates = &mut generics.make_where_clause().predicates;
    predicates.push(parse_quote!(Alg: ::alux_shape::ShapeAlg + ::alux_shape::FieldAlg));
    for member in members {
        predicates.push(parse_quote!(#member: ::alux_shape::ShapeOf<Alg, Shape = <Alg as ::alux_shape::Sorts>::Ty>));
    }
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::alux_shape::ShapeOf<Alg> for #name #ty_generics
        #where_clause
        {
            type Shape = <Alg as ::alux_shape::Sorts>::Ty;

            fn shape_of(alg: &Alg) -> Self::Shape {
                use ::alux_shape::{FieldAlg as _, ShapeAlg as _, ShapeExt as _, ShapeTaggedExt as _};

                let body = #body;

                alg.named(#words, body)
            }
        }

        #construction
    })
}

/// A name, as the words it is made of.
fn literal(words: &[String]) -> TokenStream {
    let words = words.iter().map(|word| LitStr::new(word, proc_macro2::Span::call_site()));

    quote!(&[#(#words),*])
}

/// The shape of one member's type, which the member then requires of it.
fn shape_of_type(ty: &syn::Type, members: &mut Vec<syn::Type>) -> TokenStream {
    members.push(ty.clone());
    quote!(<#ty as ::alux_shape::ShapeOf<Alg>>::shape_of(alg))
}

/// The product a struct's members state.
fn product(fields: &Fields, types: &mut Vec<syn::Type>) -> Result<TokenStream> {
    let Fields::Named(named) = fields else {
        return Err(Error::new(
            fields.span(),
            "a product is read from named members; a newtype or tuple layout states no member names",
        ));
    };

    let mut members = Vec::new();

    for field in &named.named {
        if skipped(&field.attrs)? {
            continue;
        }

        // A field states its own writing where its type cannot: a foreign type, or an alias whose
        // meaning is narrower than the type it names.
        let shape = match written_of(&field.attrs)? {
            Some(written) => written.shape(),
            None => shape_of_type(&field.ty, types),
        };

        if merged(&field.attrs)? {
            members.push(quote!(alg.merge(#shape)));
            continue;
        }

        let ident = field.ident.as_ref().expect("a named member has an identifier");
        let words = renamed(&field.attrs)?.unwrap_or_else(|| words::of_snake(&ident.to_string()));
        let words = literal(&words);

        members.push(quote!(alg.field(#words, #shape)));
    }

    let bindings = members.iter().enumerate().map(|(index, member)| {
        let binding = format_ident!("member_{index}");

        quote!(let #binding = #member;)
    });
    let names = (0..members.len()).map(|index| format_ident!("member_{index}"));

    Ok(quote! {{
        #(#bindings)*

        alg.product(vec![#(#names),*])
    }})
}

/// The choice a set of alternatives states, under the encoding the attributes state.
fn choice(
    attrs: &[syn::Attribute],
    variants: &syn::punctuated::Punctuated<Variant, syn::Token![,]>,
    types: &mut Vec<syn::Type>,
) -> Result<TokenStream> {
    let tagging = tagging(attrs)?;
    let named_only = variants.iter().all(|variant| matches!(variant.fields, Fields::Unit));

    let mut alternatives = Vec::new();

    for variant in variants {
        let words = renamed(&variant.attrs)?.unwrap_or_else(|| words::of_camel(&variant.ident.to_string()));
        let words = literal(&words);

        if named_only {
            alternatives.push(words);
            continue;
        }

        let shape = match &variant.fields {
            Fields::Unit => quote!(alg.unit()),
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => shape_of_type(&fields.unnamed[0].ty, types),
            _ => {
                return Err(Error::new(
                    variant.fields.span(),
                    "an alternative states one shape; a tuple or struct alternative is not read yet",
                ));
            }
        };

        alternatives.push(quote!((#words, #shape)));
    }

    if named_only {
        return Ok(quote!(alg.sum_of_names(vec![#(#alternatives),*])));
    }

    Ok(match tagging {
        Tagging::External => quote!(alg.sum_external(vec![#(#alternatives),*])),
        Tagging::Untagged => quote!(alg.sum_untagged(vec![#(#alternatives),*])),
        Tagging::Internal(tag) => {
            let tag = literal(&tag);

            quote!(alg.sum_internal(#tag, vec![#(#alternatives),*]))
        }
        Tagging::Adjacent(tag, content) => {
            let tag = literal(&tag);
            let content = literal(&content);

            quote!(alg.sum_adjacent(#tag, #content, vec![#(#alternatives),*]))
        }
    })
}

/// Which encoding of a choice the attributes state.
fn tagging(attrs: &[syn::Attribute]) -> Result<Tagging> {
    let mut tag = None;
    let mut content = None;
    let mut untagged = false;

    for attr in serde_attrs(attrs) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("untagged") {
                untagged = true;
            } else if meta.path.is_ident("tag") {
                tag = Some(words::of_spelled(&meta.value()?.parse::<LitStr>()?.value()));
            } else if meta.path.is_ident("content") {
                content = Some(words::of_spelled(&meta.value()?.parse::<LitStr>()?.value()));
            } else if meta.input.peek(syn::Token![=]) {
                let _: syn::Expr = meta.value()?.parse()?;
            }

            Ok(())
        })?;
    }

    Ok(match (untagged, tag, content) {
        (true, ..) => Tagging::Untagged,
        (false, Some(tag), Some(content)) => Tagging::Adjacent(tag, content),
        (false, Some(tag), None) => Tagging::Internal(tag),
        (false, None, _) => Tagging::External,
    })
}

/// The words a `rename` states, if one does.
pub(crate) fn renamed(attrs: &[syn::Attribute]) -> Result<Option<Vec<String>>> {
    let mut renamed = None;

    for attr in serde_attrs(attrs) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                // A rename may state a name in any spelling, so its words are read back out of it.
                renamed = Some(words::of_spelled(&meta.value()?.parse::<LitStr>()?.value()));
            } else if meta.input.peek(syn::Token![=]) {
                let _: syn::Expr = meta.value()?.parse()?;
            }

            Ok(())
        })?;
    }

    Ok(renamed)
}

/// Whether a member states a merge rather than a member of its own.
pub(crate) fn merged(attrs: &[syn::Attribute]) -> Result<bool> {
    flag(attrs, "flatten")
}

/// Whether a member is written at all.
pub(crate) fn skipped(attrs: &[syn::Attribute]) -> Result<bool> {
    Ok(flag(attrs, "skip")? || flag(attrs, "skip_serializing")?)
}

/// Whether a bare `serde` flag is present.
fn flag(attrs: &[syn::Attribute], name: &str) -> Result<bool> {
    let mut present = false;

    for attr in serde_attrs(attrs) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(name) && !meta.input.peek(syn::Token![=]) {
                present = true;
            } else if meta.input.peek(syn::Token![=]) {
                let _: syn::Expr = meta.value()?.parse()?;
            }

            Ok(())
        })?;
    }

    Ok(present)
}

/// The `serde` attributes among a set, which are the ones a shape is read from.
fn serde_attrs(attrs: &[syn::Attribute]) -> impl Iterator<Item = &syn::Attribute> {
    attrs.iter().filter(|attr| attr.path().is_ident("serde"))
}
