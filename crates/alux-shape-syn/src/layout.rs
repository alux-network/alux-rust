//! Reading the shape a layout states.
//!
//! What is read are the `serde` attributes already present, so the shape and the serialization are
//! two readings of one annotation: a rename states which words a member is named by, `flatten`
//! states a merge, and a tagging attribute states which encoding of a choice is meant. A spelling is
//! not read, because a shape carries no spelling: `rename_all` says how names are written, which is
//! an interpretation's statement and not the type's.

use crate::words;
use crate::written::{Written, written_of};
use syn::spanned::Spanned;
use syn::{Attribute, Data, DeriveInput, Error, Fields, LitStr, Result, Token, Type, Variant};

/// The shape a layout states: its name and its body.
#[derive(Clone)]
#[allow(missing_debug_implementations, reason = "it holds `syn` types, which are `Debug` only with `extra-traits`")]
pub struct Layout {
    /// The layout's name, as words.
    pub words: Vec<String>,
    /// What it holds.
    pub body: Body,
}

/// What a layout holds.
#[derive(Clone)]
#[allow(missing_debug_implementations, reason = "it holds `syn` types, which are `Debug` only with `extra-traits`")]
pub enum Body {
    /// A struct with named members: its members, in order.
    Product(Vec<Member>),
    /// A tuple struct, or a unit struct: its members' forms by position. None is `unit`, one is
    /// that member's shape (serde writes a newtype as its item), and more are a tuple.
    Positional(Vec<Form>),
    /// An enum: how it states its alternative, and the alternatives.
    Choice(Tagging, Vec<Alternative>),
}

/// One member of a product.
#[derive(Clone)]
#[allow(missing_debug_implementations, reason = "it holds `syn` types, which are `Debug` only with `extra-traits`")]
pub enum Member {
    /// A member under its name.
    Field(Vec<String>, Form),
    /// Another product's members, observed as this product's own (`flatten`).
    Merge(Form),
}

/// One alternative of a choice.
#[derive(Clone)]
#[allow(missing_debug_implementations, reason = "it holds `syn` types, which are `Debug` only with `extra-traits`")]
pub struct Alternative {
    /// The alternative's name, as words.
    pub words: Vec<String>,
    /// What it carries; `None` for a unit alternative.
    pub carries: Option<Form>,
}

/// Where a member's shape comes from: its type, or a form the member states.
#[derive(Clone)]
#[allow(missing_debug_implementations, reason = "it holds `syn` types, which are `Debug` only with `extra-traits`")]
pub enum Form {
    /// The member's type, which states its own shape.
    Type(Box<Type>),
    /// A form the member states itself.
    Written(Written),
}

/// How a choice states which alternative a value carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tagging {
    /// `{ "alternative": carried }`.
    External,
    /// A tag member beside the carried members.
    Internal(Vec<String>),
    /// A tag member and a content member.
    Adjacent(Vec<String>, Vec<String>),
    /// No tag: the carried shape alone.
    Untagged,
}

impl Body {
    /// Reports whether every alternative of a choice is a name alone.
    pub fn names_only(&self) -> bool {
        matches!(self, Body::Choice(_, alternatives) if alternatives.iter().all(|alternative| alternative.carries.is_none()))
    }
}

/// Returns the shape `input` states.
///
/// # Errors
///
/// Returns an error for a union, an alternative with more than one member, and a malformed `serde`
/// or `written` attribute.
pub fn read_layout(input: &DeriveInput) -> Result<Layout> {
    let words = renamed(&input.attrs)?.unwrap_or_else(|| words::of_camel(&input.ident.to_string()));
    let body = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(_) => Body::Product(product(&data.fields)?),
            Fields::Unnamed(_) | Fields::Unit => Body::Positional(positional(&data.fields)?),
        },
        Data::Enum(data) => Body::Choice(tagging(&input.attrs)?, choice(&data.variants)?),
        Data::Union(_) => return Err(Error::new(input.span(), "a union states no shape")),
    };
    Ok(Layout { words, body })
}

/// The members a struct states.
fn product(fields: &Fields) -> Result<Vec<Member>> {
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
        let form = match written_of(&field.attrs)? {
            Some(written) => Form::Written(written),
            None => Form::Type(Box::new(field.ty.clone())),
        };
        if merged(&field.attrs)? {
            members.push(Member::Merge(form));
            continue;
        }
        let ident = field.ident.as_ref().expect("a named member has an identifier");
        let words = renamed(&field.attrs)?.unwrap_or_else(|| words::of_snake(&ident.to_string()));
        members.push(Member::Field(words, form));
    }
    Ok(members)
}

/// The forms a tuple or unit struct's members state, by position.
fn positional(fields: &Fields) -> Result<Vec<Form>> {
    let mut forms = Vec::new();
    for field in fields {
        if skipped(&field.attrs)? {
            continue;
        }
        forms.push(match written_of(&field.attrs)? {
            Some(written) => Form::Written(written),
            None => Form::Type(Box::new(field.ty.clone())),
        });
    }
    Ok(forms)
}

/// The alternatives an enum states.
fn choice(variants: &syn::punctuated::Punctuated<Variant, Token![,]>) -> Result<Vec<Alternative>> {
    variants
        .iter()
        .map(|variant| {
            let words = renamed(&variant.attrs)?.unwrap_or_else(|| words::of_camel(&variant.ident.to_string()));
            let carries = match &variant.fields {
                Fields::Unit => None,
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                    Some(Form::Type(Box::new(fields.unnamed[0].ty.clone())))
                }
                _ => {
                    return Err(Error::new(
                        variant.fields.span(),
                        "an alternative states one shape; a tuple or struct alternative is not read yet",
                    ));
                }
            };
            Ok(Alternative { words, carries })
        })
        .collect()
}

/// Which encoding of a choice the attributes state.
fn tagging(attrs: &[Attribute]) -> Result<Tagging> {
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
            } else if meta.input.peek(Token![=]) {
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

/// Returns the words a `rename` states, if one does.
///
/// # Errors
///
/// Returns the error of a malformed `serde` attribute.
pub fn renamed(attrs: &[Attribute]) -> Result<Option<Vec<String>>> {
    let mut renamed = None;

    for attr in serde_attrs(attrs) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                // A rename may state a name in any spelling, so its words are read back out of it.
                renamed = Some(words::of_spelled(&meta.value()?.parse::<LitStr>()?.value()));
            } else if meta.input.peek(Token![=]) {
                let _: syn::Expr = meta.value()?.parse()?;
            }

            Ok(())
        })?;
    }

    Ok(renamed)
}

/// Reports whether a member states a merge rather than a member of its own.
///
/// # Errors
///
/// Returns the error of a malformed `serde` attribute.
pub fn merged(attrs: &[Attribute]) -> Result<bool> {
    flag(attrs, "flatten")
}

/// Reports whether a member is left unwritten.
///
/// # Errors
///
/// Returns the error of a malformed `serde` attribute.
pub fn skipped(attrs: &[Attribute]) -> Result<bool> {
    Ok(flag(attrs, "skip")? || flag(attrs, "skip_serializing")?)
}

/// Whether a bare `serde` flag is present.
fn flag(attrs: &[Attribute], name: &str) -> Result<bool> {
    let mut present = false;

    for attr in serde_attrs(attrs) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(name) && !meta.input.peek(Token![=]) {
                present = true;
            } else if meta.input.peek(Token![=]) {
                let _: syn::Expr = meta.value()?.parse()?;
            }

            Ok(())
        })?;
    }

    Ok(present)
}

/// The `serde` attributes among a set, which are the ones a shape is read from.
fn serde_attrs(attrs: &[Attribute]) -> impl Iterator<Item = &Attribute> {
    attrs.iter().filter(|attr| attr.path().is_ident("serde"))
}
