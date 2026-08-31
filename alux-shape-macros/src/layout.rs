//! The layout a shape declaration states.
//!
//! A declaration holds everything a layout needs: a member's name, and the type carrying it. Neither
//! is a shape, so neither is folded — both are read. What this emits is therefore a copy of what the
//! declaration already says, which is the point: the declaration is the source, and a struct beside it
//! would be the same thing said twice.
//!
//! This sits outside `#[ext(… defunc(via = shape))]` and re-emits it untouched, so the program and the
//! layout come from one statement. It belongs in that backend; until it is there, it is an attribute of
//! its own.

use crate::words;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::visit_mut::{self as visit_mut, VisitMut};
use syn::{
    Error, Expr, ExprMethodCall, FnArg, ImplItem, ImplItemFn, ItemImpl, LitStr, Result, Type, Visibility, parse2,
};

/// One member of the layout a declaration states.
struct Member {
    /// The identifier the declaration named it with, kept for its span.
    name: syn::Ident,
    /// The type carrying it.
    carrier: Type,
    /// Whether the member holds another layout's members rather than one of its own.
    merged: bool,
}

/// Collects the members a declaration's body states, in the order stated.
struct Members {
    found: Vec<Result<Member>>,
}

impl Visit<'_> for Members {
    fn visit_expr_method_call(&mut self, call: &ExprMethodCall) {
        let merged = call.method == "merge";

        if call.method == "field" || merged {
            self.found.push(read_member(call, merged));
        }

        visit::visit_expr_method_call(self, call);
    }
}

/// Reads one member from the call stating it.
fn read_member(call: &ExprMethodCall, merged: bool) -> Result<Member> {
    let carrier = match &call.turbofish {
        Some(turbofish) => match turbofish.args.first() {
            Some(syn::GenericArgument::Type(carrier)) => carrier.clone(),
            _ => {
                return Err(Error::new(
                    turbofish.span(),
                    "a member states the type carrying it, as `field::<Carrier>(name, shape)`",
                ));
            }
        },
        None => {
            return Err(Error::new(
                call.span(),
                "a member states the type carrying it, as `field::<Carrier>(name, shape)`; without \
                 one there is nothing to emit a field from",
            ));
        }
    };

    if merged {
        return Ok(Member { name: name_of_type(&carrier), carrier, merged });
    }

    match call.args.first() {
        Some(Expr::Path(path)) => match path.path.get_ident() {
            Some(name) => Ok(Member { name: name.clone(), carrier, merged }),
            None => Err(Error::new(path.span(), "a member is named by an identifier")),
        },
        Some(other) => Err(Error::new(other.span(), "a member is named by an identifier")),
        None => Err(Error::new(call.span(), "a member states a name")),
    }
}

/// Names a merged member after the type carrying it, since it states no name of its own.
fn name_of_type(carrier: &Type) -> syn::Ident {
    let text = quote!(#carrier).to_string();
    let last = text.rsplit(':').next().unwrap_or(&text).trim().to_owned();
    let words = words::of_camel(last.split('<').next().unwrap_or(&last).trim());

    format_ident!("{}", words.join("_"), span = carrier.span())
}

/// The layout one declaration states.
fn layout(method: &ImplItemFn, ext: &Option<syn::Path>, spelling: &str) -> Result<TokenStream> {
    if !method.sig.inputs.iter().any(|input| matches!(input, FnArg::Receiver(_))) {
        return Ok(TokenStream::new());
    }

    let name = method.sig.ident.to_string();
    let Some(stem) = name.strip_suffix("_shape") else {
        return Ok(TokenStream::new());
    };

    let mut members = Members { found: Vec::new() };
    members.visit_block(&method.block);

    let members = members.found.into_iter().collect::<Result<Vec<_>>>()?;
    let fields = members.iter().map(|member| {
        // The field is the identifier the declaration wrote, at the place it wrote it, so an editor
        // reaches one from the other.
        let name = syn::Ident::new(&member.name.to_string(), member.name.span());
        let carrier = &member.carrier;

        if member.merged { quote!(#[serde(flatten)] pub #name: #carrier) } else { quote!(pub #name: #carrier) }
    });

    let name = words::of_snake(stem).iter().map(|word| pascal(word)).collect::<String>();
    let layout = format_ident!("{name}", span = method.sig.ident.span());
    let declaration = &method.sig.ident;
    let spelling = LitStr::new(spelling, Span::call_site());
    let reach = ext.as_ref().map(|ext| quote!(use #ext as _;));

    Ok(quote! {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = #spelling)]
        pub struct #layout {
            #(#fields,)*
        }

        impl<Alg> ::alux_shape::ShapeOf<Alg> for #layout
        where
            Alg: ::alux_shape::ShapeAlg + ::alux_shape::FieldAlg,
        {
            type Shape = <Alg as ::alux_shape::Sorts>::Ty;

            /// The shape this layout carries is the one the declaration states.
            fn shape_of(alg: &Alg) -> Self::Shape {
                use ::alux_shape::ShapeProgramExt as _;
                #reach

                alg.compile_shape(alg.#declaration())
            }
        }
    })
}

/// Writes one word capitalised.
fn pascal(word: &str) -> String {
    let mut chars = word.chars();

    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// The extension trait the declarations are reached through, as `#[ext(name = …)]` states it.
fn extension(attrs: &[syn::Attribute]) -> Option<syn::Path> {
    let mut found = None;

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("ext")) {
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                found = meta.value().and_then(|value| value.parse()).ok();
            } else if meta.input.peek(syn::Token![=]) {
                let _: TokenStream = meta.value()?.parse()?;
            }

            Ok(())
        });
    }

    found
}

/// Rewrites each member's call onto the constructor stating what it states.
///
/// A declaration writes one name, `field`, with or without a type and with or without a shape. Rust
/// overloads neither a name nor a method's type parameter, so the runtime states three constructors and
/// the choice between them is made here.
struct Constructors;

impl VisitMut for Constructors {
    fn visit_expr_method_call_mut(&mut self, call: &mut ExprMethodCall) {
        if call.method == "field" {
            // A member's name reaches the term as the words it states.
            if let Some(Expr::Path(path)) = call.args.first()
                && let Some(name) = path.path.get_ident()
            {
                let words = words::of_snake(&name.to_string());
                let words = words.iter().map(|word| LitStr::new(word, name.span()));

                call.args[0] = syn::parse_quote!(&[#(#words),*]);
            }

            // The type a member states is the layout's, and the layout is emitted already. Where the
            // type states the shape as well, the constructor reading it keeps it.
            match (call.turbofish.is_some(), call.args.len()) {
                (true, 1) => call.method = format_ident!("field_of", span = call.method.span()),
                (true, _) => call.turbofish = None,
                (false, _) => (),
            }
        }

        // Merging states no member of its own, so a type stated for it is the layout's alone.
        if call.method == "merge" {
            call.turbofish = None;
        }

        visit_mut::visit_expr_method_call_mut(self, call);
    }
}

/// Emits the layouts a block of declarations states, and the block itself unchanged.
pub(crate) fn expand(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let spelling = if attr.is_empty() { "camelCase".to_owned() } else { parse2::<LitStr>(attr)?.value() };
    let (attrs, visibility, rest) = split_prefix(item)?;
    let mut block = parse2::<ItemImpl>(rest)?;
    let ext = extension(&attrs);

    let layouts = block
        .items
        .iter()
        .filter_map(|item| match item {
            ImplItem::Fn(method) => Some(layout(method, &ext, &spelling)),
            _ => None,
        })
        .collect::<Result<Vec<_>>>()?;

    Constructors.visit_item_impl_mut(&mut block);

    Ok(quote! {
        #(#layouts)*

        #(#attrs)*
        #visibility #block
    })
}

/// Splits what precedes the `impl`: the attributes the block carries, and a visibility the `ext`
/// attribute accepts where Rust would not.
fn split_prefix(item: TokenStream) -> Result<(Vec<syn::Attribute>, Option<Visibility>, TokenStream)> {
    let mut tokens = item.into_iter().peekable();
    let mut attrs = TokenStream::new();

    // Attributes first: a `#` and the group following it, for as many as there are.
    while let Some(proc_macro2::TokenTree::Punct(punct)) = tokens.peek() {
        if punct.as_char() != '#' {
            break;
        }

        attrs.extend(tokens.next());
        attrs.extend(tokens.next());
    }

    let mut visibility = TokenStream::new();

    if let Some(proc_macro2::TokenTree::Ident(ident)) = tokens.peek()
        && ident == "pub"
    {
        visibility.extend(tokens.next());

        if let Some(proc_macro2::TokenTree::Group(group)) = tokens.peek()
            && group.delimiter() == proc_macro2::Delimiter::Parenthesis
        {
            visibility.extend(tokens.next());
        }
    }

    let attrs = syn::parse::Parser::parse2(syn::Attribute::parse_outer, attrs)?;
    let visibility = if visibility.is_empty() { None } else { Some(parse2(visibility)?) };

    Ok((attrs, visibility, tokens.collect()))
}
