//! Lowers a fluent program declaration into a first-order program type.
//!
//! The lowering is the same for every transport: each declaration method becomes a zero-sized
//! program type, the authored body becomes the program that type compiles, and the obligations
//! discovered in the body become the `where` clause of one interpretation. A backend supplies only
//! what its own transport means.

use crate::syntax::{ExtensionImpl, ReplaceSelf, Subprograms, method_names, program_ident, program_type_params};
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::visit_mut::VisitMut;
use syn::{Block, GenericParam, Ident, ImplItem, ImplItemFn, Token, Visibility, WherePredicate, parse_quote};

/// Carries the parts of a lowered program that no backend chooses.
pub(crate) struct LoweredProgram {
    /// The program type, with its type arguments applied.
    pub(crate) program_type: TokenStream,
    /// The declaration's own generic parameters, interpreted alongside `This`.
    pub(crate) compiler_params: Punctuated<GenericParam, Token![,]>,
    /// Every obligation the declaration implies, in declaration order.
    pub(crate) predicates: Vec<TokenStream>,
    /// The rewritten body, building the first-order program from a builder named `builder`.
    pub(crate) body: Block,
}

/// Describes what one transport contributes to the shared program lowering.
pub(crate) trait ProgramBackendAlg {
    /// Marks a method name whose call denotes a nested program of this transport.
    const NESTED_SUFFIX: &'static str;

    /// Explains a rejected generic parameter in this transport's vocabulary.
    const REJECTED_PARAM: &'static str;

    /// Adds the interpreter evidence implied by the declarations in one method body.
    fn require_declarations(method: &mut ImplItemFn);

    /// States the obligation carried by a nested program value.
    fn require_subprogram(program: &TokenStream) -> TokenStream;

    /// Emits the compilation meaning of one lowered program.
    fn compile_program(lowered: &LoweredProgram) -> TokenStream;
}

/// Expands a program declaration into its extension trait and its first-order programs.
pub(crate) fn expand_program<Backend>(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream>
where
    Backend: ProgramBackendAlg,
{
    let input = syn::parse2::<ExtensionImpl>(item)?;
    let visibility = input.item_visibility();
    let impl_predicates = input.impl_predicates();
    let mut extension = input.item.clone();
    extension.generics.where_clause = None;
    let methods = method_names(&extension);
    let mut generated = Vec::new();
    for item in &mut extension.items {
        if let ImplItem::Fn(method) = item {
            let (constructor, program) = lower_program::<Backend>(method, &methods, &visibility, &impl_predicates)?;
            *method = constructor;
            generated.push(program);
        }
    }
    let forwarded = input.forwarded(attr);

    Ok(quote! {
        #[::alux_ext::extend::ext(#forwarded)]
        #extension
        #(#generated)*
    })
}

/// Replaces one declaration method by a program constructor and generates that program.
fn lower_program<Backend>(
    method: &ImplItemFn,
    methods: &[Ident],
    visibility: &Visibility,
    impl_predicates: &Punctuated<WherePredicate, Token![,]>,
) -> syn::Result<(ImplItemFn, TokenStream)>
where
    Backend: ProgramBackendAlg,
{
    let program = program_ident(&method.sig.ident);
    let type_params = program_type_params(method, Backend::REJECTED_PARAM)?;
    let type_arguments = type_params.iter().map(|param| &param.ident).collect::<Vec<_>>();
    let program_type =
        if type_arguments.is_empty() { quote!(#program) } else { quote!(#program<#(#type_arguments),*>) };
    let marker = if type_arguments.is_empty() { quote!(()) } else { quote!(fn() -> (#(#type_arguments,)*)) };

    // The authored method keeps its name and parameters but now returns the program value.
    let mut constructor = method.clone();
    constructor.sig.output = parse_quote!(-> #program_type);
    constructor.block = parse_quote!({ #program::default() });

    // The same method read again as the program's compilation: nested programs first, then the
    // evidence its own declarations require, then the builder that replaces `self`.
    let mut compiler = method.clone();
    let mut subprograms = Subprograms::new(methods, Backend::NESTED_SUFFIX);
    subprograms.visit_block_mut(&mut compiler.block);
    Backend::require_declarations(&mut compiler);
    ReplaceSelf.visit_block_mut(&mut compiler.block);

    let method_predicates =
        compiler.sig.generics.where_clause.as_ref().map(|clause| clause.predicates.clone()).unwrap_or_default();
    let predicates = impl_predicates
        .iter()
        .map(|predicate| quote!(#predicate))
        .chain(method_predicates.iter().map(|predicate| quote!(#predicate)))
        .chain(subprograms.programs().iter().map(Backend::require_subprogram))
        .collect();
    let lowered = LoweredProgram {
        program_type,
        compiler_params: compiler.sig.generics.params.clone(),
        predicates,
        body: compiler.block,
    };
    let compile = Backend::compile_program(&lowered);
    let program_type = &lowered.program_type;

    let generated = quote! {
        #[doc(hidden)]
        #visibility struct #program<#(#type_params),*>(core::marker::PhantomData<#marker>);

        impl<#(#type_params),*> core::default::Default for #program_type {
            fn default() -> Self {
                Self(core::marker::PhantomData)
            }
        }

        #compile
    };
    Ok((constructor, generated))
}
