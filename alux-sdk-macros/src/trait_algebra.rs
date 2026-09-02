//! `#[trait_algebra]` generates the dispatch artifacts of a trait-as-algebra.
//!
//! In addition to re-emitting the trait, it generates:
//!
//! - `<Trait>Op`, the operation as pure data: one variant per method and one field per argument.
//!   It contains no receiver, return value, or reply channel, so interpreters can inspect it.
//! - `<Trait>Reply`, one variant per returning method plus a hidden unit variant for void
//!   operations.
//! - `<Trait>Interpreter`, the interpreter contract: one mutable-receiver method per operation that
//!   returns the operation's value directly.
//! - `<Trait>Op::interpret`, the generated fold that routes one variant to the corresponding interpreter
//!   method and returns `<Trait>Reply`.
//! - `<Trait>Op::<method>`, typed operation constructors.
//! - `<Trait>Reply::into_<method>`, typed reply accessors.
//!
//! Associated types are lifted according to where they occur. Each `Self::Associated` used in an
//! argument becomes a generic parameter of `<Trait>Op`; each one used in a return becomes a generic
//! parameter of `<Trait>Reply`. `<Trait>Interpreter` redeclares the associated types, and `interpret` binds
//! the syntax carriers through the interpreter's associated types.
//!
//! Transport is a separate interpretation, asked for by `transport`, and the two spellings differ in
//! who names a type. `transport = <Carrier>` states the trait for one carrier the author names, and
//! belongs to the crate that owns that carrier. Bare `transport`, or `transport = capability`, names
//! none: the impl is headed by `AlgebraCall` and `AlgebraSend`, so it is stated for whatever
//! witnesses them, which is what the trait's own crate can state while knowing no transport at all.
//! It is headed by `Send + Sync` as well, since it reaches the carrier through a reference, and that
//! is what lets an algebra whose calls may be awaited elsewhere be stated this way.
//!
//! Stream transport is a separate interpretation. `alux-tokio` envelopes the generated operation
//! and reply types in a bounded channel while keeping the receiver available as a stream.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};
use syn::{FnArg, Ident, ItemTrait, Meta, Pat, ReturnType, Token, TraitItem, Type, TypePath};

/// Describes one operation while retaining the trait's original argument and return types.
struct Method {
    ident: Ident,
    variant: Ident,
    arguments: Vec<Ident>,
    argument_types: Vec<Type>,
    return_type: Option<Type>,
    is_async: bool,
}

/// Separates all associated carriers from those needed specifically by operations and replies.
struct Algebra {
    associated_types: Vec<Ident>,
    argument_associated_types: Vec<Ident>,
    reply_associated_types: Vec<Ident>,
    methods: Vec<Method>,
}

/// States who names the type a transport impl is stated for.
enum Transport {
    /// The author names one carrier, and the impl is stated for it.
    Carrier(syn::Path),
    /// The author names none, and the impl is stated for whatever witnesses the capabilities.
    Capability,
}

/// Holds the public names derived from the annotated trait name.
struct Names {
    operation: Ident,
    reply: Ident,
    interpreter: Ident,
    proxy: Ident,
}

/// Generates operation syntax, reply syntax, the interpreter contract, and their one-operation fold.
///
/// Every attribute argument is reapplied to the operation and reply enums.
pub(crate) fn trait_algebra_internal(attribute: TokenStream, definition: &ItemTrait) -> TokenStream {
    let attributes = match parse_attributes(attribute) {
        Ok(attributes) => attributes,
        Err(error) => return error,
    };
    let (proxied, attributes) = split_proxy(attributes);
    let (stated, attributes) = split_transport(attributes);
    let algebra = Algebra::from_definition(definition);
    let names = Names::for_trait(&definition.ident);
    let syntax = render_syntax(definition, &attributes, &algebra, &names);
    let interpreter = render_interpreter(definition, &algebra, &names);
    let operation = render_operation(definition, &algebra, &names);
    let reply = render_reply(definition, &algebra, &names);
    let proxy = if proxied { render_proxy(definition, &algebra, &names) } else { TokenStream::new() };
    let transport =
        stated.map_or_else(TokenStream::new, |stated| render_transport(definition, &algebra, &names, &stated));

    quote! {
        #definition
        #syntax
        #interpreter
        #operation
        #reply
        #proxy
        #transport
    }
}

impl Algebra {
    /// Analyzes associated carrier use without choosing their concrete representations.
    fn from_definition(definition: &ItemTrait) -> Self {
        let associated_types = definition
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Type(associated) => Some(associated.ident.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let methods = definition
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Fn(method) => Some(parse_method(method)),
                _ => None,
            })
            .collect::<Vec<_>>();
        let argument_associated_types = associated_types
            .iter()
            .filter(|associated| {
                methods.iter().any(|method| method.argument_types.iter().any(|item| type_uses_assoc(item, associated)))
            })
            .cloned()
            .collect();
        let reply_associated_types = associated_types
            .iter()
            .filter(|associated| {
                methods
                    .iter()
                    .any(|method| method.return_type.as_ref().is_some_and(|item| type_uses_assoc(item, associated)))
            })
            .cloned()
            .collect();

        Self { associated_types, argument_associated_types, reply_associated_types, methods }
    }
}

impl Names {
    /// Derives the generated public names from the annotated trait name.
    fn for_trait(trait_ident: &Ident) -> Self {
        Self {
            operation: format_ident!("{trait_ident}Op"),
            reply: format_ident!("{trait_ident}Reply"),
            interpreter: format_ident!("{trait_ident}Interpreter"),
            proxy: format_ident!("{trait_ident}Proxy"),
        }
    }
}

/// Removes the receiver from operation data and records whether folding must await the interpreter.
fn parse_method(method: &syn::TraitItemFn) -> Method {
    let ident = method.sig.ident.clone();
    let (arguments, argument_types) = method
        .sig
        .inputs
        .iter()
        .enumerate()
        .filter_map(|(index, input)| match input {
            FnArg::Receiver(_) => None,
            FnArg::Typed(argument) => {
                let name = match argument.pat.as_ref() {
                    Pat::Ident(ident) => ident.ident.clone(),
                    _ => format_ident!("argument_{index}"),
                };
                Some((name, (*argument.ty).clone()))
            }
        })
        .unzip();
    let return_type = match &method.sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, output) if is_unit(output) => None,
        ReturnType::Type(_, output) => Some(*output.clone()),
    };

    Method {
        variant: pascal(&ident),
        ident,
        arguments,
        argument_types,
        return_type,
        is_async: method.sig.asyncness.is_some(),
    }
}

fn render_syntax(definition: &ItemTrait, attributes: &[Meta], algebra: &Algebra, names: &Names) -> TokenStream {
    let visibility = &definition.vis;
    let operation = &names.operation;
    let reply = &names.reply;
    let operation_generics = generics(&algebra.argument_associated_types);
    let reply_generics = generics(&algebra.reply_associated_types);
    let operation_doc = format!("Represents one `{}` operation as pure data.", definition.ident);
    let reply_doc = format!("Represents the reply produced by a `{}` operation.", definition.ident);
    let operation_variants = algebra.methods.iter().map(|method| {
        let variant = &method.variant;
        let doc = format!("Represents the `{}` operation.", method.ident);
        let fields = method.arguments.iter().zip(method.argument_types.iter()).map(|(argument, item)| {
            let doc = format!("Carries the `{argument}` argument.");
            let item = rewrite(item, &algebra.associated_types);
            quote! {
                #[doc = #doc]
                #argument: #item
            }
        });
        quote! {
            #[doc = #doc]
            #variant { #(#fields),* }
        }
    });
    let reply_variants = algebra.methods.iter().filter_map(|method| {
        let output = method.return_type.as_ref()?;
        let variant = &method.variant;
        let doc = format!("Contains the reply to `{}`.", method.ident);
        let output = rewrite(output, &algebra.associated_types);
        Some(quote! {
            #[doc = #doc]
            #variant(#output)
        })
    });

    quote! {
        #(#[#attributes])*
        #[doc = #operation_doc]
        #visibility enum #operation #operation_generics {
            #(#operation_variants),*
        }

        #(#[#attributes])*
        #[doc = #reply_doc]
        #visibility enum #reply #reply_generics {
            #(#reply_variants,)*
            #[doc(hidden)]
            __Unit,
        }
    }
}

fn render_interpreter(definition: &ItemTrait, algebra: &Algebra, names: &Names) -> TokenStream {
    let visibility = &definition.vis;
    let interpreter = &names.interpreter;
    let interpreter_doc = format!("Interprets one `{}` operation at a time.", definition.ident);
    let associated_types = algebra.associated_types.iter().map(|ident| {
        let doc = format!("Represents the `{ident}` carrier used by the algebra.");
        quote! {
            #[doc = #doc]
            type #ident;
        }
    });
    let methods = algebra.methods.iter().map(|method| {
        let ident = &method.ident;
        let doc = format!("Interprets the `{ident}` operation.");
        let arguments = &method.arguments;
        let argument_types = &method.argument_types;
        let output = method.return_type.as_ref().map_or_else(TokenStream::new, |output| quote!(-> #output));
        let asyncness = method.is_async.then(|| quote!(async));
        quote! {
            #[doc = #doc]
            #asyncness fn #ident(&mut self, #(#arguments: #argument_types),*) #output;
        }
    });

    quote! {
        #[allow(async_fn_in_trait)]
        #[doc = #interpreter_doc]
        #visibility trait #interpreter {
            #(#associated_types)*
            #(#methods)*
        }
    }
}

fn render_operation(definition: &ItemTrait, algebra: &Algebra, names: &Names) -> TokenStream {
    let visibility = &definition.vis;
    let operation = &names.operation;
    let reply = &names.reply;
    let interpreter = &names.interpreter;
    let operation_generics = generics(&algebra.argument_associated_types);
    let constructors = algebra.methods.iter().map(|method| {
        let ident = &method.ident;
        let variant = &method.variant;
        let doc = format!("Constructs the `{ident}` operation.");
        let arguments = &method.arguments;
        let types = method.argument_types.iter().map(|item| rewrite(item, &algebra.associated_types));
        quote! {
            #[doc = #doc]
            #visibility fn #ident(#(#arguments: #types),*) -> Self {
                Self::#variant { #(#arguments),* }
            }
        }
    });
    let unit_reply = if algebra.reply_associated_types.is_empty() {
        quote!(#reply::__Unit)
    } else {
        let associated = &algebra.reply_associated_types;
        quote!(#reply::<#(Interpreter::#associated),*>::__Unit)
    };
    let call_arms = algebra.methods.iter().map(|method| {
        let variant = &method.variant;
        let ident = &method.ident;
        let arguments = &method.arguments;
        let await_result = method.is_async.then(|| quote!(.await));
        if method.return_type.is_some() {
            quote! {
                Self::#variant { #(#arguments),* } =>
                    #reply::#variant(interpreter.#ident(#(#arguments),*)#await_result),
            }
        } else {
            quote! {
                Self::#variant { #(#arguments),* } => {
                    interpreter.#ident(#(#arguments),*)#await_result;
                    #unit_reply
                },
            }
        }
    });
    let interpreter_bound = if algebra.argument_associated_types.is_empty() {
        quote!(#interpreter)
    } else {
        let associated = &algebra.argument_associated_types;
        quote!(#interpreter<#(#associated = #associated),*>)
    };
    let reply_type = if algebra.reply_associated_types.is_empty() {
        quote!(#reply)
    } else {
        let associated = &algebra.reply_associated_types;
        quote!(#reply<#(Interpreter::#associated),*>)
    };
    let call = if algebra.methods.iter().any(|method| method.is_async) {
        quote! {
            /// Interprets this operation with one interpreter and returns its reply.
            #visibility async fn interpret<Interpreter>(
                self,
                interpreter: &mut Interpreter,
            ) -> #reply_type
            where
                Interpreter: #interpreter_bound,
            {
                match self {
                    #(#call_arms)*
                }
            }
        }
    } else {
        quote! {
            /// Interprets this operation with one interpreter and returns its reply.
            #visibility fn interpret<Interpreter>(self, interpreter: &mut Interpreter) -> #reply_type
            where
                Interpreter: #interpreter_bound,
            {
                match self {
                    #(#call_arms)*
                }
            }
        }
    };

    quote! {
        impl #operation_generics #operation #operation_generics {
            #(#constructors)*
            #call
        }
    }
}

fn render_reply(definition: &ItemTrait, algebra: &Algebra, names: &Names) -> TokenStream {
    let visibility = &definition.vis;
    let reply = &names.reply;
    let reply_generics = generics(&algebra.reply_associated_types);
    let accessors = algebra.methods.iter().filter_map(|method| {
        let output = method.return_type.as_ref()?;
        let variant = &method.variant;
        let accessor = format_ident!("into_{}", method.ident);
        let doc = format!("Extracts the reply to `{}`.", method.ident);
        let panic = format!("`{accessor}` called on a different reply variant");
        let output = rewrite(output, &algebra.associated_types);
        Some(quote! {
            #[doc = #doc]
            #visibility fn #accessor(self) -> #output {
                match self {
                    Self::#variant(value) => value,
                    _ => panic!(#panic),
                }
            }
        })
    });

    quote! {
        impl #reply_generics #reply #reply_generics {
            #(#accessors)*
        }
    }
}

fn parse_attributes(attribute: TokenStream) -> Result<Vec<Meta>, TokenStream> {
    if attribute.is_empty() {
        return Ok(Vec::new());
    }
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    parser
        .parse2(attribute)
        .map(|attributes| attributes.into_iter().collect())
        .map_err(|error| error.to_compile_error())
}

/// Separates how a transport impl is stated from the attributes the enums carry.
///
/// Both spellings are transport interpretations, and they differ in who is allowed to name a type.
/// `transport = <Carrier>` names one carrier, resolved in the author's scope, and belongs to the
/// crate that owns it. Bare `transport`, or `transport = capability`, names none: the impl is stated
/// for whatever witnesses the capabilities, which is what a crate declaring the trait can state
/// while knowing no transport at all. Either way this crate names none of its own, and what it
/// emits speaks only `AlgebraCall` and `AlgebraSend`.
fn split_transport(attributes: Vec<Meta>) -> (Option<Transport>, Vec<Meta>) {
    let mut stated = None;
    let mut kept = Vec::new();
    for attribute in attributes {
        match &attribute {
            Meta::Path(path) if path.is_ident("transport") => stated = Some(Transport::Capability),
            Meta::NameValue(named) if named.path.is_ident("transport") => match &named.value {
                syn::Expr::Path(path) if path.path.is_ident("capability") => stated = Some(Transport::Capability),
                syn::Expr::Path(path) => stated = Some(Transport::Carrier(path.path.clone())),
                _ => kept.push(attribute),
            },
            _ => kept.push(attribute),
        }
    }

    (stated, kept)
}

/// States the trait for whatever reaches an interpreter elsewhere.
///
/// A method stating a value asks and unwraps: no answer means nobody will ever answer, which for an
/// interpreter meant to outlive its callers is a bug rather than a state to handle. A method stating
/// none sends and does not stay, which is what makes an algebra of such methods a feed. The bodies
/// are the same however the impl is headed, because what a carrier is asked for is the capability
/// and not the type.
fn render_transport(definition: &ItemTrait, algebra: &Algebra, names: &Names, transport: &Transport) -> TokenStream {
    let contract = &definition.ident;
    let Names { operation, reply, .. } = names;
    if !algebra.associated_types.is_empty() {
        return syn::Error::new_spanned(
            contract,
            "a transport states the trait itself, so it cannot be stated while the algebra's \
             carriers are still open",
        )
        .to_compile_error();
    }
    if let Some(synchronous) = algebra.methods.iter().find(|method| !method.is_async) {
        return syn::Error::new_spanned(
            &synchronous.ident,
            "a transport carries an operation to an interpreter elsewhere, so every method it \
             states is asynchronous",
        )
        .to_compile_error();
    }
    let methods = algebra.methods.iter().map(|method| {
        let ident = &method.ident;
        let arguments = &method.arguments;
        let argument_types = &method.argument_types;
        method.return_type.as_ref().map_or_else(
            || {
                quote! {
                    async fn #ident(&self, #(#arguments: #argument_types),*) {
                        let _unheard = ::alux_sdk::AlgebraSend::send(
                            self,
                            #operation::#ident(#(#arguments),*),
                        )
                        .await;
                    }
                }
            },
            |output| {
                let into = format_ident!("into_{}", ident);
                let gone = format!("`{contract}::{ident}`: the interpreter answered nothing, so it is gone");
                quote! {
                    async fn #ident(&self, #(#arguments: #argument_types),*) -> #output {
                        ::alux_sdk::AlgebraCall::ask(self, #operation::#ident(#(#arguments),*))
                            .await
                            .expect(#gone)
                            .#into()
                    }
                }
            },
        )
    });

    let header = match transport {
        Transport::Carrier(carrier) => quote!(impl #contract for #carrier<#operation, #reply>),
        Transport::Capability => {
            let asking = algebra
                .methods
                .iter()
                .any(|method| method.return_type.is_some())
                .then(|| quote!(::alux_sdk::AlgebraCall<#operation, #reply>));
            let sending = algebra
                .methods
                .iter()
                .any(|method| method.return_type.is_none())
                .then(|| quote!(::alux_sdk::AlgebraSend<#operation>));
            let stated = asking.into_iter().chain(sending);
            quote! {
                impl<Carrier> #contract for Carrier
                where
                    Carrier: #(#stated +)* ::core::marker::Send + ::core::marker::Sync,
            }
        }
    };

    quote! {
        #header {
            #(#methods)*
        }
    }
}

/// Separates the request for a proxy from the attributes the generated enums carry.
fn split_proxy(attributes: Vec<Meta>) -> (bool, Vec<Meta>) {
    let proxied = attributes.iter().any(|attribute| matches!(attribute, Meta::Path(path) if path.is_ident("proxy")));
    let carried = attributes
        .into_iter()
        .filter(|attribute| !matches!(attribute, Meta::Path(path) if path.is_ident("proxy")))
        .collect();
    (proxied, carried)
}

/// Renders the proxy: a value that *is* the trait, calling an interpreter elsewhere.
///
/// The proxy keeps the trait's own signature, so whoever holds one calls the trait and never
/// learns that an interpreter is somewhere else. Every method waits for its operation to be
/// interpreted, which is what awaiting the trait's own method means where the interpreter is at
/// hand. A method stating a value takes it out of the reply, and is left with no value to state
/// where the interpreter is gone; a method stating none has nothing to be left without, and
/// returns.
fn render_proxy(definition: &ItemTrait, algebra: &Algebra, names: &Names) -> TokenStream {
    let visibility = &definition.vis;
    let contract = &definition.ident;
    let Names { operation, reply, proxy, .. } = names;
    if !algebra.associated_types.is_empty() {
        return syn::Error::new_spanned(
            contract,
            "a proxy states the trait itself, so it cannot be generated while the algebra's \
             carriers are still open",
        )
        .to_compile_error();
    }
    if let Some(stated) = algebra.methods.iter().find(|method| !method.is_async) {
        return syn::Error::new_spanned(
            &stated.ident,
            "a proxy waits on an interpreter elsewhere, so every method it states is asynchronous",
        )
        .to_compile_error();
    }
    let methods = algebra.methods.iter().map(|method| {
        let ident = &method.ident;
        let arguments = &method.arguments;
        let argument_types = &method.argument_types;
        method.return_type.as_ref().map_or_else(
            || {
                quote! {
                    async fn #ident(&self, #(#arguments: #argument_types),*) {
                        self.calling.ask(#operation::#ident(#(#arguments),*)).await;
                    }
                }
            },
            |output| {
                let into = format_ident!("into_{}", ident);
                quote! {
                    async fn #ident(&self, #(#arguments: #argument_types),*) -> #output {
                        self.calling
                            .ask(#operation::#ident(#(#arguments),*))
                            .await
                            .expect("the interpreter this proxy states is gone")
                            .#into()
                    }
                }
            },
        )
    });
    let proxy_doc = format!("States `{contract}` by sending its operations to an interpreter elsewhere.");
    let new_doc = format!("States one `{contract}` by calling through the given caller.");
    quote! {
        #[doc = #proxy_doc]
        #visibility struct #proxy<Calling> {
            calling: Calling,
        }

        impl<Calling> #proxy<Calling> {
            #[doc = #new_doc]
            #visibility const fn new(calling: Calling) -> Self {
                Self { calling }
            }
        }

        impl<Calling> ::core::clone::Clone for #proxy<Calling>
        where
            Calling: ::core::clone::Clone,
        {
            fn clone(&self) -> Self {
                Self {
                    calling: self.calling.clone(),
                }
            }
        }

        impl<Calling> #contract for #proxy<Calling>
        where
            Calling: ::alux_sdk::AlgebraCall<#operation, #reply>,
        {
            #(#methods)*
        }
    }
}

fn generics(identifiers: &[Ident]) -> TokenStream {
    if identifiers.is_empty() { TokenStream::new() } else { quote!(<#(#identifiers),*>) }
}

fn type_uses_assoc(item: &Type, associated: &Ident) -> bool {
    struct Finder<'a> {
        associated: &'a Ident,
        found: bool,
    }

    impl<'syntax> Visit<'syntax> for Finder<'_> {
        fn visit_type_path(&mut self, path: &'syntax TypePath) {
            if is_self_assoc(path, self.associated) {
                self.found = true;
            }
            visit::visit_type_path(self, path);
        }
    }

    let mut finder = Finder { associated, found: false };
    finder.visit_type(item);
    finder.found
}

fn rewrite(item: &Type, associated_types: &[Ident]) -> Type {
    struct Rewriter<'a> {
        associated_types: &'a [Ident],
    }

    impl VisitMut for Rewriter<'_> {
        fn visit_type_path_mut(&mut self, path: &mut TypePath) {
            visit_mut::visit_type_path_mut(self, path);
            if path.qself.is_none()
                && path.path.segments.len() >= 2
                && path.path.segments[0].ident == "Self"
                && self.associated_types.iter().any(|associated| path.path.segments[1].ident == *associated)
            {
                path.path.segments = path.path.segments.iter().skip(1).cloned().collect();
            }
        }
    }

    let mut item = item.clone();
    Rewriter { associated_types }.visit_type_mut(&mut item);
    item
}

fn is_self_assoc(path: &TypePath, associated: &Ident) -> bool {
    path.qself.is_none()
        && path.path.segments.len() >= 2
        && path.path.segments[0].ident == "Self"
        && path.path.segments[1].ident == *associated
}

fn pascal(identifier: &Ident) -> Ident {
    let name =
        identifier.to_string().split('_').filter(|part| !part.is_empty()).fold(String::new(), |mut name, part| {
            let mut characters = part.chars();
            if let Some(first) = characters.next() {
                name.extend(first.to_uppercase());
                name.push_str(characters.as_str());
            }
            name
        });
    Ident::new(&name, identifier.span())
}

fn is_unit(item: &Type) -> bool {
    matches!(item, Type::Tuple(tuple) if tuple.elems.is_empty())
}
