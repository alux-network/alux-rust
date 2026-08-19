# Denotational Design

## What this means

ALUX Rust's **Design by Meaning** is a Rust adaptation of **Denotational Design**, a methodology
developed and named by [Conal Elliott](http://conal.net/). Its central move is to give types and
operations precise, simple, implementation-independent meanings and use those meanings to design the
programming interface.

An implementation is not the definition of an operation. It is one representation that must
preserve the operation's meaning and relevant structure.

```text
representation first                          meaning first
--------------------                          -------------
choose callbacks, framework, and state        choose the observation or transformation
derive an API from that machinery             derive operations compositionally
generate framework code as the authority      preserve a neutral first-order program
describe behavior afterward                   choose and validate interpreters afterward
```

This is more than dependency inversion, trait-oriented programming, or procedural macros. Traits
can abstract machinery while leaving meaning vague. Macros can remove syntax while duplicating or
hiding the real program. Design by Meaning requires the semantic object and its laws to be identified
before representation or generation choices dominate the API.

## How ALUX Rust adapts it

ALUX Rust uses Rust-native encodings without claiming that the encodings define Denotational Design:

| Denotational role | ALUX Rust encoding |
| --- | --- |
| Primitive domain observations/effects | Small capability traits in downstream specification crates |
| Derived domain transformations | `#[ext]` methods over explicit capability bounds |
| First-order operation meaning | `OperationAlg` and `ApplyAlg` |
| Interface composition | Typed HTTP and JSON-RPC program values |
| Alternative interpretations | Text, Poem, jsonrpsee, metadata, tests, and downstream adapters |
| Structure-preservation obligations | Generic laws and shared cross-interpreter scenarios |

The Rust technique is justified only when it preserves useful meaning:

- A capability trait is useful only when it exposes a coherent primitive observation or effect.
- An extension is useful only when its bounds and result state a real composition.
- A generated operation is useful only when method application must become first-order data.
- A transport program is useful only when multiple interpreters can share it.
- A test is finite executable evidence, not universal proof.

When stronger assurance matters, add property tests, exhaustive finite checks, model comparison, or
proof artifacts rather than describing example tests as proof.

## Scope

These are strict design and review rules for ALUX Rust foundation crates and for downstream
specification crates using them. The foundation preserves operation and interface-program meaning;
it does not own downstream domain semantics.

## Choose the semantic carrier

A semantic carrier is the world of meanings admitted at an abstraction boundary, not necessarily a
state struct. For a downstream operation, the carrier may be an interpreter implementing several
capability traits. For ALUX Rust, the relevant first-order carriers are:

```text
operation carrier = context algebra × ordered arguments -> output

HTTP program carrier = selectors × input roles × operation × output role

JSON-RPC program carrier = method name × parameter mode × operation
```

Each carrier intentionally forgets machine details. An operation does not contain a Tokio task or
Poem request. An HTTP program does not contain a concrete router. A JSON-RPC program does not contain
a jsonrpsee module.

Externality is relative to the boundary. Runtime ownership is external to a domain operation, then
internalized by an interpreter-selected `HandlerContextAlg::Handle`. Request extraction is external
to an operation, then internalized by an HTTP interpreter. That movement is valid only because the
new capability and its obligations are explicit.

Do not hide distinct meanings in a universal context, input enum, response enum, or callback bag.
Extension bounds and first-order type parameters are the open semantic dependency row.

## Specify domain meaning downstream

ALUX expects many independently published specifications. Each specification owns its primitive
domain vocabulary:

```rust
trait PriceAlg {
    type Asset;
    type Price;

    fn price(&self, asset: &Self::Asset) -> Self::Price;
}
```

`alux-ext`, `alux-http`, and `alux-jsonrpc` must not accumulate unrelated domain traits merely so
examples can share them. Doing so would create a central god specification and reverse the intended
dependency direction.

Stable semantic value types may be concrete in a specification. Interpreter-selected carrier,
framework, storage, and application-state representations should remain abstract or outside it.

## Derive behavior in extensions

Derived behavior belongs in an extension whose bounds state its complete contract:

```rust
use alux_ext::ext;

#[ext(name = QuoteExt)]
impl<This> This
where
    This: PriceAlg + FeeAlg,
{
    fn quote(&self, asset: &This::Asset) -> This::Quote {
        // Derived only through the named capabilities.
    }
}
```

Do not put `quote` on an application state merely because that state stores prices and fees. The
concrete carrier interprets primitives; the extension owns their composition.

The `where` clause is meaningful documentation. Keep it minimal and explicit. Do not replace it with
one inherited `ApplicationContext` trait.

## Reify application, not implementation

Rust method items involving extension traits and abstract associated types are difficult to preserve
as uniform first-class values. Defunctionalization solves that representation problem:

```text
method application
    -> zero-sized operation type
    -> context + argument product + argument names + output
    -> ApplyAlg interpretation
```

The operation type denotes applying the authored method. It must call that method rather than contain
a copied body. Generated first-order application and ordinary extension invocation should agree.

Reification is not automatically good design. Do not use `defunc` when a normal call composes
adequately. Use it when the operation must be stored in a route/method program, inspected by another
interpreter, or composed before execution.

## Preserve interface programs

A framework annotation is an implementation choice. The interface program should exist before the
framework:

```rust
#[ext(name = PriceApiExt, defunc(via = http))]
impl<This> This
where
    This: HttpApiAlg + JsonOutAlg,
{
    fn price_api<Alg>(&self)
    where
        Alg: PriceAlg,
    {
        self.routes().get("/price/:asset", self.op(Alg::price_current).path::<String>().json());
    }
}
```

This program preserves the route selector, extraction role, operation, and output role. Poem is one
interpretation. Text documentation, OpenAPI metadata, a client generator, or another server can be
other interpretations.

Do not separately maintain a Poem route list and an OpenAPI route list. Interpret the same program
twice. Duplication is not merely inconvenient; it creates competing meanings.

JSON-RPC follows the same rule. Method names and parameter modes belong to the neutral program.
jsonrpsee registration belongs to its interpreter.

## Compose, do not inherit

Semantic conjunction is expressed by bounds. Static policy composition is expressed by values.
Runtime environments are introduced only when they genuinely carry independent interpreters.

| Meaning location | Composition |
| --- | --- |
| One receiver answers all capabilities | Direct extension bounds |
| One separately selected policy | Explicit parameter |
| Several independently selected policies | Product value |
| Environment carries separate interpreters | `HasX` projections |
| Wrapper truthfully substitutes for inner value | Delegation |

Projection and delegation are not interchangeable. Projection says “has”; delegation says “is.”
Neither should be used only to shorten a method call.

Transport interpreter aliases may collect repeated mechanical traits such as route construction and
input selection. They are conveniences at an interpreter boundary, not new domain meaning and not a
license to hide operation-specific dependencies.

## Keep interpreters neutral

Concrete interpreters preserve program meaning while choosing machinery:

- A text interpreter records selectors and type roles.
- Poem chooses extractor types, endpoint erasure, response conversion, and shared context ownership.
- jsonrpsee chooses parameter parsing, method registration, serialization, and RPC errors.

Interpreters must not add routes, rename methods, reorder arguments, select domain policy, or invoke a
different operation. Mechanical defaults may be derived; semantic defaults must be explicit.

Separate interpreter crates enforce neutrality. A specification crate cannot name Poem or jsonrpsee,
so if neutral program code seems to need one, the interpreter boundary has leaked.

## Keep macros below meaning

Procedural macros are syntax translators. Their target must be a public, manually constructible
first-order representation.

Good direction:

```text
convenient extension syntax -> public operation/program values -> generic fold
```

Reject this direction:

```text
framework-shaped syntax -> opaque generated callbacks -> reverse-engineered documentation
```

Macro expansion tests verify lowering. Downstream integration tests verify that generated public
code resolves and composes. Neither replaces laws over the first-order algebra.

Diagnostics are part of the authoring surface. Report the violated rule at the authored expression;
do not expose internal visitor or generated-type accidents when a semantic explanation is available.

## Put bounds at use sites

Do not constrain generic traits, syntax nodes, or carriers preemptively. A bound belongs on the fold,
extension, or interpreter operation that uses it.

For example, an operation signature does not require JSON serialization. The jsonrpsee interpreter
adds `Serialize` to the output it registers. An HTTP input marker does not require deserialization;
the Poem interpreter adds `DeserializeOwned` when extracting it.

This discipline keeps the first-order meaning reusable and makes interpreter contracts readable.

## State laws once

Examples show one execution. Laws state obligations for every compatible interpretation.

Prefer laws about preserved structure:

- direct invocation equals first-order application
- identity selectors and initial routes behave as identities
- selector and route composition are associative where claimed
- nesting composes prefixes in order
- merging named programs preserves each child surface
- positional and named decoding produce the same typed argument product for equivalent input
- text/metadata and executable interpreters observe the same ordered route or method surface

Keep shared scenarios independent from framework-specific setup. The JSON-RPC comparison is a local
model: one expectation function observes both the specification-first interpreter and jsonrpsee's
native macro service.

Finite tests are evidence. When a law must hold universally, encode generic checks, property tests,
or stronger verification rather than relying on one fixture.

## Keep the product surface intentional

Root exports are vocabulary users learn. Export semantic algebras, first-order syntax, generic folds,
intentional interpreters, and public macros. Keep parser visitors, tuple machinery, framework helper
types, and test fixtures private unless their names carry stable public meaning.

Generated names are also product surface when downstream code refers to them. Changing an operation
or program naming rule can be a breaking change even if ordinary extension calls still compile.

A specification crate's dependency list is a semantic promise about its boundary. Keep frameworks in
separate interpreter crates rather than behind a feature, because an optional dependency still makes
framework meaning expressible in the specification.

Rejected syntax is product surface too. A macro diagnostic is the only documentation a rejected form
has, so its wording can be pinned by a downstream compile-fail test, and later accepting the form
silently changes what that source means.

## Review checklist

- Is the meaning stated before its representation or macro syntax?
- Does the domain capability belong downstream rather than in this foundation workspace?
- Does each trait or first-order node expose one coherent primitive distinction?
- Is derived behavior an extension over minimal explicit bounds?
- Does the generated operation invoke the authored method rather than duplicate it?
- Can the program be constructed and folded without the convenience macro?
- Are framework types confined to optional interpreters?
- Are generic bounds located at the operations that require them?
- Does composition preserve route/method order, nesting, arguments, and outputs?
- Can an independent interpreter implement the public algebra without using the existing framework?
- Is a reusable law or shared scenario added for a general obligation?
- Do exports, crate boundaries, diagnostics, and documentation describe meaning rather than file topology?

If several answers are no, revise the semantic surface before adding machinery.

## Lineage and further reading

- [ALUX programming guidelines](https://alux-network.github.io/alux-programming/) teach this method in
  full, from denotations and laws to capability algebras and first-order programs in Rust.
- [Conal Elliott's website](http://conal.net/)
- [Denotational Design: from meanings to programs](https://github.com/conal/talk-2014-bayhac-denotational-design)
- [LambdaJam Denotational Design workshop](https://github.com/conal/talk-2014-lambdajam-denotational-design)
- [Denotational design with type class morphisms](http://conal.net/papers/type-class-morphisms/)

The methodology and document structure are adapted primarily from CCP's executable-specification
work. ALUX's larger runtime remains a secondary source for practical extension, delegation, and
interpreter patterns.
