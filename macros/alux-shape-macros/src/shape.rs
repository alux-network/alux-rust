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
use crate::written::shape as written_shape;
use alux_shape_syn::{Body, Form, Layout, Member, Tagging, read_layout};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Error, Fields, LitStr, Result, parse_quote};

/// States `ShapeOf` for the type this derive was written on.
pub(crate) fn expand(input: DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let Layout { words, body } = read_layout(&input)?;
    let words = literal(&words);

    let mut members = Vec::new();
    let construction = match &input.data {
        // A struct with named members is a product of its fields; a tuple struct states its shape
        // and nothing more.
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => construct::product(&input, named)?,
            Fields::Unnamed(_) | Fields::Unit => quote!(),
        },
        Data::Enum(data) => construct::sum(&input, data)?,
        Data::Union(_) => return Err(Error::new(input.span(), "a union states no shape")),
    };
    let body = match &body {
        Body::Product(fields) => product(fields, &mut members),
        Body::Positional(forms) => positional(forms, &mut members),
        Body::Choice(..) => choice(&body, &mut members),
    };

    // Each member's type states its own shape, which a type parameter does only where it is bound to.
    let mut generics = input.generics.clone();
    let alg = algebra(&input.generics);
    generics.params.push(parse_quote!(#alg));
    let predicates = &mut generics.make_where_clause().predicates;
    predicates.push(parse_quote!(#alg: ::alux_shape::ShapeAlg + ::alux_shape::FieldAlg));
    for member in members {
        predicates.push(parse_quote!(#member: ::alux_shape::ShapeOf<#alg, Shape = <#alg as ::alux_shape::Sorts>::Ty>));
    }
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let (_, ty_generics, _) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::alux_shape::ShapeOf<#alg> for #name #ty_generics
        #where_clause
        {
            type Shape = <#alg as ::alux_shape::Sorts>::Ty;

            fn shape_of(alg: &#alg) -> Self::Shape {
                use ::alux_shape::{FieldAlg as _, ShapeAlg as _, ShapeExt as _, ShapeTaggedExt as _};

                let body = #body;

                alg.named(#words, body)
            }
        }

        #construction
    })
}

/// Names the algebra parameter with an identifier none of the layout's own parameters is.
fn algebra(generics: &syn::Generics) -> syn::Ident {
    let mut name = String::from("Alg");
    while generics.type_params().any(|param| param.ident == name) {
        name.push('_');
    }
    syn::Ident::new(&name, proc_macro2::Span::call_site())
}

/// A name, as the words it is made of.
fn literal(words: &[String]) -> TokenStream {
    let words = words.iter().map(|word| LitStr::new(word, proc_macro2::Span::call_site()));

    quote!(&[#(#words),*])
}

/// The shape a form states: its type's, which the member then requires of it, or its own.
fn form(form: &Form, members: &mut Vec<syn::Type>) -> TokenStream {
    match form {
        Form::Type(ty) => {
            members.push((**ty).clone());
            // The algebra is the one `alg` is, so its parameter is never named here.
            quote!(<#ty as ::alux_shape::ShapeOf<_>>::shape_of(alg))
        }
        Form::Written(written) => written_shape(written),
    }
}

/// The product a struct's members state.
fn product(fields: &[Member], types: &mut Vec<syn::Type>) -> TokenStream {
    let members: Vec<_> = fields
        .iter()
        .map(|member| match member {
            Member::Merge(merged) => {
                let shape = form(merged, types);
                quote!(alg.merge(#shape))
            }
            Member::Field(words, member) => {
                let shape = form(member, types);
                let words = literal(words);
                quote!(alg.field(#words, #shape))
            }
        })
        .collect();

    let bindings = members.iter().enumerate().map(|(index, member)| {
        let binding = format_ident!("member_{index}");

        quote!(let #binding = #member;)
    });
    let names = (0..members.len()).map(|index| format_ident!("member_{index}"));

    quote! {{
        #(#bindings)*

        alg.product(vec![#(#names),*])
    }}
}

/// The shape a tuple or unit struct states: `unit` for no member, a newtype's item for one, and a
/// tuple for more, as serde writes each.
fn positional(forms: &[Form], types: &mut Vec<syn::Type>) -> TokenStream {
    let shapes: Vec<_> = forms.iter().map(|member| form(member, types)).collect();
    match shapes.as_slice() {
        [] => quote!(alg.unit()),
        [item] => item.clone(),
        _ => quote!(alg.tuple(vec![#(#shapes),*])),
    }
}

/// The choice a set of alternatives states, under the encoding the attributes state.
fn choice(body: &Body, types: &mut Vec<syn::Type>) -> TokenStream {
    let Body::Choice(tagging, alternatives) = body else { unreachable!("a choice is read from an enum") };
    if body.names_only() {
        let names = alternatives.iter().map(|alternative| literal(&alternative.words));
        return quote!(alg.sum_of_names(vec![#(#names),*]));
    }
    let alternatives: Vec<_> = alternatives
        .iter()
        .map(|alternative| {
            let words = literal(&alternative.words);
            let shape = alternative.carries.as_ref().map_or_else(|| quote!(alg.unit()), |carried| form(carried, types));
            quote!((#words, #shape))
        })
        .collect();

    match tagging {
        Tagging::External => quote!(alg.sum_external(vec![#(#alternatives),*])),
        Tagging::Untagged => quote!(alg.sum_untagged(vec![#(#alternatives),*])),
        Tagging::Internal(tag) => {
            let tag = literal(tag);

            quote!(alg.sum_internal(#tag, vec![#(#alternatives),*]))
        }
        Tagging::Adjacent(tag, content) => {
            let tag = literal(tag);
            let content = literal(content);

            quote!(alg.sum_adjacent(#tag, #content, vec![#(#alternatives),*]))
        }
    }
}
