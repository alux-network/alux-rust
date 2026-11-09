//! The `laws` backend of `#[ext(name = XLaws, defunc(via = laws))]`.
//!
//! Each method is a law over the block's subject, lowered to an operation `Op_<law>` that states its
//! name through `OperationAlg` and its check through `alux_laws::LawOp`. The block also states
//! `x_laws()`, the suite of those operations in block order, beside the extension rather than in it,
//! so a call needs no subject: the run that reads the suite supplies one.

use crate::extension::{extension, stated};
use crate::syntax::{ExtensionImpl, doc_text, operation_ident, predicates, program_ident, unbind};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::visit_mut::{self, VisitMut};
use syn::{
    FnArg, GenericParam, Generics, Ident, ImplItem, ImplItemFn, ItemImpl, Meta, Pat, Receiver, ReceiverKind, Token,
    Type, TypeParamBound, Visibility, WherePredicate,
};

/// One law of the block, as the backend reads it.
struct Law {
    ident: Ident,
    doc: String,
    mutable: bool,
    /// The `I` of the `&I` argument and the argument's name, when the law takes one.
    input: Option<(Type, Ident)>,
}

/// Lowers a suite block into its operations, its suite function, and its extension.
pub(crate) fn laws_program_defunc_internal(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let arguments = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(attr)?.into_iter().collect::<Vec<_>>();
    if let Some(then) = arguments.iter().find(|argument| argument.path().is_ident(crate::lower::THEN)) {
        return Err(syn::Error::new_spanned(then, "`via = laws` lowers a block alone: a suite is no other program"));
    }
    if !arguments.iter().any(|argument| argument.path().is_ident("name")) {
        return Err(syn::Error::new(
            Span::call_site(),
            "a laws suite needs a name, which its reports and its suite function are named after: \
             `#[ext(name = XLaws, defunc(via = laws))]`",
        ));
    }
    let mut input = syn::parse2::<ExtensionImpl>(item)?;
    let item = &mut input.item;
    if item.trait_.is_some() {
        return Err(syn::Error::new_spanned(&item.self_ty, "a laws suite is an inherent `impl`"));
    }
    if let Some(param) = item.generics.params.iter().find(|param| !matches!(param, GenericParam::Type(_))) {
        return Err(syn::Error::new_spanned(param, "a laws suite takes only type parameters"));
    }

    let (laws, errors): (Vec<_>, Vec<_>) = item
        .items
        .iter()
        .filter_map(|item| match item {
            ImplItem::Fn(method) => Some(read_law(method)),
            _ => None,
        })
        .partition(Result::is_ok);
    if let Some(error) = errors.into_iter().filter_map(Result::err).reduce(|mut all, error| {
        all.combine(error);
        all
    }) {
        return Err(error);
    }
    let laws = laws
        .into_iter()
        .filter_map(Result::ok)
        .map(|mut law| {
            // An operation is declared outside the block, where `Self` means nothing.
            if let Some((ty, _)) = &mut law.input {
                AsSubject(&item.self_ty).visit_type_mut(ty);
            }
            law
        })
        .collect::<Vec<_>>();

    let visibility = input.item_visibility();
    let stated = stated(&arguments, &input.item)?;
    let lowered = Lowered { name: &stated.name, visibility: &visibility, item: &input.item };
    let operations = laws.iter().map(|law| lowered.operation(law)).collect::<Vec<_>>();
    let suite = lowered.suite(&laws);
    let extension = extension(&visibility, stated, &input.item)?;

    Ok(quote! {
        #extension
        #(#operations)*
        #suite
    })
}

/// Reads a law of the block.
fn read_law(method: &ImplItemFn) -> syn::Result<Law> {
    let sig = &method.sig;
    let ident = sig.ident.clone();
    let refuse = |span: &dyn ToTokens, why: &str| Err(syn::Error::new_spanned(span, format!("`{ident}`: {why}")));

    if let Some(asyncness) = &sig.asyncness {
        return refuse(asyncness, "a law is not async");
    }
    if !sig.generics.params.is_empty() {
        return refuse(&sig.generics, "a law takes no generic parameters of its own");
    }
    let mut arguments = sig.inputs.iter();
    let mutable = match arguments.next() {
        Some(FnArg::Receiver(Receiver { kind: ReceiverKind::Reference(_, _, mutability), .. })) => mutability.is_some(),
        Some(FnArg::Receiver(receiver)) => {
            return refuse(receiver, "a law borrows its subject: take `&self` or `&mut self`");
        }
        _ => return refuse(&sig.ident, "a law takes `&self` or `&mut self`"),
    };
    let input = match arguments.next() {
        None => None,
        Some(FnArg::Typed(typed)) => match &*typed.ty {
            Type::Reference(borrowed) if borrowed.mutability.is_none() => {
                let named = match &*typed.pat {
                    Pat::Ident(named) => named.ident.clone(),
                    _ => format_ident!("input"),
                };
                Some(((*borrowed.elem).clone(), named))
            }
            other => return refuse(other, "a law's input is borrowed: take `&Input`"),
        },
        Some(other) => return refuse(other, "a law's input is borrowed: take `&Input`"),
    };
    if let Some(extra) = arguments.next() {
        return refuse(extra, "a law takes at most one input, `&Input`");
    }
    Ok(Law { ident, doc: doc_text(&method.attrs), mutable, input })
}

/// Replaces `Self` in a signature type by the subject.
struct AsSubject<'a>(&'a Type);

impl VisitMut for AsSubject<'_> {
    fn visit_type_mut(&mut self, ty: &mut Type) {
        if let Type::Path(named) = ty
            && named.qself.is_none()
            && named.path.segments.first().is_some_and(|segment| segment.ident == "Self")
        {
            let subject = self.0;
            let rest = named.path.segments.iter().skip(1).collect::<Vec<_>>();
            // `This::Key` for a parameter subject, as the block writes it; `<Corpus<T>>::Key`
            // otherwise, where a bare path would not resolve.
            let parameter =
                matches!(subject, Type::Path(path) if path.qself.is_none() && path.path.get_ident().is_some());
            *ty = if rest.is_empty() {
                subject.clone()
            } else if parameter {
                syn::parse_quote!(#subject #(:: #rest)*)
            } else {
                syn::parse_quote!(<#subject> #(:: #rest)*)
            };
        }
        visit_mut::visit_type_mut(self, ty);
    }
}

/// What every generated item of one block shares.
struct Lowered<'a> {
    name: &'a Ident,
    visibility: &'a Visibility,
    item: &'a ItemImpl,
}

impl Lowered<'_> {
    /// `Op_<law>` over the block's type parameters, with no bounds: it states which law, nothing
    /// more. Its `OperationAlg` and `LawOp` impls carry the block's bounds.
    fn operation(&self, law: &Law) -> TokenStream {
        let Self { name, visibility, item } = self;
        let op = operation_ident(&law.ident);
        let subject = &item.self_ty;
        let params = item.generics.type_params().map(|param| &param.ident).collect::<Vec<_>>();
        let unbound = unbound_but_unsized(&item.generics);
        let (unbound_impl, _, _) = unbound.split_for_impl();
        let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();
        let Law { ident, doc, .. } = law;
        let (input, args, arg_names, argument) = match &law.input {
            Some((ty, named)) => (quote!(#ty), quote!((#ty,)), quote!(&[stringify!(#named)]), quote!(input)),
            None => (quote!(()), quote!(()), quote!(&[]), quote!(_input)),
        };
        let receiver = if law.mutable { quote!(subject) } else { quote!(&*subject) };
        let passed = law.input.as_ref().map(|_| quote!(, input));
        let op_doc = format!("The law `{name}::{ident}`, as an operation.");

        quote! {
            #[doc = #op_doc]
            #[allow(non_camel_case_types)]
            #visibility struct #op #unbound_impl(::core::marker::PhantomData<fn() -> (#(*const #params,)*)>);

            impl #unbound_impl ::core::default::Default for #op #ty_generics {
                fn default() -> Self {
                    Self(::core::marker::PhantomData)
                }
            }

            impl #impl_generics ::alux_ext::OperationAlg for #op #ty_generics #where_clause {
                type Context = #subject;
                type Args = #args;

                const DOC: &'static str = #doc;
                const NAME: &'static str = stringify!(#ident);
                const ARG_NAMES: &'static [&'static str] = #arg_names;
            }

            impl #impl_generics ::alux_laws::LawOp<#subject> for #op #ty_generics #where_clause {
                type Input = #input;

                fn check(&self, subject: &mut #subject, #argument: &#input) -> ::core::result::Result<(), ::alux_laws::Failure> {
                    ::alux_laws::IntoLawOutcome::into_law_outcome(<#subject as #name #ty_generics>::#ident(#receiver #passed))
                }
            }
        }
    }

    /// `Program_x_laws`, the law program of the suite named `XLaws`, and `x_laws()`, which returns
    /// it. The program folds each law, in block order, through `LawAlg::finish_law`, and asks of its
    /// compiler one `LawAlg` per input type, as a JSON-RPC program asks one method algebra per
    /// signature. The function takes the block's parameters unbounded, so a run infers the subject
    /// from where it is used.
    fn suite(&self, laws: &[Law]) -> TokenStream {
        let Self { name, visibility, item } = self;
        let function = suite_ident(name);
        let program = program_ident(&function);
        let subject = &item.self_ty;
        let params = item.generics.type_params().map(|param| &param.ident).collect::<Vec<_>>();
        let unbound = unbound_but_unsized(&item.generics);
        let (unbound_impl, ty_generics, _) = unbound.split_for_impl();
        let mut compiling = item.generics.clone();
        compiling.params.push(syn::parse_quote!(Compiler));
        let mut inputs: Vec<TokenStream> = Vec::new();
        for law in laws {
            let input = law.input.as_ref().map_or_else(|| quote!(()), |(ty, _)| quote!(#ty));
            if !inputs.iter().any(|seen| seen.to_string() == input.to_string()) {
                inputs.push(input);
            }
        }
        let where_clause = compiling.make_where_clause();
        where_clause.predicates.push(syn::parse_quote!(Compiler: ::alux_laws::LawsAlg));
        for input in &inputs {
            where_clause.predicates.push(syn::parse_quote!(Compiler: ::alux_laws::LawAlg<#subject, #input>));
        }
        let (impl_generics, _, where_clause) = compiling.split_for_impl();
        let folded = laws.iter().map(|law| {
            let op = operation_ident(&law.ident);
            let input = law.input.as_ref().map_or_else(|| quote!(()), |(ty, _)| quote!(#ty));
            quote! {
                let laws = ::alux_laws::LawsAlg::laws_merge(
                    compiler,
                    laws,
                    ::alux_laws::LawAlg::<#subject, #input>::finish_law(
                        compiler,
                        stringify!(#name),
                        <#op #ty_generics as ::core::default::Default>::default(),
                    ),
                );
            }
        });
        let program_doc = format!("The law program of the `{name}` suite.");
        let doc = format!("The `{name}` suite: its laws in block order, for any interpreter to read.");

        quote! {
            #[doc = #program_doc]
            #[allow(non_camel_case_types)]
            #visibility struct #program #unbound_impl(::core::marker::PhantomData<fn() -> (#(*const #params,)*)>);

            impl #unbound_impl ::core::default::Default for #program #ty_generics {
                fn default() -> Self {
                    Self(::core::marker::PhantomData)
                }
            }

            impl #unbound_impl ::alux_laws::LawsProgram for #program #ty_generics {}

            impl #impl_generics ::alux_laws::LawsProgramAlg<Compiler> for #program #ty_generics #where_clause {
                type Laws = <Compiler as ::alux_laws::LawsAlg>::Laws;

                fn compile_laws(&self, compiler: &Compiler) -> Self::Laws {
                    let laws = ::alux_laws::LawsAlg::laws_empty(compiler);
                    #(#folded)*
                    laws
                }
            }

            #[doc = #doc]
            #[must_use]
            #visibility fn #function #unbound_impl() -> #program #ty_generics {
                <#program #ty_generics as ::core::default::Default>::default()
            }
        }
    }
}

/// Removes every bound from `generics` but `?Sized`, which a generated type or function needs to
/// accept the unsized types the block accepts.
fn unbound_but_unsized(generics: &Generics) -> Generics {
    let relaxed = predicates(generics)
        .into_iter()
        .filter_map(|predicate| match predicate {
            WherePredicate::Type(bound)
                if bound
                    .bounds
                    .iter()
                    .any(|bound| matches!(bound, TypeParamBound::Trait(bound) if bound.maybe.is_some())) =>
            {
                match bound.bounded_ty {
                    Type::Path(bounded) if bounded.qself.is_none() => bounded.path.get_ident().cloned(),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut unbound = generics.clone();
    unbind(&mut unbound);
    for param in unbound.type_params_mut().filter(|param| relaxed.contains(&param.ident)) {
        param.bounds.push(syn::parse_quote!(?Sized));
    }
    unbound
}

/// Names the function stating a suite: its trait name in snake case, `CounterLaws` naming
/// `counter_laws`.
fn suite_ident(name: &Ident) -> Ident {
    let mut snake = String::new();
    for (at, character) in name.to_string().char_indices() {
        if character.is_uppercase() && at > 0 {
            snake.push('_');
        }
        snake.extend(character.to_lowercase());
    }
    Ident::new(&snake, name.span())
}

#[cfg(test)]
#[path = "laws_program_test.rs"]
mod laws_program_test;
