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
//! An algebra whose carriers are still open is stated for every choice of them at once. Each
//! associated type becomes a parameter of the impl, bounded by what the trait declares of it, and
//! the impl defines it as that parameter. The carrier form reads the parameters off the carrier's
//! own type and is conditioned on the capabilities that carrier witnesses; the capability form and
//! the proxy read them off `AlgebraCarrier`, which names the one operation and reply type a carrier
//! carries.
//! Either way, every associated type must occur in some argument or return, since one that occurs
//! in neither is fixed by nothing a carrier carries.
//!
//! Stream transport is a separate interpretation. `alux-tokio` envelopes the generated operation
//! and reply types in a bounded channel while keeping the receiver available as a stream.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};
use syn::{FnArg, Ident, ItemTrait, Meta, Pat, ReturnType, Token, TraitItem, TraitItemType, Type, TypePath};

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
    contract: Ident,
    associated_types: Vec<Ident>,
    associated_declarations: Vec<TraitItemType>,
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
        let associated_declarations = definition
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Type(associated) => Some(associated.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let associated_types =
            associated_declarations.iter().map(|associated| associated.ident.clone()).collect::<Vec<_>>();
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
                methods.iter().any(|method| {
                    method.argument_types.iter().any(|item| type_uses_assoc(item, &definition.ident, associated))
                })
            })
            .cloned()
            .collect();
        let reply_associated_types = associated_types
            .iter()
            .filter(|associated| {
                methods.iter().any(|method| {
                    method.return_type.as_ref().is_some_and(|item| type_uses_assoc(item, &definition.ident, associated))
                })
            })
            .cloned()
            .collect();

        Self {
            contract: definition.ident.clone(),
            associated_types,
            associated_declarations,
            argument_associated_types,
            reply_associated_types,
            methods,
        }
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
            let item = rewrite(item, algebra, Naming::Bare);
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
        let output = rewrite(output, algebra, Naming::Bare);
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
        let argument_types =
            method.argument_types.iter().map(|item| rewrite(item, algebra, Naming::Projected)).collect::<Vec<_>>();
        let output = method.return_type.as_ref().map_or_else(TokenStream::new, |output| {
            let output = rewrite(output, algebra, Naming::Projected);
            quote!(-> #output)
        });
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
        let types = method.argument_types.iter().map(|item| rewrite(item, algebra, Naming::Bare));
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
        let output = rewrite(output, algebra, Naming::Bare);
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
    let Names { operation, .. } = names;
    let open = match OpenCarriers::for_impl(definition, algebra, names, "a transport") {
        Ok(open) => open,
        Err(error) => return error,
    };
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
        let argument_types =
            method.argument_types.iter().map(|item| rewrite(item, algebra, Naming::Parameter)).collect::<Vec<_>>();
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
                let output = rewrite(output, algebra, Naming::Parameter);
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

    let OpenCarriers { parameters, bounds, definitions, operation_type, reply_type, carried } = &open;
    let asking = algebra
        .methods
        .iter()
        .any(|method| method.return_type.is_some())
        .then(|| quote!(::alux_sdk::AlgebraCall<#operation_type, #reply_type>));
    let sending = algebra
        .methods
        .iter()
        .any(|method| method.return_type.is_none())
        .then(|| quote!(::alux_sdk::AlgebraSend<#operation_type>));
    let header = match transport {
        Transport::Carrier(carrier) if parameters.is_empty() => {
            quote!(impl #contract for #carrier<#operation_type, #reply_type>)
        }
        Transport::Carrier(carrier) => {
            // The carrier may witness the capabilities only for some carriers, so they are what the
            // impl is conditioned on, and the algebra's own bounds need not repeat the transport's.
            let stated = asking.iter().chain(sending.iter());
            quote! {
                impl<#(#parameters),*> #contract for #carrier<#operation_type, #reply_type>
                where
                    #carrier<#operation_type, #reply_type>: #(#stated)+*,
                    #(#bounds,)*
            }
        }
        Transport::Capability => {
            let stated = carried.iter().cloned().chain(asking).chain(sending);
            let carrier = subject("Carrier");
            quote! {
                impl<#carrier #(, #parameters)*> #contract for #carrier
                where
                    #carrier: #(#stated +)* ::core::marker::Send + ::core::marker::Sync,
                    #(#bounds,)*
            }
        }
    };

    quote! {
        #header {
            #(#definitions)*
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
    let Names { operation, proxy, .. } = names;
    let open = match OpenCarriers::for_impl(definition, algebra, names, "a proxy") {
        Ok(open) => open,
        Err(error) => return error,
    };
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
        let argument_types =
            method.argument_types.iter().map(|item| rewrite(item, algebra, Naming::Parameter)).collect::<Vec<_>>();
        method.return_type.as_ref().map_or_else(
            || {
                quote! {
                    async fn #ident(&self, #(#arguments: #argument_types),*) {
                        self.calling.ask(#operation::#ident(#(#arguments),*)).await;
                    }
                }
            },
            |output| {
                let output = rewrite(output, algebra, Naming::Parameter);
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
    let OpenCarriers { parameters, bounds, definitions, operation_type, reply_type, carried } = &open;
    let carried = carried.iter();
    let calling = subject("Calling");
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

        impl<#calling #(, #parameters)*> #contract for #proxy<#calling>
        where
            #calling: #(#carried +)* ::alux_sdk::AlgebraCall<#operation_type, #reply_type>,
            #(#bounds,)*
        {
            #(#definitions)*
            #(#methods)*
        }
    }
}

/// States what an impl of the trait itself leaves open: the algebra's carriers, as its parameters.
///
/// Each associated type becomes a parameter, its declared bounds become the parameter's, and the
/// impl defines the associated type as the parameter. The parameters, like the type an impl headed
/// by a parameter is stated for, are named apart from every name the trait states, so a bound such
/// as `type Error: Error` still names the trait it named where the trait was declared. With no associated type,
/// every part is empty and the operation and reply types are the bare enums, so an algebra whose
/// carriers are all chosen states exactly what it stated before carriers could be open.
struct OpenCarriers {
    /// The impl's parameters standing for the algebra's carriers.
    parameters: Vec<Ident>,
    /// The declared bounds of each carrier, as where-clause predicates on its parameter.
    bounds: Vec<TokenStream>,
    /// `type Carrier = <its parameter>;` for each carrier.
    definitions: Vec<TokenStream>,
    /// The operation type, applied to the carriers its arguments use.
    operation_type: TokenStream,
    /// The reply type, applied to the carriers its returns use.
    reply_type: TokenStream,
    /// `AlgebraCarrier` bound to the operation and reply types, which is what fixes the parameters of
    /// an impl headed by a type parameter; absent when there is no parameter to fix.
    carried: Option<TokenStream>,
}

impl OpenCarriers {
    /// Reads the open carriers off the algebra, refusing any no impl could fix.
    ///
    /// An impl's parameter is fixed only by what it is stated for, so a carrier occurring in no
    /// argument and no return is fixed by nothing a carrier carries. A generic associated type has
    /// parameters of its own, which no single carrier could state for all of them. A where clause of the
    /// associated type's own has no parameter to be stated on, so its bounds go where it is declared
    /// or in the trait's where clause, whose predicates on carriers the impl restates.
    fn for_impl(definition: &ItemTrait, algebra: &Algebra, names: &Names, stating: &str) -> Result<Self, TokenStream> {
        for declaration in &algebra.associated_declarations {
            let ident = &declaration.ident;
            let refused = if !declaration.generics.params.is_empty() {
                Some(format!(
                    "{stating} states the trait for one carrier of each kind, so `{ident}` cannot have \
                     parameters of its own"
                ))
            } else if declaration.generics.where_clause.is_some() {
                Some(format!(
                    "{stating} states each carrier as a parameter, so `{ident}` cannot have a where clause \
                     of its own: bound it where it is declared, or in the trait's where clause"
                ))
            } else if !algebra.argument_associated_types.contains(ident)
                && !algebra.reply_associated_types.contains(ident)
            {
                Some(format!(
                    "{stating} reads each carrier off what it carries, so `{ident}` must occur in some \
                     argument or return"
                ))
            } else {
                None
            };
            if let Some(refused) = refused {
                return Err(syn::Error::new_spanned(ident, refused).to_compile_error());
            }
        }
        let Names { operation, reply, .. } = names;
        let operation_generics = generics(&algebra.argument_associated_types.iter().map(parameter).collect::<Vec<_>>());
        let reply_generics = generics(&algebra.reply_associated_types.iter().map(parameter).collect::<Vec<_>>());
        let operation_type = quote!(#operation #operation_generics);
        let reply_type = quote!(#reply #reply_generics);
        let declared = algebra.associated_declarations.iter().filter(|declaration| !declaration.bounds.is_empty()).map(
            |declaration| {
                let ident = parameter(&declaration.ident);
                let bounds = declaration.bounds.iter().map(|bound| rewrite_bound(bound, algebra));
                quote!(#ident: #(#bounds)+*)
            },
        );
        // The trait's own predicates are obligations of every impl. Those naming a carrier are
        // restated on the parameters, since only this impl can name them. Those naming none (such as
        // `Self: Clone`) and supertraits are not restated yet, as before carriers could be open, so
        // an impl over a carrier that does not witness them fails to compile.
        let stated = definition
            .generics
            .where_clause
            .iter()
            .flat_map(|clause| clause.predicates.iter())
            .filter(|predicate| {
                algebra
                    .associated_types
                    .iter()
                    .any(|associated| predicate_uses_assoc(predicate, &algebra.contract, associated))
            })
            .map(|predicate| {
                let predicate = rewrite_predicate(predicate, algebra);
                quote!(#predicate)
            });
        let bounds = declared.chain(stated).collect();
        let definitions = algebra
            .associated_types
            .iter()
            .map(|ident| {
                let parameter = parameter(ident);
                quote!(type #ident = #parameter;)
            })
            .collect();
        let carried = (!algebra.associated_types.is_empty())
            .then(|| quote!(::alux_sdk::AlgebraCarrier<Operation = #operation_type, Reply = #reply_type>));

        Ok(Self {
            parameters: algebra.associated_types.iter().map(parameter).collect(),
            bounds,
            definitions,
            operation_type,
            reply_type,
            carried,
        })
    }
}

fn generics(identifiers: &[Ident]) -> TokenStream {
    if identifiers.is_empty() { TokenStream::new() } else { quote!(<#(#identifiers),*>) }
}

/// Names the impl's parameter standing for one carrier apart from every name the trait states.
fn parameter(associated: &Ident) -> Ident {
    format_ident!("__Open{}", associated)
}

/// Names the type an impl headed by a parameter is stated for apart from every name the trait states.
fn subject(name: &str) -> Ident {
    format_ident!("__{}", name)
}

/// Finds `Self::Associated`, or `<Self as Trait>::Associated`, anywhere in what it visits.
struct Finder<'a> {
    contract: &'a Ident,
    associated: &'a Ident,
    found: bool,
}

impl<'syntax> Visit<'syntax> for Finder<'_> {
    fn visit_type_path(&mut self, path: &'syntax TypePath) {
        if projected(path, self.contract).is_some_and(|(associated, _)| associated == self.associated) {
            self.found = true;
        }
        visit::visit_type_path(self, path);
    }
}

fn type_uses_assoc(item: &Type, contract: &Ident, associated: &Ident) -> bool {
    let mut finder = Finder { contract, associated, found: false };
    finder.visit_type(item);
    finder.found
}

fn predicate_uses_assoc(predicate: &syn::WherePredicate, contract: &Ident, associated: &Ident) -> bool {
    let mut finder = Finder { contract, associated, found: false };
    finder.visit_where_predicate(predicate);
    finder.found
}

/// Says what a generated item calls a carrier the trait projects off itself.
#[derive(Clone, Copy)]
enum Naming {
    /// The bare carrier, which the generated enums are generic over.
    Bare,
    /// The parameter an impl of the trait stands for it with.
    Parameter,
    /// `Self::Carrier`, which the interpreter declares.
    Projected,
}

/// Names each carrier the trait projects off itself as one generated item calls it.
struct Rewriter<'a> {
    algebra: &'a Algebra,
    naming: Naming,
}

fn rewrite(item: &Type, algebra: &Algebra, naming: Naming) -> Type {
    let mut item = item.clone();
    Rewriter { algebra, naming }.visit_type_mut(&mut item);
    item
}

fn rewrite_predicate(predicate: &syn::WherePredicate, algebra: &Algebra) -> syn::WherePredicate {
    let mut predicate = predicate.clone();
    Rewriter { algebra, naming: Naming::Parameter }.visit_where_predicate_mut(&mut predicate);
    predicate
}

fn rewrite_bound(bound: &syn::TypeParamBound, algebra: &Algebra) -> syn::TypeParamBound {
    let mut bound = bound.clone();
    Rewriter { algebra, naming: Naming::Parameter }.visit_type_param_bound_mut(&mut bound);
    bound
}

impl VisitMut for Rewriter<'_> {
    fn visit_type_path_mut(&mut self, path: &mut TypePath) {
        visit_mut::visit_type_path_mut(self, path);
        let Some((associated, at)) = projected(path, &self.algebra.contract) else {
            return;
        };
        if !self.algebra.associated_types.contains(associated) {
            return;
        }
        let mut carrier = path.path.segments[at].clone();
        let mut segments = Punctuated::new();
        match self.naming {
            Naming::Bare => {}
            Naming::Parameter => carrier.ident = parameter(&carrier.ident),
            Naming::Projected => segments.push(syn::PathSegment::from(Ident::new("Self", carrier.ident.span()))),
        }
        segments.push(carrier);
        segments.extend(path.path.segments.iter().skip(at + 1).cloned());
        path.qself = None;
        path.path.leading_colon = None;
        path.path.segments = segments;
    }
}

/// Reads the carrier a path projects off the trait itself, as `Self::Carrier` or
/// `<Self as Trait>::Carrier`, and where in the path it stands.
fn projected<'path>(path: &'path TypePath, contract: &Ident) -> Option<(&'path Ident, usize)> {
    let segments = &path.path.segments;
    let at = match &path.qself {
        None => {
            let first = segments.first()?;
            (path.path.leading_colon.is_none() && first.ident == "Self" && first.arguments.is_none()).then_some(1)?
        }
        Some(qself) => {
            let Type::Path(own) = qself.ty.as_ref() else {
                return None;
            };
            let named = qself.position.checked_sub(1).and_then(|last| segments.get(last))?;
            (own.qself.is_none() && own.path.is_ident("Self") && named.ident == *contract).then_some(qself.position)?
        }
    };
    segments.get(at).map(|carrier| (&carrier.ident, at))
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

#[cfg(test)]
mod tests {
    use super::trait_algebra_internal;
    use proc_macro2::TokenStream;
    use quote::quote;

    fn expand(attribute: TokenStream, definition: TokenStream) -> String {
        trait_algebra_internal(attribute, &syn::parse2(definition).unwrap()).to_string()
    }

    #[test]
    fn an_algebra_whose_carriers_are_chosen_states_the_capability_impl_unchanged() {
        let output = expand(
            quote!(transport),
            quote! {
                trait Counter {
                    async fn increment(&self, by: u32) -> u32;
                    async fn note(&self, text: String);
                }
            },
        );

        assert!(output.contains(
            "impl < __Carrier > Counter for __Carrier where __Carrier : :: alux_sdk :: AlgebraCall < CounterOp , \
             CounterReply > + :: alux_sdk :: AlgebraSend < CounterOp > + :: core :: marker :: Send + :: core :: \
             marker :: Sync ,"
        ));
        assert!(!output.contains("AlgebraCarrier"));
    }

    #[test]
    fn the_capability_impl_reads_open_carriers_off_what_the_carrier_carries() {
        let output = expand(
            quote!(transport),
            quote! {
                trait Registry {
                    type Peer: Clone + Send;
                    type Record;
                    async fn admit(&self, peer: Self::Peer) -> Option<Self::Record>;
                    async fn forget(&self, peer: Self::Peer);
                }
            },
        );

        assert!(output.contains(
            "impl < __Carrier , __OpenPeer , __OpenRecord > Registry for __Carrier where __Carrier : :: alux_sdk \
             :: AlgebraCarrier < Operation = RegistryOp < __OpenPeer > , Reply = RegistryReply < __OpenRecord > > + \
             :: alux_sdk :: AlgebraCall < RegistryOp < __OpenPeer > , RegistryReply < __OpenRecord > > + :: \
             alux_sdk :: AlgebraSend < RegistryOp < __OpenPeer > > + :: core :: marker :: Send + :: core :: marker \
             :: Sync , __OpenPeer : Clone + Send ,"
        ));
        assert!(output.contains("type Peer = __OpenPeer ; type Record = __OpenRecord ;"));
        assert!(output.contains("async fn admit (& self , peer : __OpenPeer) -> Option < __OpenRecord >"));
        assert!(output.contains("enum RegistryOp < Peer >"));
    }

    #[test]
    fn the_carrier_impl_reads_open_carriers_off_the_carrier_type() {
        let output = expand(
            quote!(transport = Kept),
            quote! {
                trait Ledger {
                    type Entry: From<Self::Note>;
                    type Note;
                    async fn enter(&self, entry: Self::Entry, note: Self::Note) -> Self::Entry;
                }
            },
        );

        assert!(output.contains(
            "impl < __OpenEntry , __OpenNote > Ledger for Kept < LedgerOp < __OpenEntry , __OpenNote > , \
             LedgerReply < __OpenEntry > > where Kept < LedgerOp < __OpenEntry , __OpenNote > , LedgerReply < \
             __OpenEntry > > : :: alux_sdk :: AlgebraCall < LedgerOp < __OpenEntry , __OpenNote > , LedgerReply < \
             __OpenEntry > > , __OpenEntry : From < __OpenNote > ,"
        ));
        assert!(!output.contains("AlgebraCarrier"));
    }

    #[test]
    fn the_proxy_reads_open_carriers_off_what_its_caller_carries() {
        let output = expand(
            quote!(proxy),
            quote! {
                trait Directory {
                    type Name;
                    async fn look_up(&self, name: Self::Name) -> usize;
                }
            },
        );

        assert!(output.contains(
            "impl < __Calling , __OpenName > Directory for DirectoryProxy < __Calling > where __Calling : :: \
             alux_sdk :: AlgebraCarrier < Operation = DirectoryOp < __OpenName > , Reply = DirectoryReply > + :: \
             alux_sdk :: AlgebraCall < DirectoryOp < __OpenName > , DirectoryReply > ,"
        ));
        assert!(output.contains("type Name = __OpenName ;"));
    }

    #[test]
    fn a_carrier_nothing_carries_is_refused() {
        let output = expand(
            quote!(transport),
            quote! {
                trait Registry {
                    type Peer;
                    type Unused;
                    async fn admit(&self, peer: Self::Peer);
                }
            },
        );

        assert!(output.contains("compile_error"));
        assert!(output.contains("so `Unused` must occur in some argument or return"));
    }

    #[test]
    fn a_carrier_with_parameters_of_its_own_is_refused() {
        let output = expand(
            quote!(proxy),
            quote! {
                trait Registry {
                    type Peer<Tag>;
                    async fn admit(&self, peer: Self::Peer<u8>);
                }
            },
        );

        assert!(output.contains("compile_error"));
        assert!(output.contains("so `Peer` cannot have parameters of its own"));
    }

    #[test]
    fn an_algebra_whose_carriers_are_chosen_states_the_carrier_impl_unchanged() {
        let output = expand(
            quote!(transport = Kept),
            quote! {
                trait Counter {
                    async fn increment(&self, by: u32) -> u32;
                }
            },
        );

        assert!(output.contains("impl Counter for Kept < CounterOp , CounterReply > {"));
    }

    #[test]
    fn the_carrier_impl_is_conditioned_on_the_capabilities_the_carrier_witnesses() {
        let output = expand(
            quote!(transport = Kept),
            quote! {
                trait Registry {
                    type Peer;
                    async fn admit(&self, peer: Self::Peer) -> usize;
                    async fn forget(&self, peer: Self::Peer);
                }
            },
        );

        assert!(output.contains(
            "impl < __OpenPeer > Registry for Kept < RegistryOp < __OpenPeer > , RegistryReply > where Kept < \
             RegistryOp < __OpenPeer > , RegistryReply > : :: alux_sdk :: AlgebraCall < RegistryOp < __OpenPeer > \
             , RegistryReply > + :: alux_sdk :: AlgebraSend < RegistryOp < __OpenPeer > > ,"
        ));
    }

    #[test]
    fn the_traits_own_predicates_on_carriers_are_restated() {
        let output = expand(
            quote!(transport),
            quote! {
                trait Registry
                where
                    Self::Peer: Clone,
                    Self: Sized,
                {
                    type Peer;
                    async fn admit(&self, peer: Self::Peer) -> usize;
                }
            },
        );

        assert!(output.contains(":: core :: marker :: Sync , __OpenPeer : Clone , {"), "{output}");
    }

    #[test]
    fn the_traits_own_predicates_on_qualified_carriers_are_restated() {
        let output = expand(
            quote!(transport),
            quote! {
                trait Registry
                where
                    <Self as Registry>::Peer: Clone,
                {
                    type Peer;
                    async fn admit(&self, peer: Self::Peer) -> usize;
                }
            },
        );

        assert!(output.contains(":: core :: marker :: Sync , __OpenPeer : Clone , {"), "{output}");
    }

    #[test]
    fn a_qualified_carrier_is_lifted_like_an_unqualified_one() {
        let output = expand(
            quote!(proxy),
            quote! {
                trait Directory {
                    type Name;
                    async fn look_up(&self, name: <Self as Directory>::Name) -> usize;
                }
            },
        );

        assert!(output.contains("enum DirectoryOp < Name > {"), "{output}");
        assert!(output.contains("async fn look_up (& mut self , name : Self :: Name) -> usize ;"), "{output}");
        assert!(output.contains("async fn look_up (& self , name : __OpenName) -> usize"), "{output}");
    }

    #[test]
    fn a_bound_naming_a_trait_called_like_its_carrier_still_names_that_trait() {
        let output = expand(
            quote!(transport),
            quote! {
                trait Fallible {
                    type Error: Error;
                    async fn fail(&self) -> Self::Error;
                }
            },
        );

        assert!(output.contains("__OpenError : Error ,"), "{output}");
        assert!(output.contains("type Error = __OpenError ;"), "{output}");
    }

    #[test]
    fn a_carrier_with_a_where_clause_of_its_own_is_refused() {
        let output = expand(
            quote!(transport),
            quote! {
                trait Registry {
                    type Peer where Self: Sized;
                    async fn admit(&self, peer: Self::Peer);
                }
            },
        );

        assert!(output.contains("compile_error"));
        assert!(output.contains("so `Peer` cannot have a where clause of its own"));
    }

    #[test]
    fn a_carrier_named_like_the_named_carrier_is_free() {
        let output = expand(
            quote!(transport = Kept),
            quote! {
                trait Ledger {
                    type Kept;
                    async fn enter(&self, entry: Self::Kept);
                }
            },
        );

        assert!(!output.contains("compile_error"));
        assert!(output.contains("impl < __OpenKept > Ledger for Kept < LedgerOp < __OpenKept > , LedgerReply >"));
    }

    #[test]
    fn a_carrier_named_carrier_is_free_where_a_carrier_is_named() {
        let output = expand(
            quote!(transport = Kept),
            quote! {
                trait Ledger {
                    type Carrier;
                    async fn enter(&self, entry: Self::Carrier);
                }
            },
        );

        assert!(!output.contains("compile_error"));
        assert!(output.contains("impl < __OpenCarrier > Ledger for Kept < LedgerOp < __OpenCarrier > , LedgerReply >"));
    }

    #[test]
    fn a_carrier_named_like_what_the_impl_is_stated_for_is_free() {
        let output = expand(
            quote!(transport),
            quote! {
                trait Registry {
                    type Carrier;
                    async fn admit(&self, peer: Self::Carrier);
                }
            },
        );

        assert!(!output.contains("compile_error"));
        assert!(output.contains("impl < __Carrier , __OpenCarrier > Registry for __Carrier"));
    }
}
