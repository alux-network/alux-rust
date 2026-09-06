#![doc = include_str!("../README.md")]

mod layout;
mod shape;
mod words;
mod written;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

/// States `ShapeOf` for a layout, reading its members and their names from the type.
#[proc_macro_derive(Shape, attributes(written))]
pub fn derive_shape(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    shape::expand(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// Emits the layouts a block of shape declarations states, beside the declarations themselves.
///
/// Written outside `#[ext(… defunc(via = shape))]`, which it re-emits untouched:
///
/// ```rust ignore
/// #[shape_layout]
/// #[ext(name = UserShapeExt, defunc(via = shape))]
/// pub impl<This> This
/// where
///     This: ShapeAlg + FieldAlg,
/// {
///     fn user_shape(&self) {
///         self.record().field::<String>(display_name, self.text())
///     }
/// }
/// ```
///
/// Each member states the type carrying it, which is what a layout is emitted from. An optional
/// argument states how the layout writes its names, defaulting to `"camelCase"`.
///
/// The emitted layout derives `serde::Serialize` and `serde::Deserialize`, so a crate using this needs
/// `serde` among its dependencies.
#[proc_macro_attribute]
pub fn shape_layout(attr: TokenStream, item: TokenStream) -> TokenStream {
    layout::expand(attr.into(), item.into()).unwrap_or_else(syn::Error::into_compile_error).into()
}
