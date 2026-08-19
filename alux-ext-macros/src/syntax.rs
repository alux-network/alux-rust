//! Reads and rewrites the authored extension syntax shared by every backend.
//!
//! Nothing here decides what a declaration means. These items only recognize the shapes an author
//! writes — the visibility before an `impl`, the naming rule for generated types, the `op(...)`
//! handler position, a nested program call, and `self` inside a program body — so that each backend
//! states only its own meaning.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::visit_mut::{self, VisitMut};
use syn::{
    Expr, GenericParam, Ident, ImplItem, ImplItemFn, ItemImpl, Token, Type, TypeParam, TypePath, Visibility,
    WherePredicate, parse_quote,
};

/// Parses the visibility syntax accepted by `extend::ext` before an inherent impl.
pub(crate) struct ExtensionImpl {
    pub(crate) visibility: Option<Visibility>,
    pub(crate) item: ItemImpl,
}

impl Parse for ExtensionImpl {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attrs = syn::Attribute::parse_outer(input)?;
        let visibility =
            input.parse::<Visibility>().ok().filter(|visibility| !matches!(visibility, Visibility::Inherited));
        let mut item = input.parse::<ItemImpl>()?;
        item.attrs.extend(attrs);
        Ok(Self { visibility, item })
    }
}

impl ExtensionImpl {
    /// Returns the visibility carried by generated items.
    pub(crate) fn item_visibility(&self) -> Visibility {
        self.visibility.clone().unwrap_or(Visibility::Inherited)
    }

    /// Returns the semantic dependencies stated by the impl's `where` clause.
    pub(crate) fn impl_predicates(&self) -> Punctuated<WherePredicate, Token![,]> {
        self.item.generics.where_clause.as_ref().map(|clause| clause.predicates.clone()).unwrap_or_default()
    }

    /// Returns the attribute arguments forwarded to `extend::ext`, authored visibility first.
    pub(crate) fn forwarded(&self, attr: TokenStream) -> TokenStream {
        match &self.visibility {
            Some(visibility) => quote!(#visibility, #attr),
            None => attr,
        }
    }
}

/// Returns the names of every method the extension declares.
pub(crate) fn method_names(item: &ItemImpl) -> Vec<Ident> {
    item.items
        .iter()
        .filter_map(|item| match item {
            ImplItem::Fn(method) => Some(method.sig.ident.clone()),
            _ => None,
        })
        .collect()
}

/// Derives a generated type name from an authored method name.
pub(crate) fn pascal_ident(name: &Ident, suffix: &str) -> Ident {
    let mut pascal = String::new();
    for part in name.to_string().split('_') {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            pascal.extend(first.to_uppercase());
            pascal.extend(chars);
        }
    }
    format_ident!("{pascal}{suffix}")
}

/// Names the first-order operation reifying one method's application.
pub(crate) fn operation_ident(method: &Ident) -> Ident {
    pascal_ident(method, "Operation")
}

/// Names the first-order program produced by one declaration method.
pub(crate) fn program_ident(method: &Ident) -> Ident {
    pascal_ident(method, "Program")
}

/// Returns the program's type parameters, rejecting parameters a program cannot carry.
pub(crate) fn program_type_params(method: &ImplItemFn, rejected: &str) -> syn::Result<Vec<TypeParam>> {
    method
        .sig
        .generics
        .params
        .iter()
        .map(|param| match param {
            GenericParam::Type(param) => Ok(param.clone()),
            _ => Err(syn::Error::new_spanned(param, rejected)),
        })
        .collect()
}

/// Reads an already first-order handler written as `Operation::<Alg>::default()`.
fn first_order_type(expression: &Expr) -> Option<Type> {
    let Expr::Call(call) = expression else { return None };
    if !call.args.is_empty() {
        return None;
    }
    let Expr::Path(function) = call.func.as_ref() else { return None };
    let mut path = function.path.clone();
    if path.segments.last()?.ident != "default" {
        return None;
    }
    path.segments.pop();
    path.segments.pop_punct();

    Some(Type::Path(TypePath { attrs: Vec::new(), qself: function.qself.clone(), path }))
}

/// Returns the operation denoted by the `op(...)` in a declaration, reifying a named method.
///
/// A handler written as a method path is replaced by its generated operation value; a handler that is
/// already first-order is left as authored. The declaration is unchanged when it denotes no
/// operation.
pub(crate) fn lift_operation(declaration: &mut Expr) -> Option<Type> {
    let mut current = declaration;
    loop {
        let Expr::MethodCall(call) = current else { return None };
        if call.method == "op" {
            let handler = call.args.first()?;
            let (operation, reified) = match handler {
                Expr::Path(handler) => {
                    let method = handler.path.segments.last()?.ident.clone();
                    let operation = operation_ident(&method);
                    (parse_quote!(#operation<Alg>), true)
                }
                handler => (first_order_type(handler)?, false),
            };
            if reified {
                let argument: Expr = parse_quote!(<#operation>::default());
                call.args = Punctuated::from_iter([argument]);
            }
            return Some(operation);
        }
        current = &mut call.receiver;
    }
}

/// Collects and rewrites the calls that denote nested programs.
pub(crate) struct Subprograms<'a> {
    methods: &'a [Ident],
    suffix: &'static str,
    programs: Vec<TokenStream>,
}

impl<'a> Subprograms<'a> {
    /// Recognizes calls to `methods` and to any method whose name ends with `suffix`.
    pub(crate) fn new(methods: &'a [Ident], suffix: &'static str) -> Self {
        Self { methods, suffix, programs: Vec::new() }
    }

    /// Returns the nested program types found so far.
    pub(crate) fn programs(&self) -> &[TokenStream] {
        &self.programs
    }
}

impl VisitMut for Subprograms<'_> {
    fn visit_expr_mut(&mut self, expression: &mut Expr) {
        visit_mut::visit_expr_mut(self, expression);
        let Expr::MethodCall(call) = expression else { return };
        if !(self.methods.contains(&call.method) || call.method.to_string().ends_with(self.suffix))
            || !matches!(call.receiver.as_ref(), Expr::Path(path) if path.path.is_ident("self"))
        {
            return;
        }
        let program = program_ident(&call.method);
        let arguments = call.turbofish.as_ref().map(|arguments| arguments.args.clone()).unwrap_or_default();
        let program_type = if arguments.is_empty() { quote!(#program) } else { quote!(#program<#arguments>) };
        self.programs.push(program_type);
        let call = call.clone();
        *expression = parse_quote!(self.program(#call));
    }
}

/// Retargets authored `self` calls to the program builder used during compilation.
pub(crate) struct ReplaceSelf;

impl VisitMut for ReplaceSelf {
    fn visit_expr_mut(&mut self, expression: &mut Expr) {
        if matches!(expression, Expr::Path(path) if path.path.is_ident("self")) {
            *expression = parse_quote!(builder);
        } else {
            visit_mut::visit_expr_mut(self, expression);
        }
    }
}
