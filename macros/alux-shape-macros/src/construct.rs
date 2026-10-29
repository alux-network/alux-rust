//! Reading how a value is made from its members out of a layout, beside its shape.
//!
//! The names are the words the shape reads from the same layout, `serde` renames included, written
//! as identifiers. A skipped member is no field: it is made with `Default`, as `serde` makes it when
//! reading. A flattened member is one field carrying the whole value. An enum states a `Sum` when it
//! has two to sixteen alternatives, which is what `Sum2` through `Sum16` hold.

use crate::shape::{merged, renamed, skipped};
use crate::words;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{DataEnum, DeriveInput, Fields, FieldsNamed, LitStr, Result, parse_quote};

/// The alternatives a `SumN` holds, by position.
const VARIANTS: [&str; 16] = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P"];

/// States `Product` for a struct whose shape is read from named members.
pub(crate) fn product(input: &DeriveInput, named: &FieldsNamed) -> Result<TokenStream> {
    let mut idents = Vec::new();
    let mut types = Vec::new();
    let mut names = Vec::new();
    let mut defaulted = Vec::new();
    let mut generics = input.generics.clone();

    for field in &named.named {
        let ident = field.ident.as_ref().expect("a named member has an identifier");
        if skipped(&field.attrs)? {
            let ty = &field.ty;
            generics.make_where_clause().predicates.push(parse_quote!(#ty: ::core::default::Default));
            defaulted.push(ident);
            continue;
        }
        // A flattened member is named by its identifier: its own words are its members'.
        let _ = merged(&field.attrs)?;
        let words = renamed(&field.attrs)?.unwrap_or_else(|| words::of_snake(&ident.to_string()));
        idents.push(ident);
        types.push(&field.ty);
        names.push(identifier(&words));
    }

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::alux_shape::Product for #name #ty_generics #where_clause {
            type Fields = (#(#types,)*);
            const FIELDS: &'static [&'static str] = &[#(#names),*];

            fn from_fields((#(#idents,)*): Self::Fields) -> Self {
                Self { #(#idents,)* #(#defaulted: ::core::default::Default::default(),)* }
            }

            fn into_fields(self) -> Self::Fields {
                (#(self.#idents,)*)
            }
        }
    })
}

/// States `Sum` for an enum whose shape is read from its alternatives, when `Sum2` through `Sum16`
/// can hold them.
pub(crate) fn sum(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream> {
    let count = data.variants.len();
    if !(2..=16).contains(&count) {
        return Ok(TokenStream::new());
    }

    let sum = format_ident!("Sum{count}");
    let mut types = Vec::new();
    let mut names = Vec::new();
    let mut from = Vec::new();
    let mut into = Vec::new();

    for (variant, position) in data.variants.iter().zip(VARIANTS) {
        let ident = &variant.ident;
        let position = format_ident!("{position}");
        let words = renamed(&variant.attrs)?.unwrap_or_else(|| words::of_camel(&ident.to_string()));
        names.push(identifier(&words));

        match &variant.fields {
            Fields::Unit => {
                types.push(quote!(()));
                from.push(quote!(::alux_shape::#sum::#position(()) => Self::#ident));
                into.push(quote!(Self::#ident => ::alux_shape::#sum::#position(())));
            }
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let ty = &fields.unnamed[0].ty;
                types.push(quote!(#ty));
                from.push(quote!(::alux_shape::#sum::#position(value) => Self::#ident(value)));
                into.push(quote!(Self::#ident(value) => ::alux_shape::#sum::#position(value)));
            }
            _ => return Ok(TokenStream::new()),
        }
    }

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::alux_shape::Sum for #name #ty_generics #where_clause {
            type Alternatives = ::alux_shape::#sum<#(#types),*>;
            const ALTERNATIVES: &'static [&'static str] = &[#(#names),*];

            fn from_alternative(alternative: Self::Alternatives) -> Self {
                match alternative {
                    #(#from,)*
                }
            }

            fn into_alternative(self) -> Self::Alternatives {
                match self {
                    #(#into,)*
                }
            }
        }
    })
}

/// Writes a name's words as an identifier.
fn identifier(words: &[String]) -> LitStr {
    LitStr::new(&words.join("_"), Span::call_site())
}
