//! CLI program backend for extension defunctionalization.
//!
//! The backend states what a CLI declaration means: each command handler is reified as a typed
//! operation, its declared argument sources become one `CliOperationAlg` bound on the interpreter,
//! and the command surface is compiled through `CliProgramAlg`. Everything shared with other
//! transports lives in [`crate::lower`] and [`crate::syntax`].

use crate::lower::{Chain, LoweredProgram, ProgramBackendAlg, expand_program};
use crate::syntax::{Reified, lift_operation, program_ident};
use proc_macro2::{Group, Span, TokenStream, TokenTree};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::visit_mut::{self, VisitMut};
use syn::{Expr, ExprLit, ExprMethodCall, GenericParam, ImplItemFn, Lit, Stmt, Type, parse_quote, parse_quote_spanned};

/// Interprets the shared lowering as a CLI command program.
struct CliBackend;

/// Reads one source declaration as the source type it denotes, by the call it starts from.
///
/// Spelling modifiers such as `.short('n')` keep a source's type. `.env(..)` and `.default(..)`
/// give it a fallback, an environment variable alone or a default, and `.optional()`, `.many()`,
/// and `.switch()` a shape; both are part of its type. A product's `.field(..)` and a sum's
/// `.or(..)` each add the source they declare, in order.
/// States the checks one source carries where it is written: the argument name `.arg_for(..)` gives
/// it, and the type its method states for the argument at its position.
fn source_checks(
    operation: &Type,
    named: Option<&(proc_macro2::Ident, proc_macro2::Ident)>,
    position: usize,
    name: Option<&Expr>,
    source: &Type,
    span: Span,
) -> Vec<TokenStream> {
    let mut checks = Vec::new();
    // A source declared for an argument by name is checked against the operation's own name for it.
    if let Some(name) = name {
        // A reified method's names are on an item no context changes, so the check is an item of its
        // own, run where the declaration is written; a handler written first-order is checked where
        // the program is compiled.
        let check = if let Some((method, arg_names)) = named {
            let method = method.to_string();
            quote_spanned! {name.span()=>
                const _: () = {
                    let message = ::alux_cli::cli_arg_mismatch(#method, #arg_names::ARG_NAMES, #position, #name);
                    if !message.is_empty() {
                        ::core::panic!("{}", message.as_str());
                    }
                };
            }
        } else {
            quote_spanned! {name.span()=>
                const {
                    if !::alux_cli::cli_arg_named(<#operation as ::alux_ext::OperationAlg>::ARG_NAMES, #position, #name) {
                        ::core::panic!("an argument is not the one `.arg_for(..)` names: sources bind to the operation's arguments in order");
                    }
                };
            }
        };
        checks.push(check);
    }
    // A reified method states each argument type it knows without a context, so the source is
    // checked against it where the declaration is written.
    if let Some((_, arg_names)) = named {
        let stated = format_ident!("ARG_{position}");
        let spanned = respan(quote!(#source), span);
        checks.push(quote_spanned! {span=>
            const _: () = ::alux_cli::cli_source_stated::<#spanned, _>(&#arg_names::#stated);
        });
    }
    checks
}

/// Names `aliased` as `name` beside the program, over those of `parameters` it mentions, and answers
/// the name as the evidence writes it.
fn alias(
    items: &mut Vec<TokenStream>,
    name: &proc_macro2::Ident,
    parameters: &[proc_macro2::Ident],
    aliased: &TokenStream,
) -> TokenStream {
    let mentioned = mentioned(aliased);
    let used = parameters.iter().filter(|parameter| mentioned.contains(parameter)).collect::<Vec<_>>();
    let generics = if used.is_empty() { quote!() } else { quote!(<#(#used),*>) };
    items.push(quote! {
        #[doc(hidden)]
        #[allow(type_alias_bounds)]
        type #name #generics = #aliased;
    });
    quote!(#name #generics)
}

/// Collects every identifier in `tokens`.
fn mentioned(tokens: &TokenStream) -> Vec<proc_macro2::Ident> {
    let mut idents = Vec::new();
    for token in tokens.clone() {
        match token {
            TokenTree::Ident(ident) => idents.push(ident),
            TokenTree::Group(group) => idents.extend(mentioned(&group.stream())),
            _ => {}
        }
    }
    idents
}

/// Places every token at `span`, so a check on a generated type is reported where it is authored.
fn respan(tokens: TokenStream, span: Span) -> TokenStream {
    tokens
        .into_iter()
        .map(|mut token| {
            if let TokenTree::Group(group) = &token {
                let mut inner = Group::new(group.delimiter(), respan(group.stream(), span));
                inner.set_span(span);
                token = TokenTree::Group(inner);
            } else {
                token.set_span(span);
            }
            token
        })
        .collect()
}

fn source_type(declaration: &Expr) -> Option<Type> {
    let (mut env, mut defaults, mut delimited, mut assigned) = (false, false, false, false);
    let (mut optional, mut many, mut switch, mut optional_value) = (false, false, false, false);
    let mut parts = Vec::new();
    let mut current = declaration;
    while let Expr::MethodCall(call) = current {
        if matches!(call.receiver.as_ref(), Expr::Path(path) if path.path.is_ident("self")) {
            let mode: Type = match (defaults, env) {
                (true, _) => parse_quote!(::alux_cli::WithFallback),
                (false, true) => parse_quote!(::alux_cli::WithEnv),
                (false, false) => parse_quote!(::alux_cli::NoFallback),
            };
            let presence: Type =
                if optional { parse_quote!(::alux_cli::Optional) } else { parse_quote!(::alux_cli::Whole) };
            let shape: Type = match (many, switch, optional_value) {
                (true, _, _) => parse_quote!(::alux_cli::Many),
                (false, true, _) => parse_quote!(::alux_cli::Switch),
                (false, false, true) => parse_quote!(::alux_cli::OptionalValue),
                (false, false, false) => presence.clone(),
            };
            // A source spelled as options names no spelling, so its type is the one written before
            // spellings existed.
            let spelling = assigned.then(|| quote!(, ::alux_cli::Assigned));
            parts.reverse();
            let product = parts.into_iter().fold(quote!(()), |product, part| quote!((#product, #part)));
            return match call.method.to_string().as_str() {
                "positional" => Some(parse_quote!(::alux_cli::PositionalSource<#mode, #shape>)),
                "named" if delimited => Some(parse_quote!(::alux_cli::DelimitedSource<#mode #spelling>)),
                "named" => Some(parse_quote!(::alux_cli::NamedSource<#mode, #shape #spelling>)),
                "product" => Some(parse_quote!(::alux_cli::ProductSource<#product, #presence>)),
                "sum" => Some(parse_quote!(::alux_cli::SumSource<#product, #presence>)),
                _ => None,
            };
        }
        env |= call.method == "env";
        defaults |= call.method == "default";
        delimited |= call.method == "delimiter";
        assigned |= call.method == "assigned";
        optional |= call.method == "optional";
        many |= call.method == "many";
        switch |= call.method == "switch";
        optional_value |= call.method == "optional_value";
        if call.method == "field" || call.method == "or" {
            parts.push(source_type(call.args.first()?)?);
        }
        current = &call.receiver;
    }

    None
}

/// Reads the sources a command declares, in authored order, as their product.
fn inputs_type(declaration: &Expr) -> Option<Type> {
    let Expr::MethodCall(call) = declaration else { return None };
    match call.method.to_string().as_str() {
        "op" => Some(parse_quote!(())),
        "arg" => {
            let previous = inputs_type(&call.receiver)?;
            let argument = source_type(call.args.first()?)?;
            Some(parse_quote!((#previous, #argument)))
        }
        "arg_for" => {
            let previous = inputs_type(&call.receiver)?;
            let argument = source_type(call.args.iter().nth(1)?)?;
            Some(parse_quote!((#previous, #argument)))
        }
        // Any other modifier, of how the output is read or how the words are, changes no argument.
        _ => inputs_type(&call.receiver),
    }
}

/// One source a command declares: where it is written, its type, and the argument name it is
/// declared for, if `.arg_for(..)` names one.
struct ArgumentSource {
    span: proc_macro2::Span,
    source: Type,
    name: Option<Expr>,
}

/// Reads each source a command declares, in authored order, with the span of the `.arg(..)` that
/// declares it, so the obligation it carries is reported where it is written.
fn argument_sources(declaration: &Expr) -> Vec<ArgumentSource> {
    let Expr::MethodCall(call) = declaration else { return Vec::new() };
    let named = call.method == "arg_for";
    match call.method.to_string().as_str() {
        "arg" | "arg_for" => {
            let mut sources = argument_sources(&call.receiver);
            let name = named.then(|| call.args.first()).flatten().cloned();
            if let Some(argument) = call.args.iter().nth(usize::from(named))
                && let Some(source) = source_type(argument)
            {
                sources.push(ArgumentSource { span: argument.span(), source, name });
            }
            sources
        }
        _ => argument_sources(&call.receiver),
    }
}

/// Reads how a command's output is read as its outcome: `Fallible` when it says `.fallible()`.
fn mode_type(declaration: &Expr) -> Type {
    let mut current = declaration;
    while let Expr::MethodCall(call) = current {
        if call.method == "fallible" {
            return parse_quote!(::alux_cli::Fallible);
        }
        current = &call.receiver;
    }

    parse_quote!(::alux_cli::Total)
}

/// Carries one command's operation, its input product, and its outcome mode.
type CommandRequirement = (Reified, Type, Type, Vec<ArgumentSource>);

/// Carries one context's operation, input product, outcome mode, and the program it applies to.
type ContextRequirement = (Reified, Type, Type, TokenStream);

/// Reads `self.program(self.method::<Types>())`, a nested program the lowering rewrote, as its type.
fn program_type(expression: &Expr) -> Option<TokenStream> {
    let Expr::MethodCall(call) = expression else { return None };
    let Some(Expr::MethodCall(named)) = call.args.first() else { return None };
    if call.method != "program" {
        return None;
    }
    let program = program_ident(&named.method);
    Some(if let Some(arguments) = &named.turbofish {
        let arguments = &arguments.args;
        quote!(#program<#arguments>)
    } else {
        quote!(#program)
    })
}

/// Rejects each command path a declaration states a second time, at its name: a command line could
/// not choose between the two. A program merged in from another declaration is not seen here.
fn duplicate_paths(declaration: &Expr) -> Vec<TokenStream> {
    let mut paths = Vec::new();
    declared_paths(declaration, &[], &mut paths);
    let mut errors = Vec::new();
    for (at, (path, span)) in paths.iter().enumerate() {
        if paths[..at].iter().any(|(earlier, _)| earlier == path) {
            let message = format!(
                "the command `{}` is declared more than once: a command line could not choose between them",
                path.join(" ")
            );
            errors.push(quote_spanned! {*span=> ::core::compile_error!(#message); });
        }
    }
    errors
}

/// Collects the command paths a chain declares beneath `prefix`, in authored order, each with the
/// span of the name that declares it.
fn declared_paths(chain: &Expr, prefix: &[String], paths: &mut Vec<(Vec<String>, proc_macro2::Span)>) {
    let mut links = Vec::new();
    let mut current = chain;
    while let Expr::MethodCall(call) = current {
        links.push(call);
        current = &call.receiver;
    }
    for call in links.into_iter().rev() {
        let name = match call.args.first() {
            Some(Expr::Lit(ExprLit { lit: Lit::Str(name), .. })) => Some(name),
            _ => None,
        };
        match (call.method.to_string().as_str(), name) {
            ("command", Some(name)) => {
                let path = prefix.iter().cloned().chain([name.value()]).collect();
                paths.push((path, name.span()));
            }
            ("cmd", _) => paths.push((prefix.to_vec(), call.method.span())),
            ("nest", Some(name)) => {
                if let Some(program) = call.args.iter().nth(1) {
                    let nested = prefix.iter().cloned().chain([name.value()]).collect::<Vec<_>>();
                    declared_paths(program, &nested, paths);
                }
            }
            _ => {}
        }
    }
}

/// Finds the commands, contexts, and nested programs of a CLI program, and whether it nests any.
struct Commands<'a> {
    requirements: &'a mut Vec<CommandRequirement>,
    contexts: &'a mut Vec<ContextRequirement>,
    programs: &'a mut Vec<TokenStream>,
    errors: &'a mut Vec<TokenStream>,
    nests: bool,
}

impl VisitMut for Commands<'_> {
    fn visit_expr_method_call_mut(&mut self, call: &mut ExprMethodCall) {
        self.nests |= call.method == "nest";
        if let Some(program) = program_type(&Expr::MethodCall(call.clone())) {
            self.programs.push(program);
            return;
        }
        if call.method == "within"
            && let [declaration, program] = call.args.iter_mut().collect::<Vec<_>>().as_mut_slice()
            && let Some(inputs) = inputs_type(declaration)
        {
            // The program within a context compiles with the context's interpreter, so it is not one
            // of this program's nested programs.
            let Some(inner) = program_type(program) else {
                // A subtree compiles with the context's interpreter, so its bound names its type, and
                // only a program declared by a method of its own has one.
                self.errors.push(quote_spanned! {program.span()=>
                    ::core::compile_error!("`within` applies a named program: declare the subtree in a method of its own and call it here");
                });
                return;
            };
            let mode = mode_type(declaration);
            if let Some(reified) = lift_operation(declaration) {
                self.contexts.push((reified, inputs, mode, inner));
            }
            self.visit_expr_mut(&mut call.receiver);
            return;
        }
        let declared = match call.method.to_string().as_str() {
            "command" => 1,
            "cmd" => 0,
            _ => usize::MAX,
        };
        if let Some(declaration) = call.args.iter_mut().nth(declared)
            && let Some(inputs) = inputs_type(declaration)
            && let Some(reified) = lift_operation(declaration)
        {
            let mode = mode_type(declaration);
            self.requirements.push((reified, inputs, mode, argument_sources(declaration)));
        }
        visit_mut::visit_expr_method_call_mut(self, call);
    }
}

impl ProgramBackendAlg for CliBackend {
    /// A CLI program states nothing once for every command it declares.
    type Defaults = ();

    const ROOT: &'static str = "commands";
    const REJECTED_PARAM: &'static str = "CLI programs currently support type parameters only";
    const STATES_SUBPROGRAM_BOUNDS: bool = true;
    const READS_BINDINGS: bool = true;

    fn prepare_declarations(method: &mut ImplItemFn, (): &Self::Defaults) -> TokenStream {
        let (mut commands, mut contexts, mut programs, mut errors) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let mut visitor = Commands {
            requirements: &mut commands,
            contexts: &mut contexts,
            programs: &mut programs,
            errors: &mut errors,
            nests: false,
        };
        visitor.visit_block_mut(&mut method.block);
        let nests = visitor.nests;
        if let Some(Stmt::Expr(declaration, None)) = method.block.stmts.last() {
            errors.extend(duplicate_paths(declaration));
        }
        if !errors.is_empty() {
            // A rejected declaration lowers to its errors alone, so they are all the author reads.
            method.block = parse_quote!({ #(#errors)* self.commands() });
            return TokenStream::new();
        }
        // Each long type the evidence names is named once, beside the program, so the evidence
        // reads one short bound per line.
        let program = program_ident(&method.sig.ident);
        let parameters = method
            .sig
            .generics
            .params
            .iter()
            .filter_map(|param| match param {
                GenericParam::Type(param) => Some(param.ident.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut items = Vec::new();
        let where_clause = method.sig.generics.make_where_clause();
        for program in programs {
            where_clause.predicates.push(parse_quote!(#program: ::alux_cli::CliProgramAlg<This, Commands = <This as ::alux_cli::CliCommandsAlg>::Commands>));
        }
        for (index, (Reified { operation, carrier, .. }, inputs, mode, inner)) in contexts.into_iter().rev().enumerate()
        {
            let inputs = alias(&mut items, &format_ident!("{program}Context{index}Sources"), &[], &quote!(#inputs));
            let context = quote!(<This as ::alux_cli::CliContextAlg<#operation, #inputs, #mode>>);
            where_clause.predicates.push(parse_quote! {
                #operation: ::alux_ext::OperationAlg<Context = #carrier>
            });
            // The context's arguments are the subtree's global arguments, so they are named.
            where_clause.predicates.push(parse_quote! {
                #inputs: ::alux_cli::CliSources<<#operation as ::alux_ext::OperationAlg>::Args> + ::alux_cli::CliNamedSources
            });
            where_clause.predicates.push(parse_quote! {
                This: ::alux_cli::CliContextAlg<#operation, #inputs, #mode, Commands = <This as ::alux_cli::CliCommandsAlg>::Commands>
            });
            where_clause.predicates.push(parse_quote! {
                #inner: ::alux_cli::CliProgramAlg<#context::Inner, Commands = #context::InnerCommands>
            });
        }
        if nests {
            where_clause.predicates.push(parse_quote! {
                This: ::alux_cli::CliNestAlg<Commands = <This as ::alux_cli::CliCommandsAlg>::Commands>
            });
        }
        let mut checks = Vec::new();
        for (index, (Reified { operation, carrier, method: named }, _, mode, sources)) in
            // Numbered as authored; the visitor meets the last command first.
            commands.into_iter().rev().enumerate()
        {
            where_clause.predicates.push(parse_quote! {
                #operation: ::alux_ext::OperationAlg<Context = #carrier>
            });
            let args = alias(
                &mut items,
                &format_ident!("{program}Command{index}Args"),
                &parameters,
                &quote!(<#operation as ::alux_ext::OperationAlg>::Args),
            );
            let mut inputs = quote!(());
            // Each source supplies the argument at its own position, so a source that cannot is
            // reported at the `.arg(..)` that declares it.
            for (position, ArgumentSource { span, source, name }) in sources.into_iter().enumerate() {
                let named_source =
                    alias(&mut items, &format_ident!("{program}Command{index}Source{position}"), &[], &quote!(#source));
                inputs = quote!((#inputs, #named_source));
                checks.extend(source_checks(&operation, named.as_ref(), position, name.as_ref(), &source, span));
                where_clause.predicates.push(parse_quote_spanned! {span=>
                    #named_source: ::alux_cli::CliSourceAt<#args, #position>
                });
            }
            let inputs = alias(&mut items, &format_ident!("{program}Command{index}Sources"), &[], &inputs);
            // One source per operation argument, in order, each able to supply its argument.
            where_clause.predicates.push(parse_quote! {
                #inputs: ::alux_cli::CliSources<#args>
            });
            where_clause.predicates.push(parse_quote! {
                This: ::alux_cli::CliOperationAlg<#operation, #inputs, #mode, Commands = <This as ::alux_cli::CliCommandsAlg>::Commands>
            });
        }
        // The checks lead the block, so its last expression is still the declaration chain.
        let checks: Vec<Stmt> = checks.into_iter().map(|check| parse_quote!(#check)).collect();
        method.block.stmts.splice(0..0, checks);
        quote!(#(#items)*)
    }

    fn require_subprogram(program: &TokenStream) -> TokenStream {
        quote! {
            #program: ::alux_cli::CliProgramAlg<This, Commands = <This as ::alux_cli::CliCommandsAlg>::Commands>
        }
    }

    fn read_link(call: &ExprMethodCall) -> Option<TokenStream> {
        // Each link is the authored call itself, on an empty program, so the method keeps the
        // author's token: an editor reading the expansion finds `cmd` or `command` where it is
        // defined.
        let method = &call.method;
        let arguments = &call.args;
        match (method.to_string().as_str(), arguments.len()) {
            ("command" | "nest" | "within", 2) | ("cmd" | "merge", 1) => {
                Some(quote!(builder.commands().#method(#arguments).into_program()))
            }
            _ => None,
        }
    }

    fn compile_program(lowered: &LoweredProgram) -> TokenStream {
        let LoweredProgram { program_type, compiler_params, predicates, chain } = lowered;
        let Chain { leading, root, leaves } = chain;
        quote! {
            impl<This, #compiler_params> ::alux_cli::CliProgramAlg<This> for #program_type
            where
                #(#predicates,)*
            {
                type Commands = <This as ::alux_cli::CliCommandsAlg>::Commands;

                fn compile_cli(self, compiler: &This) -> Self::Commands {
                    let _ = self;
                    let builder = ::alux_cli::CliProgramBuilder;
                    #(#leading)*
                    // Every command is merged into the one collection the interpreter states, so no
                    // type here grows with how many commands the declaration has.
                    let commands =
                        ::alux_cli::CompileCliProgram::compile_cli_program((#root).into_program(), compiler);
                    #(
                        let commands = ::alux_cli::CliCommandsAlg::cli_merge(
                            compiler,
                            commands,
                            ::alux_cli::CompileCliProgram::compile_cli_program(#leaves, compiler),
                        );
                    )*

                    commands
                }
            }
        }
    }
}

/// Expands the facade macro after converting compiler token streams into testable tokens.
pub(crate) fn cli_program_defunc_internal(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    expand_program::<CliBackend>(attr, item, &())
}

#[cfg(test)]
mod tests {
    use super::{cli_program_defunc_internal, inputs_type, source_type};
    use quote::ToTokens;
    use quote::quote;
    use syn::{Expr, parse_quote};

    #[test]
    fn reads_a_source_by_its_fallback_and_delimiter() {
        let source = |declaration: Expr| source_type(&declaration).unwrap().to_token_stream().to_string();
        assert_eq!(
            source(parse_quote!(self.named("Dev").switch().env("ALUX_DEV"))),
            quote! { ::alux_cli::NamedSource<::alux_cli::WithEnv, ::alux_cli::Switch> }.to_string(),
        );
        assert_eq!(
            source(parse_quote!(self.named("Port").env("PORT").default("8545"))),
            quote! { ::alux_cli::NamedSource<::alux_cli::WithFallback, ::alux_cli::Whole> }.to_string(),
        );
        assert_eq!(
            source(parse_quote!(self.named("Peers").many().env("PEERS").delimiter(','))),
            quote! { ::alux_cli::DelimitedSource<::alux_cli::WithEnv> }.to_string(),
        );
    }

    #[test]
    fn reads_a_source_assigned_by_an_operand() {
        let source = |declaration: Expr| source_type(&declaration).unwrap().to_token_stream().to_string();
        assert_eq!(
            source(parse_quote!(self.named("Input").assigned("if").optional())),
            quote! { ::alux_cli::NamedSource<::alux_cli::NoFallback, ::alux_cli::Optional, ::alux_cli::Assigned> }
                .to_string(),
        );
        assert_eq!(
            source(parse_quote!(self.named("Conversions").assigned("conv").many().delimiter(','))),
            quote! { ::alux_cli::DelimitedSource<::alux_cli::NoFallback, ::alux_cli::Assigned> }.to_string(),
        );
    }

    #[test]
    fn reads_a_product_or_sum_source_as_its_parts_and_presence() {
        let product: Expr =
            parse_quote!(self.product().field(self.named("User")).field(self.named("Secret").env("SECRET")));
        let sum: Expr = parse_quote!(self.sum().or(self.named("At").short('a')).or(self.named("Out")).optional());
        assert_eq!(
            source_type(&product).unwrap().to_token_stream().to_string(),
            quote! {
                ::alux_cli::ProductSource<
                    (
                        ((), ::alux_cli::NamedSource<::alux_cli::NoFallback, ::alux_cli::Whole>),
                        ::alux_cli::NamedSource<::alux_cli::WithEnv, ::alux_cli::Whole>
                    ),
                    ::alux_cli::Whole
                >
            }
            .to_string(),
        );
        assert_eq!(
            source_type(&sum).unwrap().to_token_stream().to_string(),
            quote! {
                ::alux_cli::SumSource<
                    (
                        ((), ::alux_cli::NamedSource<::alux_cli::NoFallback, ::alux_cli::Whole>),
                        ::alux_cli::NamedSource<::alux_cli::NoFallback, ::alux_cli::Whole>
                    ),
                    ::alux_cli::Optional
                >
            }
            .to_string(),
        );
    }

    #[test]
    fn keeps_the_type_parameters_a_command_selects() {
        let output = cli_program_defunc_internal(
            quote!(name = EchoCliExt),
            quote! {
                pub impl<This> This {
                    fn echo_cli<Machine>(&self) {
                        self.commands().command("echo", self.op(Machine::echo::<Vec<Vec<u8>>, Vec<u8>>))
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("EchoOperation < Machine , Vec < Vec < u8 > > , Vec < u8 > >"), "{output}");
    }

    #[test]
    fn rejects_a_command_path_declared_twice_at_its_second_name() {
        let output = cli_program_defunc_internal(
            quote!(name = TwiceCliExt),
            quote! {
                pub impl<This> This {
                    fn twice_cli<Machine>(&self) {
                        self.commands()
                            .command("head", self.op(Machine::head))
                            .nest("owner", self.commands().command("greet", self.op(Machine::greet)))
                            .nest("owner", self.commands().command("greet", self.op(Machine::greet)))
                            .command("head", self.op(Machine::split))
                            .command("tail", self.op(Machine::tail))
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("the command `head` is declared more than once"), "{output}");
        assert!(output.contains("the command `owner greet` is declared more than once"), "{output}");
        assert!(!output.contains("`tail` is declared"), "{output}");
    }

    #[test]
    fn rejects_a_declaration_named_after_the_root() {
        let error = cli_program_defunc_internal(
            quote!(name = SurfaceExt),
            quote! {
                pub impl<This> This {
                    fn commands<Machine>(&self) {
                        self.commands().merge(self.cat_cli::<Machine>())
                    }
                }
            },
        )
        .unwrap_err();

        assert!(error.to_string().contains("a declaration named `commands`"), "{error}");
    }

    #[test]
    fn reads_the_sources_of_a_command_declared_with_conventions() {
        let declaration: Expr =
            parse_quote!(self.op(Machine::echo).arg(self.named("No newline").short('n')).options_first().verbatim());
        assert_eq!(
            inputs_type(&declaration).unwrap().to_token_stream().to_string(),
            quote! { ((), ::alux_cli::NamedSource<::alux_cli::NoFallback, ::alux_cli::Whole>) }.to_string(),
        );
    }

    #[test]
    fn checks_each_source_against_the_argument_at_its_position() {
        let output = cli_program_defunc_internal(
            quote!(name = NodeCliExt),
            quote! {
                pub impl<This> This {
                    fn node_cli<Machine>(&self) {
                        self.commands().cmd(
                            self.op(Machine::start)
                                .arg(self.named("Development mode").switch())
                                .arg(self.named("Cache size").optional()),
                        )
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        // Each long type is named once beside the program, and the evidence names it.
        assert!(
            output.contains(
                "type NodeCliProgramCommand0Args < Machine > = < StartOperation < Machine > as :: alux_ext :: OperationAlg > :: Args ;"
            ),
            "{output}"
        );
        assert!(
            output.contains(
                "type NodeCliProgramCommand0Source1 = :: alux_cli :: NamedSource < :: alux_cli :: NoFallback , :: alux_cli :: Optional > ;"
            ),
            "{output}"
        );
        // One bound per source, at the `.arg(..)` that declares it.
        assert!(
            output.contains(
                "NodeCliProgramCommand0Source1 : :: alux_cli :: CliSourceAt < NodeCliProgramCommand0Args < Machine > , 1usize >"
            ),
            "{output}"
        );
    }

    #[test]
    fn checks_a_source_declared_for_an_argument_by_name() {
        let output = cli_program_defunc_internal(
            quote!(name = LoginCliExt),
            quote! {
                pub impl<This> This {
                    fn login_cli<Machine>(&self) {
                        self.commands().command(
                            "login",
                            self.op(Machine::login).arg_for("user", self.named("User")).arg_for("host", self.named("Host")),
                        )
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        // The names are checked on an item no context changes, so the check runs where it is written.
        assert!(
            output.contains(
                ":: alux_cli :: cli_arg_mismatch (\"login\" , LoginArgNames :: ARG_NAMES , 0usize , \"user\")"
            ),
            "{output}"
        );
        assert!(
            output.contains(
                ":: alux_cli :: cli_arg_mismatch (\"login\" , LoginArgNames :: ARG_NAMES , 1usize , \"host\")"
            ),
            "{output}"
        );
        assert!(output.contains("const _ : () ="), "{output}");
        // Each source is checked against the type its argument is stated with, where it is written.
        assert!(output.contains(":: alux_cli :: cli_source_stated :: <"), "{output}");
        assert!(output.contains("(& LoginArgNames :: ARG_1)"), "{output}");
    }

    #[test]
    fn generates_a_composable_cli_program() {
        let output = cli_program_defunc_internal(
            quote!(name = ReadingCliExt),
            quote! {
                pub impl<This> This
                where
                    This: CliCommandsAlg,
                {
                    fn reading_cli<Machine, Name>(&self) {
                        self.commands()
                            .command("lookup", self.op(Machine::lookup).arg(self.positional("Requested name")))
                            .command("count", self.op(Machine::count).arg(self.named("Every name").short('a')).fallible())
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("struct ReadingCliProgram"));
        assert!(output.contains("alux_cli :: CliProgramAlg"));
        assert!(output.contains("LookupOperation < Machine >"));
        assert!(
            output.contains("type ReadingCliProgramCommand0Sources = (() , ReadingCliProgramCommand0Source0) ;"),
            "{output}"
        );
        assert!(
            output.contains(
                "CliOperationAlg < LookupOperation < Machine > , ReadingCliProgramCommand0Sources , :: alux_cli :: Total"
            ),
            "{output}"
        );
        assert!(
            output.contains("CliOperationAlg < CountOperation < Machine > , ReadingCliProgramCommand1Sources , :: alux_cli :: Fallible"),
            "{output}"
        );
        assert!(
            output.contains("ReadingCliProgramCommand0Sources : :: alux_cli :: CliSources < ReadingCliProgramCommand0Args < Machine > >"),
            "{output}"
        );
        // One declaration per command, merged into the collection, and no nested program type.
        // Each command is compiled through the call the author wrote, so an editor finds its definition.
        assert_eq!(output.matches("builder . commands () . command (").count(), 2);
        assert_eq!(output.matches("cli_merge").count(), 2);
    }

    #[test]
    fn lowers_only_its_own_declarations_in_a_chain_and_hands_the_block_on() {
        let output = cli_program_defunc_internal(
            quote!(name = SurfacesExt, __alux_then(jsonrpc)),
            quote! {
                pub impl<This> This {
                    fn reading_cli<Machine>(&self) {
                        self.commands().command("sum", self.op(Machine::sum))
                    }

                    fn reading_rpc<Machine>(&self) {
                        self.methods().method("sum", self.op(Machine::sum))
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("struct ReadingCliProgram"));
        assert!(!output.contains("ReadingRpcProgram"), "the CLI backend lowered a JSON-RPC declaration");
        assert!(output.contains("# [jsonrpc (name = SurfacesExt , __alux_then ())] pub impl"), "{output}");
        assert!(output.contains("self . methods ()"), "the JSON-RPC declaration did not reach its backend");
        assert!(!output.contains("trait SurfacesExt"), "a backend before the last stated the extension");
    }

    #[test]
    fn states_the_extension_once_at_the_end_of_a_chain() {
        let output = cli_program_defunc_internal(
            quote!(name = SurfacesExt, __alux_then()),
            quote! {
                pub impl<This> This {
                    fn reading_cli<Machine>(&self) {
                        self.commands().command("sum", self.op(Machine::sum))
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert_eq!(output.matches("trait SurfacesExt").count(), 1);
        assert!(!output.contains("__alux_then"));
    }

    #[test]
    fn moves_a_declarations_interpreter_bound_off_its_constructor() {
        let output = cli_program_defunc_internal(
            quote!(name = SurfacesExt, __alux_then()),
            quote! {
                pub impl<This> This {
                    fn reading_cli<Machine>(&self)
                    where
                        This: CliCommandsAlg,
                        Machine: Reading,
                    {
                        self.commands().command("sum", self.op(Machine::sum))
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        // The constructor keeps the domain bound and drops the interpreter's.
        assert!(
            output.contains(
                "fn reading_cli < Machine > (& self) -> ReadingCliProgram < Machine > where Machine : Reading"
            ),
            "{output}"
        );
        // The interpretation states both.
        assert!(output.contains("This : CliCommandsAlg , Machine : Reading"), "{output}");
    }

    #[test]
    fn compiles_a_context_subtree_with_the_contexts_interpreter() {
        let output = cli_program_defunc_internal(
            quote!(name = LibraryCliExt),
            quote! {
                pub impl<This> This {
                    fn library_cli<Shelf, Store>(&self) {
                        self.commands()
                            .within(self.op(Shelf::open).arg(self.named("Store")).fallible(), self.notes_cli::<Store>())
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        let context = "< This as :: alux_cli :: CliContextAlg < OpenOperation < Shelf > , LibraryCliProgramContext0Sources , :: alux_cli :: Fallible > >";
        assert!(
            output.contains(&format!("NotesCliProgram < Store > : :: alux_cli :: CliProgramAlg < {context} :: Inner")),
            "{output}"
        );
        assert!(output.contains(":: alux_cli :: CliNamedSources"), "{output}");
        // The subtree is not compiled with the interpreter around it.
        assert!(!output.contains("NotesCliProgram < Store > : :: alux_cli :: CliProgramAlg < This"), "{output}");
        // The expansion is in A-normal form: each part is bound once, then named.
        assert!(output.contains("let part0 = builder . named (\"Store\") ;"), "{output}");
        assert!(
            output.contains("let part1 = builder . op (< OpenOperation < Shelf > > :: default ()) . arg (part0)"),
            "{output}"
        );
        assert!(output.contains("builder . commands () . within (part1 , part3)"), "{output}");
    }

    #[test]
    fn rejects_a_context_over_an_inline_program() {
        let output = cli_program_defunc_internal(
            quote!(name = LibraryCliExt),
            quote! {
                pub impl<This> This {
                    fn library_cli<Shelf, Store>(&self) {
                        self.commands().within(self.op(Shelf::open), self.commands().command("sum", self.op(Store::sum)))
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert!(output.contains("compile_error ! (\"`within` applies a named program"), "{output}");
    }

    #[test]
    fn nests_and_merges_named_cli_programs() {
        let output = cli_program_defunc_internal(
            quote!(name = AppCliExt),
            quote! {
                pub impl<This> This {
                    fn app_cli<Machine>(&self) {
                        self.commands().merge(self.reading_cli::<Machine>()).nest("admin", self.reading_cli::<Machine>())
                    }
                }
            },
        )
        .unwrap()
        .to_string();

        assert_eq!(output.matches("ReadingCliProgram < Machine > : :: alux_cli :: CliProgramAlg").count(), 2);
        assert!(output.contains("builder . commands () . nest (\"admin\""), "{output}");
        assert!(output.contains("This : :: alux_cli :: CliNestAlg"));
        assert!(output.contains("builder . program"));
    }
}
