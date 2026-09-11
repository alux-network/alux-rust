//! Lowers a fluent program declaration into a first-order program type.
//!
//! The lowering is the same for every transport: each declaration method becomes a zero-sized
//! program type, the authored body becomes the program that type compiles, and the obligations
//! discovered in the body become the `where` clause of one interpretation. A backend supplies only
//! what its own transport means.

use crate::syntax::{
    ExtensionImpl, ReplaceSelf, Subprograms, documentation, method_names, predicates, program_ident,
    program_type_params, unbind,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::visit_mut::VisitMut;
use syn::{
    Block, Expr, ExprMethodCall, GenericParam, Ident, ImplItem, ImplItemFn, Stmt, Token, Visibility, WherePredicate,
    parse_quote,
};

/// Carries the parts of a lowered program that no backend chooses.
pub(crate) struct LoweredProgram {
    /// The program type, with its type arguments applied.
    pub(crate) program_type: TokenStream,
    /// The declaration's own generic parameters, interpreted alongside `This`.
    pub(crate) compiler_params: Punctuated<GenericParam, Token![,]>,
    /// Every obligation the declaration implies, in declaration order.
    pub(crate) predicates: Vec<TokenStream>,
    /// The rewritten body, read as the program it starts from and what it declares on top.
    pub(crate) chain: Chain,
}

/// A program body read as one program and the declarations applied to it.
///
/// A declaration is written as a chain, and a chain read as a type nests: `n` declarations make a
/// type of depth `n` whose interpretation proves one obligation per prefix. Read as this instead,
/// every declaration is one leaf of fixed size, and interpreting the program is a fold over the
/// leaves rather than over the nesting. That is the difference between a declaration costing its
/// method count and costing the square of it.
pub(crate) struct Chain {
    /// Whatever the body states before the program it ends with.
    pub(crate) leading: Vec<Stmt>,
    /// The program the chain starts from, which no link was recognized under.
    pub(crate) root: Expr,
    /// Each recognized declaration, as the leaf program it denotes, in authored order.
    pub(crate) leaves: Vec<TokenStream>,
}

/// Reads a program body as its root program and the leaves declared on top of it.
///
/// Descending stops at the first call the backend does not recognize, so an unrecognized body is
/// interpreted exactly as it was before: as one nested program. Nothing depends on recognizing
/// every shape an author can write.
fn split_chain<Backend>(block: &Block) -> syn::Result<Chain>
where
    Backend: ProgramBackendAlg,
{
    let Some((Stmt::Expr(program, None), leading)) = block.stmts.split_last() else {
        return Err(syn::Error::new_spanned(
            block,
            "a program declaration states the program it declares, so its body ends in that program",
        ));
    };
    let mut root = program;
    let mut leaves = Vec::new();
    while let Expr::MethodCall(call) = root {
        let Some(leaf) = Backend::read_link(call) else { break };
        leaves.push(leaf);
        root = &call.receiver;
    }
    leaves.reverse();

    Ok(Chain { leading: leading.to_vec(), root: root.clone(), leaves })
}

/// Describes what one transport contributes to the shared program lowering.
pub(crate) trait ProgramBackendAlg {
    /// What a program states once about every declaration it contains.
    type Defaults;

    /// Marks a method name whose call denotes a nested program of this transport.
    const NESTED_SUFFIX: &'static str;

    /// Explains a rejected generic parameter in this transport's vocabulary.
    const REJECTED_PARAM: &'static str;

    /// Adds the interpreter evidence implied by the declarations in one method body.
    fn require_declarations(method: &mut ImplItemFn, defaults: &Self::Defaults);

    /// States the obligation carried by a nested program value.
    fn require_subprogram(program: &TokenStream) -> TokenStream;

    /// Reads one chain call as the leaf program it declares, or nothing where it declares none.
    fn read_link(call: &ExprMethodCall) -> Option<TokenStream>;

    /// Emits the compilation meaning of one lowered program.
    fn compile_program(lowered: &LoweredProgram) -> TokenStream;
}

/// Expands a program declaration into its extension trait and its first-order programs.
pub(crate) fn expand_program<Backend>(
    attr: TokenStream,
    item: TokenStream,
    defaults: &Backend::Defaults,
) -> syn::Result<TokenStream>
where
    Backend: ProgramBackendAlg,
{
    let input = syn::parse2::<ExtensionImpl>(item)?;
    let visibility = input.item_visibility();
    let impl_predicates = input.impl_predicates();
    let mut extension = input.unbounded_item();
    let methods = method_names(&extension);
    let mut generated = Vec::new();
    for item in &mut extension.items {
        if let ImplItem::Fn(method) = item {
            let (constructor, program) =
                lower_program::<Backend>(method, &methods, &visibility, &impl_predicates, defaults)?;
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
    defaults: &Backend::Defaults,
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
    // A declaration that introduces no parameter defines a plain type, not one with an empty list.
    let parameters = if type_params.is_empty() { quote!() } else { quote!(<#(#type_params),*>) };

    // The authored method keeps its name and parameters but now returns the program value.
    let mut constructor = method.clone();
    constructor.sig.output = parse_quote!(-> #program_type);
    constructor.block = parse_quote!({ #program::default() });

    // The same method read again as the program's compilation: nested programs first, then the
    // evidence its own declarations require, then the builder that replaces `self`.
    let mut compiler = method.clone();
    let mut subprograms = Subprograms::new(methods, Backend::NESTED_SUFFIX);
    subprograms.visit_block_mut(&mut compiler.block);
    Backend::require_declarations(&mut compiler, defaults);
    ReplaceSelf.visit_block_mut(&mut compiler.block);

    // The bounds move to the interpretation; the parameters they introduce stay.
    let method_predicates = predicates(&compiler.sig.generics);
    unbind(&mut compiler.sig.generics);
    let obligations = impl_predicates
        .iter()
        .map(|predicate| quote!(#predicate))
        .chain(method_predicates.iter().map(|predicate| quote!(#predicate)))
        .chain(subprograms.programs().iter().map(Backend::require_subprogram))
        .collect();
    let lowered = LoweredProgram {
        program_type,
        compiler_params: compiler.sig.generics.params.clone(),
        predicates: obligations,
        chain: split_chain::<Backend>(&compiler.block)?,
    };
    let compile = Backend::compile_program(&lowered);
    let program_type = &lowered.program_type;
    let documentation = documentation(&method.attrs);

    let generated = quote! {
        #(#documentation)*
        #[doc(hidden)]
        #visibility struct #program #parameters (core::marker::PhantomData<#marker>);

        impl #parameters core::default::Default for #program_type {
            fn default() -> Self {
                Self(core::marker::PhantomData)
            }
        }

        #compile
    };
    Ok((constructor, generated))
}
