//! A member's own written form, as calls of the shape algebra.

use alux_shape_syn::Written;
use proc_macro2::TokenStream;
use quote::quote;
use syn::LitStr;

/// The algebra calls a written form states.
pub(crate) fn shape(written: &Written) -> TokenStream {
    match written {
        Written::Truth => quote!(alg.truth()),
        Written::Text => quote!(alg.text()),
        Written::Hex(len) => match len {
            Some(len) => quote!(alg.bytes_hex(Some(#len))),
            None => quote!(alg.bytes_hex(None)),
        },
        Written::Decimal(bits) => quote!(alg.int_decimal(false, #bits)),
        Written::Int(bits) => quote!(alg.int(false, #bits)),
        Written::Named(words, item) => {
            let item = shape(item);
            let words = words.iter().map(|word| LitStr::new(word, proc_macro2::Span::call_site()));

            quote!(alg.named(&[#(#words),*], #item))
        }
        Written::Seq(item) => {
            let item = shape(item);

            quote!(alg.seq(#item))
        }
        Written::Opt(item) => {
            let item = shape(item);

            quote!(alg.opt(#item))
        }
    }
}
