//! Enumerating tuple widths: one invocation of a caller's macro per width, or per position of each
//! width, so a crate states what one width means and nothing else.

use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, ExprLit, ExprRange, Ident, Lit, Path, RangeLimits, Token};

/// `callback, from..=to[, named = Name][, each = [P, …]]`: the macro to invoke, the widths to invoke
/// it for, the prefix of each width's type name, and the prefixes of each position's identifiers.
pub(crate) struct Widths {
    callback: Path,
    range: core::ops::RangeInclusive<usize>,
    named: Option<Ident>,
    each: Option<Vec<Ident>>,
}

impl Parse for Widths {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let callback: Path = input.parse()?;
        let _: Token![,] = input.parse()?;
        let range: ExprRange = input.parse()?;
        let bound = |bound: &Option<Box<Expr>>| -> syn::Result<usize> {
            match bound.as_deref() {
                Some(Expr::Lit(ExprLit { lit: Lit::Int(int), .. })) => int.base10_parse(),
                _ => Err(syn::Error::new_spanned(&range, "the widths are a range of two integers, as `0..=16`")),
            }
        };
        if !matches!(range.limits, RangeLimits::Closed(_)) {
            return Err(syn::Error::new_spanned(&range, "the widths are an inclusive range, as `0..=16`"));
        }
        let (from, to) = (bound(&range.start)?, bound(&range.end)?);
        if from > to {
            return Err(syn::Error::new_spanned(&range, "the widths run from the narrower to the wider"));
        }
        let (mut named, mut each) = (None, None);
        while input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            if input.is_empty() {
                break;
            }
            let clause: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            match clause.to_string().as_str() {
                "named" => named = Some(input.parse()?),
                "each" => {
                    let prefixes;
                    syn::bracketed!(prefixes in input);
                    each = Some(Punctuated::<Ident, Token![,]>::parse_terminated(&prefixes)?.into_iter().collect());
                }
                _ => return Err(syn::Error::new_spanned(clause, "a clause is `named = Name` or `each = [P, …]`")),
            }
        }
        if each.is_some() && to > LETTERS.len() {
            return Err(syn::Error::new_spanned(&range, "`each` names positions by letter, so widths stop at 26"));
        }
        Ok(Widths { callback, range: from..=to, named, each })
    }
}

/// The letters naming positions under `each`.
const LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

/// `A1 => 0, …, An => n - 1`: a width's type parameters with their positions.
fn positions(width: usize) -> TokenStream {
    let pairs = (0..width).map(|at| {
        let ty = format_ident!("A{}", at + 1);
        let index = Literal::usize_unsuffixed(at);
        quote!(#ty => #index)
    });
    quote!(#(#pairs),*)
}

/// `(P0, Q0, …, A, 0), …`: each position's identifiers under `prefixes`, its letter, and its index.
fn each_position(width: usize, prefixes: &[Ident]) -> TokenStream {
    let tuples = (0..width).map(|at| {
        let named = prefixes.iter().map(|prefix| format_ident!("{prefix}{at}"));
        let letter = format_ident!("{}", &LETTERS[at..=at]);
        let index = Literal::usize_unsuffixed(at);
        quote!((#(#named,)* #letter, #index))
    });
    quote!(#(#tuples),*)
}

/// Invokes the callback once per width.
pub(crate) fn for_tuples(input: TokenStream) -> syn::Result<TokenStream> {
    let Widths { callback, range, named, each } = syn::parse2(input)?;
    let invocations = range.map(|width| {
        let positions = match &each {
            Some(prefixes) => each_position(width, prefixes),
            None => positions(width),
        };
        if let Some(name) = &named {
            let name = format_ident!("{name}{width}");
            let width = Literal::usize_unsuffixed(width);
            quote!(#callback!(#name, #width; #positions);)
        } else {
            quote!(#callback!(#positions);)
        }
    });
    Ok(quote!(#(#invocations)*))
}

/// Invokes the callback once per position of each width, the position and its type first.
pub(crate) fn for_tuple_positions(input: TokenStream) -> syn::Result<TokenStream> {
    let Widths { callback, range, named, each } = syn::parse2(input)?;
    if let Some(clause) = named.map(|name| name.span()).or_else(|| each.and_then(|each| each.first().map(Ident::span)))
    {
        return Err(syn::Error::new(clause, "positions take no `named` or `each` clause"));
    }
    let invocations = range.flat_map(|width| {
        let positions = positions(width);
        let callback = callback.clone();
        (0..width).map(move |at| {
            let ty = format_ident!("A{}", at + 1);
            let at = Literal::usize_unsuffixed(at);
            quote!(#callback!(#at => #ty; #positions);)
        })
    });
    Ok(quote!(#(#invocations)*))
}

#[cfg(test)]
#[path = "tuples_test.rs"]
mod tuples_test;
