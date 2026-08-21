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
    Expr, GenericArgument, GenericParam, Generics, Ident, ImplItem, ImplItemFn, ItemImpl, PathArguments, Token, Type,
    TypeParam, TypePath, Visibility, WherePredicate, parse_quote,
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

    /// Returns the semantic dependencies the impl states, however they are written.
    pub(crate) fn impl_predicates(&self) -> Punctuated<WherePredicate, Token![,]> {
        predicates(&self.item.generics)
    }

    /// Returns the extension declaration with its semantic dependencies removed.
    ///
    /// They move to the program's interpretation, so the method that merely names a program stays
    /// callable without them.
    pub(crate) fn unbounded_item(&self) -> ItemImpl {
        let mut item = self.item.clone();
        unbind(&mut item.generics);

        item
    }

    /// Returns the attribute arguments forwarded to `extend::ext`, authored visibility first.
    pub(crate) fn forwarded(&self, attr: TokenStream) -> TokenStream {
        match &self.visibility {
            Some(visibility) => quote!(#visibility, #attr),
            None => attr,
        }
    }
}

/// Reads every bound a declaration states, whether on a generic parameter or in a `where` clause.
///
/// A bound means the same thing in both places, so a lowering that treats them alike lets an author
/// choose either spelling.
pub(crate) fn predicates(generics: &Generics) -> Punctuated<WherePredicate, Token![,]> {
    let mut predicates: Punctuated<WherePredicate, Token![,]> = generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(param) if !param.bounds.is_empty() => {
                let (parameter, bounds) = (&param.ident, &param.bounds);
                let predicate: WherePredicate = parse_quote!(#parameter: #bounds);
                Some(predicate)
            }
            _ => None,
        })
        .collect();
    if let Some(clause) = &generics.where_clause {
        predicates.extend(clause.predicates.clone());
    }

    predicates
}

/// Removes every bound from a declaration, leaving the parameters it introduces.
///
/// Bounds belong to the interpretation that needs them, never to the definition of a program type.
pub(crate) fn unbind(generics: &mut Generics) {
    generics.where_clause = None;
    for param in &mut generics.params {
        if let GenericParam::Type(param) = param {
            param.bounds.clear();
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
    format_ident!("{pascal}{suffix}", span = name.span())
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
            GenericParam::Type(param) => {
                let mut param = param.clone();
                param.bounds.clear();
                Ok(param)
            }
            _ => Err(syn::Error::new_spanned(param, rejected)),
        })
        .collect()
}

/// Names the operation a declaration denotes, together with the domain it is written against.
///
/// Both idents come from the authored source, so the expansion carries the author's spans and the
/// author's parameter name rather than one this macro invented.
pub(crate) struct Reified {
    /// The operation type to store in the program.
    pub(crate) operation: Type,
    /// The domain the operation interprets, as the author named it.
    pub(crate) carrier: Ident,
}

/// Reads the domain out of an already first-order handler's type arguments.
fn carrier_of(operation: &Type) -> Option<Ident> {
    let Type::Path(path) = operation else { return None };
    let PathArguments::AngleBracketed(arguments) = &path.path.segments.last()?.arguments else {
        return None;
    };
    arguments.args.iter().find_map(|argument| match argument {
        GenericArgument::Type(Type::Path(carrier)) => Some(carrier.path.segments.last()?.ident.clone()),
        _ => None,
    })
}

/// Reads an already first-order handler written as `Operation::<Domain>::default()`.
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
pub(crate) fn lift_operation(declaration: &mut Expr) -> Option<Reified> {
    let mut current = declaration;
    loop {
        let Expr::MethodCall(call) = current else { return None };
        if call.method == "op" {
            let handler = call.args.first()?;
            let (operation, carrier, reify) = match handler {
                Expr::Path(handler) => {
                    // `Domain::method` names both halves; reusing them keeps the expansion hygienic.
                    let mut segments = handler.path.segments.iter().rev();
                    let method = segments.next()?.ident.clone();
                    let carrier = segments.next()?.ident.clone();
                    let operation = operation_ident(&method);
                    (parse_quote!(#operation<#carrier>), carrier, true)
                }
                handler => {
                    let operation = first_order_type(handler)?;
                    let carrier = carrier_of(&operation)?;
                    (operation, carrier, false)
                }
            };
            if reify {
                let argument: Expr = parse_quote!(<#operation>::default());
                call.args = Punctuated::from_iter([argument]);
            }
            return Some(Reified { operation, carrier });
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
