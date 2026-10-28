//! Lowers a fluent program declaration into a first-order program type.
//!
//! The lowering is the same for every transport: each declaration method becomes a zero-sized
//! program type and the authored body becomes the program that type interprets. Each backend states
//! the interpretation evidence its declarations need and the obligation a nested program places on
//! its parent. A backend supplies only what its own transport means.

use crate::extension::{extension, stated};
use crate::syntax::{
    ExtensionImpl, ReplaceSelf, Subprograms, documentation, predicates, program_ident, program_type_params, unbind,
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::visit_mut::VisitMut;
use syn::{
    Block, Expr, ExprMethodCall, GenericParam, Ident, ImplItem, ImplItemFn, Local, LocalInit, Meta, Pat, PatIdent,
    Path, Stmt, Token, Type, Visibility, WherePredicate, parse_quote,
};

/// Carries the parts of a lowered program that no backend chooses.
pub(crate) struct LoweredProgram {
    /// The program type, with its type arguments applied.
    pub(crate) program_type: TokenStream,
    /// The declaration's own generic parameters, interpreted alongside `This`.
    pub(crate) compiler_params: Punctuated<GenericParam, Token![,]>,
    /// The interpretation obligations, in declaration order.
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

    /// Names the call on `self` that starts a declaration of this transport.
    const ROOT: &'static str;

    /// Explains a rejected generic parameter in this transport's vocabulary.
    const REJECTED_PARAM: &'static str;

    /// Whether this backend derives nested-program obligations from the body.
    const INFER_SUBPROGRAM_BOUNDS: bool = true;

    /// Whether a declaration may name its parts in `let` bindings, single-assignment as in A-normal
    /// form, which the lowering reads as if each were written where its name is used.
    const READS_BINDINGS: bool = false;

    /// Whether this backend states its nested programs' obligations in `prepare_declarations`,
    /// because a nested program's interpreter depends on where in the body it is written.
    const STATES_SUBPROGRAM_BOUNDS: bool = false;

    /// Prepares operation references and any backend-specific interpretation evidence, answering
    /// the items it states beside the program, such as names for the types that evidence repeats.
    fn prepare_declarations(method: &mut ImplItemFn, defaults: &Self::Defaults) -> TokenStream;

    /// States the obligation carried by a nested program value.
    fn require_subprogram(program: &TokenStream) -> TokenStream;

    /// Reads one chain call as the leaf program it declares, or nothing where it declares none.
    fn read_link(call: &ExprMethodCall) -> Option<TokenStream>;

    /// Emits the compilation meaning of one lowered program.
    fn compile_program(lowered: &LoweredProgram) -> TokenStream;
}

/// Names the argument through which `ext` hands the backends still to apply.
///
/// `defunc(via = a, via = b)` is cumulative: each backend lowers the declarations written in its
/// own vocabulary and passes the block on, and the last one states the extension once.
pub(crate) const THEN: &str = "__alux_then";

/// Separates the backends still to apply from the arguments the extension reads.
///
/// `None` means a lone backend, which lowers every method as before; `Some` means a chain, in which
/// a backend lowers only the declarations it recognizes.
fn split_then(attr: TokenStream) -> syn::Result<(Vec<Meta>, Option<Vec<Path>>)> {
    let mut arguments = Vec::new();
    let mut then = None;
    for argument in Punctuated::<Meta, Token![,]>::parse_terminated.parse2(attr)? {
        match &argument {
            Meta::List(list) if list.path.is_ident(THEN) => {
                then = Some(
                    Punctuated::<Path, Token![,]>::parse_terminated.parse2(list.tokens.clone())?.into_iter().collect(),
                );
            }
            _ => arguments.push(argument),
        }
    }

    Ok((arguments, then))
}

/// Reports whether a declaration starts from this transport's root call on `self`.
fn declares<Backend>(method: &ImplItemFn) -> bool
where
    Backend: ProgramBackendAlg,
{
    let Some(Stmt::Expr(current, None)) = method.block.stmts.last() else { return false };
    let mut current = current;
    while let Expr::MethodCall(call) = current {
        if matches!(call.receiver.as_ref(), Expr::Path(path) if path.path.is_ident("self")) {
            return call.method == Backend::ROOT;
        }
        current = call.receiver.as_ref();
    }

    false
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
    let (arguments, then) = split_then(attr)?;
    let input = syn::parse2::<ExtensionImpl>(item)?;
    let visibility = input.item_visibility();
    let impl_predicates = input.impl_predicates();
    // A chain keeps the authored bounds for the backends after this one; the last one removes them.
    let mut block = if then.is_some() { input.item.clone() } else { input.unbounded_item() };
    let mut generated = Vec::new();
    for item in &mut block.items {
        if let ImplItem::Fn(method) = item
            && (then.is_none() || declares::<Backend>(method))
        {
            let (constructor, program) = lower_program::<Backend>(method, &visibility, &impl_predicates, defaults)?;
            *method = constructor;
            generated.push(program);
        }
    }
    let extension = match then.as_deref() {
        Some([next, rest @ ..]) => {
            let written = input.visibility.as_ref();
            quote! {
                #[#next(#(#arguments,)* __alux_then(#(#rest),*))]
                #written #block
            }
        }
        Some([]) => {
            let mut unbounded = block.clone();
            unbind(&mut unbounded.generics);
            extension(&visibility, stated(&arguments, &input.item)?, &unbounded)?
        }
        None => extension(&visibility, stated(&arguments, &input.item)?, &block)?,
    };

    Ok(quote! {
        #extension
        #(#generated)*
    })
}

/// Reports whether a predicate bounds the interpreter, `This`.
fn bounds_this(predicate: &WherePredicate) -> bool {
    matches!(predicate, WherePredicate::Type(bound) if matches!(&bound.bounded_ty, Type::Path(path) if path.qself.is_none() && path.path.is_ident("This")))
}

/// Reads each leading `let name = value;` of a declaration as `value` written where `name` is used,
/// and removes the binding, so a declaration in A-normal form lowers as its nested form does.
///
/// Only a binding of a plain name with no type is read; any other statement is kept as written. The
/// value keeps the author's tokens, so diagnostics and editor navigation still point at it.
fn inline_bindings(block: &mut Block) {
    let mut bound: Vec<(Ident, Expr)> = Vec::new();
    let mut kept = Vec::new();
    for mut statement in std::mem::take(&mut block.stmts) {
        Substitute(&bound).visit_stmt_mut(&mut statement);
        match statement {
            Stmt::Local(Local {
                pat: Pat::Ident(PatIdent { ident, by_ref: None, mutability: None, subpat: None, .. }),
                init: Some(LocalInit { expr, diverge: None, .. }),
                attrs,
                ..
            }) if attrs.is_empty() => {
                bound.push((ident, *expr));
            }
            statement => kept.push(statement),
        }
    }
    block.stmts = kept;
}

/// Writes the program a declaration ends in in A-normal form: every argument that is itself a
/// compound expression is bound once with `let`, innermost first, and named where it was written.
///
/// The expansion then reads as one short statement per part, as the author's own form does.
fn normalize(block: &mut Block) {
    let Some(Stmt::Expr(program, None)) = block.stmts.pop() else { return };
    let mut program = program;
    let mut bindings = Vec::new();
    hoist_arguments(&mut program, &mut bindings);
    block.stmts.extend(bindings);
    block.stmts.push(Stmt::Expr(program, None));
}

/// Binds each compound argument along a chain of calls, leaving the chain itself in place.
fn hoist_arguments(expression: &mut Expr, bindings: &mut Vec<Stmt>) {
    let Expr::MethodCall(call) = expression else { return };
    hoist_arguments(&mut call.receiver, bindings);
    for argument in &mut call.args {
        if matches!(argument, Expr::MethodCall(_)) {
            hoist_arguments(argument, bindings);
            let name = format_ident!("part{}", bindings.len());
            let value = std::mem::replace(argument, parse_quote!(#name));
            bindings.push(parse_quote!(let #name = #value;));
        }
    }
}

/// Replaces each use of a bound name by its value, the latest binding of a name winning.
struct Substitute<'a>(&'a [(Ident, Expr)]);

impl VisitMut for Substitute<'_> {
    fn visit_expr_mut(&mut self, expression: &mut Expr) {
        if let Expr::Path(path) = expression
            && path.qself.is_none()
            && let Some(name) = path.path.get_ident()
            && let Some((_, value)) = self.0.iter().rev().find(|(bound, _)| bound == name)
        {
            *expression = value.clone();
            return;
        }
        syn::visit_mut::visit_expr_mut(self, expression);
    }
}

/// Replaces one declaration method by a program constructor and generates that program.
fn lower_program<Backend>(
    method: &ImplItemFn,
    visibility: &Visibility,
    impl_predicates: &Punctuated<WherePredicate, Token![,]>,
    defaults: &Backend::Defaults,
) -> syn::Result<(ImplItemFn, TokenStream)>
where
    Backend: ProgramBackendAlg,
{
    if method.sig.ident == Backend::ROOT {
        return Err(syn::Error::new_spanned(
            &method.sig.ident,
            format!(
                "`{root}` is the builder's own method that starts a program, so a declaration named \
                 `{root}` could never be called; give it a name of its own",
                root = Backend::ROOT,
            ),
        ));
    }
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
    if !Backend::INFER_SUBPROGRAM_BOUNDS {
        unbind(&mut constructor.sig.generics);
    }
    constructor.block = parse_quote!({ #program::default() });
    // A bound on `This` states what the program's interpreter must answer. The constructor runs on
    // whatever names the program, a builder included, so the bound belongs to the interpretation.
    if let Some(clause) = &mut constructor.sig.generics.where_clause {
        clause.predicates = clause.predicates.iter().filter(|predicate| !bounds_this(predicate)).cloned().collect();
    }

    // The same method read again as the program's compilation: nested programs first, then the
    // evidence its own declarations require, then the builder that replaces `self`.
    let mut compiler = method.clone();
    if Backend::READS_BINDINGS {
        inline_bindings(&mut compiler.block);
    }
    let mut subprograms = Subprograms::new();
    subprograms.visit_block_mut(&mut compiler.block);
    let declared = Backend::prepare_declarations(&mut compiler, defaults);
    ReplaceSelf.visit_block_mut(&mut compiler.block);
    // Every expansion is in A-normal form, whichever transport it declares.
    normalize(&mut compiler.block);

    // The bounds move to the interpretation; the parameters they introduce stay.
    let method_predicates = predicates(&compiler.sig.generics);
    unbind(&mut compiler.sig.generics);
    let obligations = impl_predicates
        .iter()
        .map(|predicate| quote!(#predicate))
        .chain(method_predicates.iter().map(|predicate| quote!(#predicate)))
        .chain(
            subprograms
                .programs()
                .iter()
                .filter(|_| Backend::INFER_SUBPROGRAM_BOUNDS && !Backend::STATES_SUBPROGRAM_BOUNDS)
                .map(Backend::require_subprogram),
        )
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

        #declared

        #compile
    };
    Ok((constructor, generated))
}
