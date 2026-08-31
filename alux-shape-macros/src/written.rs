//! A field stating how it is written, in place of its type stating it.
//!
//! A type states its own shape through `ShapeOf`, which is the ordinary case. It cannot where the
//! type is foreign — the orphan rule forbids the impl — and it cannot where the type is an alias, a
//! `u128` that a surface writes as decimal text being no different from any other `u128`. A field
//! states its own writing in those cases, in the algebra's own vocabulary.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Error, LitInt, LitStr, Result, Token, parenthesized};

/// A shape stated by a field rather than by its type.
pub(crate) enum Written {
    Truth,
    Text,
    /// Bytes written as hexadecimal, of a stated length when fixed.
    Hex(Option<usize>),
    /// An integer of a stated width, written as decimal text.
    Decimal(u16),
    /// An integer of a stated width, written as a JSON number.
    Int(u16),
    /// What follows, under a name of its own.
    ///
    /// Two leaves may share a written form and remain different things; a name is what says so, and it
    /// is what reaches a client as a type of its own.
    Named(Vec<String>, Box<Written>),
    /// A sequence of what follows.
    Seq(Box<Written>),
    /// What follows, or nothing.
    Opt(Box<Written>),
}

impl Parse for Written {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name: syn::Ident = input.parse()?;
        let argument = |input: ParseStream<'_>| -> Result<Option<TokenStream>> {
            if input.peek(syn::token::Paren) {
                let inner;
                parenthesized!(inner in input);

                Ok(Some(inner.parse()?))
            } else {
                Ok(None)
            }
        };

        match name.to_string().as_str() {
            "truth" => Ok(Self::Truth),
            "text" => Ok(Self::Text),
            "hex" => match argument(input)? {
                Some(tokens) => Ok(Self::Hex(Some(syn::parse2::<LitInt>(tokens)?.base10_parse()?))),
                None => Ok(Self::Hex(None)),
            },
            "decimal" => Ok(Self::Decimal(width(&name, argument(input)?)?)),
            "int" => Ok(Self::Int(width(&name, argument(input)?)?)),
            "name" => {
                let tokens = argument(input)?.ok_or_else(|| {
                    Error::new(name.span(), "`name` states a name and a form, as `name(hash32, hex(32))`")
                })?;
                let named = syn::parse::Parser::parse2(
                    |input: ParseStream<'_>| {
                        let named: syn::Ident = input.parse()?;
                        let _: Token![,] = input.parse()?;
                        let form: Written = input.parse()?;

                        Ok((named, form))
                    },
                    tokens,
                )?;

                Ok(Self::Named(crate::words::of_snake(&named.0.to_string()), Box::new(named.1)))
            }
            "seq" => Ok(Self::Seq(Box::new(nested(&name, argument(input)?)?))),
            "opt" => Ok(Self::Opt(Box::new(nested(&name, argument(input)?)?))),
            other => Err(Error::new(
                name.span(),
                format!(
                    "`{other}` is not a written form; state one of truth, text, hex, decimal, int, \
                     name, seq or opt"
                ),
            )),
        }
    }
}

/// Reads the width a written integer states.
fn width(name: &syn::Ident, argument: Option<TokenStream>) -> Result<u16> {
    let tokens =
        argument.ok_or_else(|| Error::new(name.span(), format!("`{name}` states a width, as `{name}(64)`")))?;

    syn::parse2::<LitInt>(tokens)?.base10_parse()
}

/// Reads the form a container states.
fn nested(name: &syn::Ident, argument: Option<TokenStream>) -> Result<Written> {
    let tokens = argument
        .ok_or_else(|| Error::new(name.span(), format!("`{name}` states what it contains, as `{name}(text)`")))?;

    syn::parse2(tokens)
}

impl Written {
    /// The algebra calls this form states.
    pub(crate) fn shape(&self) -> TokenStream {
        match self {
            Self::Truth => quote!(alg.truth()),
            Self::Text => quote!(alg.text()),
            Self::Hex(len) => match len {
                Some(len) => quote!(alg.bytes_hex(Some(#len))),
                None => quote!(alg.bytes_hex(None)),
            },
            Self::Decimal(bits) => quote!(alg.int_decimal(false, #bits)),
            Self::Int(bits) => quote!(alg.int(false, #bits)),
            Self::Named(words, item) => {
                let item = item.shape();
                let words = words.iter().map(|word| LitStr::new(word, proc_macro2::Span::call_site()));

                quote!(alg.named(&[#(#words),*], #item))
            }
            Self::Seq(item) => {
                let item = item.shape();

                quote!(alg.seq(#item))
            }
            Self::Opt(item) => {
                let item = item.shape();

                quote!(alg.opt(#item))
            }
        }
    }
}

/// The written form a field states, if it states one.
pub(crate) fn written_of(attrs: &[Attribute]) -> Result<Option<Written>> {
    let mut written = None;

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("written")) {
        written = Some(attr.parse_args::<Written>()?);
    }

    Ok(written)
}

/// Keeps `Token` in scope for the parser above.
const _: Option<Token![,]> = None;
