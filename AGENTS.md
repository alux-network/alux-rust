# ALUX Rust repository guide

This file is the compact engineering authority for humans and coding agents working on ALUX Rust.
Apply it to the whole repository.

## Product

ALUX Rust provides reusable Design by Meaning infrastructure for independently published Rust
specifications. It reifies derived operations as first-order values and describes typed HTTP and
JSON-RPC programs independently of their concrete frameworks.

This repository is specification infrastructure. Do not turn it into a central domain-spec crate,
application framework, service runtime, dependency-injection container, or framework-owned API
generator without an explicit scope decision. Downstream crates own their domain capabilities.

## Workspace

| Crate | Responsibility |
| --- | --- |
| `alux-ext` | `ext` facade, first-order operation signature/application, and owned context handles |
| `alux-ext-macros` | Parsing, validation, and lowering for extension, HTTP, and JSON-RPC programs |
| `alux-http` | Neutral typed HTTP syntax, algebras, and folds |
| `alux-jsonrpc` | Neutral typed JSON-RPC syntax, algebras, and folds |
| `alux-http-text` | Text and metadata interpretation of an HTTP program |
| `alux-http-poem` | Poem interpretation of an HTTP program |
| `alux-jsonrpc-jsonrpsee` | jsonrpsee interpretation of a JSON-RPC program |

Dependency arrows point toward `alux-ext`. The specification crates declare exactly one dependency,
`alux-ext`, and re-export the macro backends they own from `alux_ext::macros`.

```text
alux-http-text ------\
                      -> alux-http ------\
alux-http-poem ------/                    -> alux-ext -> alux-ext-macros
alux-jsonrpc-jsonrpsee -> alux-jsonrpc --/
```

A specification crate must never name a framework, serialization format, or runtime. Each
interpretation is a separate package named `alux-<program>-<interpreter>`, and adding one must not
require changing the program it folds.

## Authority

Read authority in this order:

1. The semantic role documented for an algebra or first-order program defines the intended meaning.
2. Public trait signatures and syntax types define primitive executable meaning.
3. Extension bounds, generated operation signatures, and generic folds define derived meaning.
4. Public laws and shared scenarios define reusable obligations.
5. Text, Poem, jsonrpsee, and downstream interpreters witness the meaning.
6. Procedural-macro expansion implements a projection of authored code; it does not redefine it.
7. Prose summarizes the artifacts above.

If these disagree, do not silently choose the framework or macro expansion. State the mismatch and
fix the smallest authoritative layer that is wrong.

## Read order

For design work, read from meaning to machinery:

1. `README.md`
2. `DENOTATIONAL_DESIGN.md`
3. `ARCHITECTURE.md`
4. `alux-ext/src/lib.rs`
5. `alux-http/src/algebra.rs`, `output.rs`, and `program.rs` for HTTP work
6. `alux-jsonrpc/src/algebra.rs` and `program.rs` for JSON-RPC work
7. the relevant interpreter crate
8. `alux-ext-macros` only after the first-order target meaning is clear
9. tests that compare interpretations

Do not begin with generated tokens or a framework callback merely because it is concrete.

## Commands

The CI-equivalent commands are:

```sh
just fmt
just build
just clippy
just doc
just test
just package
```

`just ci` runs all six in that order. The [`Justfile`](Justfile) holds the full command every recipe
stands for. Run the narrowest relevant test during development and `just ci` before finishing.

## Design by Meaning

This is a Rust adaptation of Conal Elliott's Denotational Design, taught in full in the
[ALUX programming guidelines](https://alux-network.github.io/alux-programming/). Read the lineage,
limits, and review rules in `DENOTATIONAL_DESIGN.md`; do not reduce the method to “use traits and
macros.”

- Ask what value, observation, transformation, or composition is specified before choosing syntax,
  storage, callbacks, framework types, or generated code.
- Define primitive domain meaning in small downstream capability traits.
- Put derived behavior in `#[ext]` implementations over explicit `where` clauses.
- Treat an extension's bounds as its semantic dependency declaration.
- Reify a method only when its application must become first-order data.
- Preserve context, argument product, argument names, and output as operation meaning.
- Describe transport surfaces as neutral program trees before selecting a framework.
- Interpret one program into execution, documentation, metadata, tests, or another backend.
- Keep concrete interpreters thin. They own extraction, conversion, registration, and runtime
  handles; they do not own the domain operation or interface composition.
- Keep macro lowering syntax-directed. It must not infer domain policy or invent hidden routes,
  methods, arguments, or outputs.
- Add a primitive only when new meaning is required. Add syntax only when new composition must be
  preserved for interpreters.

Target shape:

```text
downstream capability traits
    -> derived extension methods
    -> first-order operation values
    -> neutral HTTP / JSON-RPC programs
    -> independent interpreters
    -> shared laws and scenarios
```

Reject these shapes:

- a universal application/context trait
- transport declarations coupled to application state
- a framework macro as the only API specification
- a framework dependency, optional or not, in a specification crate
- route or method lists duplicated for execution and metadata
- handler argument/output types repeated after operation types already preserve them
- framework policy hidden in `Default`
- generated code treated as more authoritative than its public first-order target
- domain algebras added to this workspace only to share examples

## Semantic boundaries

`OperationAlg` preserves the context algebra, argument product, and source argument names.
`ApplyAlg` preserves application and its output. `HandlerContextAlg` lets an interpreter choose an
owned runtime handle without exposing `Arc`, framework data extractors, or executor constraints in a
domain method.

HTTP programs preserve method/path selectors, input roles, output roles, nesting, and route
coproduct. JSON-RPC programs preserve method names, operation signatures, program merge, and
positional or named decoding. Framework response/request values belong only in concrete
interpreters.

Externality is boundary-relative. A downstream domain fact may be primitive to one specification
and derived by a lower layer. ALUX Rust does not erase that distinction with a universal value or
context. The downstream extension bounds must state the meanings it consumes.

## Capability composition

Choose the form that states where meaning lives:

| Situation | Composition form |
| --- | --- |
| One receiver interprets several capabilities | Minimal direct bounds on the extension |
| A separate value selects behavior | Explicit capability parameter |
| Independent policies compose statically | Ordinary product/struct |
| An environment carries independent interpreters | Small `HasX` projections |
| A wrapper substitutes for an inner interpreter | Thin delegation |

Projection says “has”; delegation says “is.” Do not add either merely for method-call convenience.
Transport interpreter aliases may bundle repeated mechanical requirements, but they must not become
semantic god traits.

## Macro rules

- The ordinary extension method remains the authored source of derived meaning.
- Plain `defunc` produces an operation value implementing the public `OperationAlg`/`ApplyAlg`
  contract.
- `defunc(via = http)` and `defunc(via = jsonrpc)` lower convenient syntax to the same public
  first-order program values users can construct directly.
- Macro errors must point to authored syntax and explain the violated semantic constraint.
- One transport backend states only what its own transport means. Recognizing authored shapes belongs
  to `syntax`, and the shared declaration-to-program lowering belongs to `lower`; a backend supplies
  its declaration evidence, its nested-program obligation, and its compilation impl through
  `ProgramBackendAlg`.
- Generated names and paths are public compatibility commitments once downstream crates use them.
- Generated code must not reference private modules or this workspace's test fixtures.
- Every lowering change requires token-level expansion tests and a compiling downstream use.
- Crate-local declarative macros use `macro_rules!` plus `pub(crate) use`; use `#[macro_export]` only
  for an intentional external API.

## Rust conventions

- Inherit edition, minimum Rust version, license, and common metadata from the workspace manifest.
- Declare dependencies in root `[workspace.dependencies]`; members use `dependency.workspace = true`.
- Use major versions for stable dependencies and minor versions for `0.x` dependencies.
- Enable only required dependency features. Framework dependencies belong to interpreter crates.
- Avoid bounds on trait and struct definitions unless the definition is meaningless without them.
  Put bounds at use sites.
- Use `alux_ext::ext` for derived operations exposed by this workspace.
- Import generated extension traits and call methods directly. Do not use extension traits as
  semantic bounds or fully qualified calls unless genuine ambiguity requires it.
- Name associated types as type parameters when this makes bounds clearer. Use fully qualified
  projections only for genuine ambiguity.
- Prefer iterator combinators when they make dataflow clearer than an explicit loop.
- Root modules define the product surface. Re-export intentionally and keep engineering submodules
  private unless their paths are meaningful public vocabulary.
- Order declarations and imports by visibility: `pub`, `pub(crate)`, then private. Keep imports at
  module scope and same-visibility imports contiguous.
- Express a capability alias as a trait with supertraits plus one blanket implementation, so it adds
  no dependency and states that one interpreter answers all of them.
- Unsafe code is forbidden by workspace lint policy.
- Public items require concise documentation describing meaning. Use US English and begin doc
  comments with a third-person singular verb where practical.
- A specification crate's `README.md` is its crate documentation through
  `#![doc = include_str!("../README.md")]`. Write its examples as executed doctests rather than
  `ignore` blocks, so the published introduction cannot drift from the compiling surface.

## Tests and laws

- Test meaning through public operations and programs, not private representation.
- Compare at least two interpreters when claiming an interface is interpreter-independent.
- Keep reusable expectation functions independent of a particular domain algebra when practical.
- Test direct first-order syntax as well as macro-lowered convenience syntax.
- Read a specification crate's manifest: only `alux-ext` belongs there, and no build states that.
- Test each interpreter crate to prove it preserves the same program.
- Prefer identity, associativity, nesting, ordering, round-trip, and interpretation-agreement laws.
- Treat finite tests as executable evidence, not universal proof.

Current key witnesses are:

- extension invocation agrees with generated `ApplyAlg`
- direct and macro-lowered HTTP programs compile through the same fold
- HTTP route identity, coproduct, selector composition, and nesting hold in the text interpretation
- one HTTP program compiled by Poem and by text exposes the same ordered route surface
- specification-first JSON-RPC and native jsonrpsee services satisfy one shared scenario
- each specification crate's README example compiles and runs as a doctest, including direct
  first-order program construction without an interpreter

## Publication

Packages publish in dependency order:

1. `alux-ext-macros`
2. `alux-ext`
3. `alux-http` and `alux-jsonrpc`
4. `alux-http-text`, `alux-http-poem`, and `alux-jsonrpc-jsonrpsee`

Allow the registry index to observe each dependency before publishing the next package. Public
traits, generated operation names, syntax types, the dependency list of a specification crate, and
macro behavior are compatibility surface. Rejecting an authored form is part of that surface: the
diagnostic explaining it is the only documentation a rejected form has, and accepting it later changes
what the same source means. Use semantic versioning and document migrations for breaking changes.

## Change workflow

1. State the semantic change in one sentence.
2. Identify the smallest primitive capability or program distinction it needs.
3. Define or refine the public algebra/syntax before changing macros or frameworks.
4. Implement the generic fold or derived operation.
5. Update macro lowering only when ergonomic source syntax must project to that public meaning.
6. Interpret the meaning in concrete frameworks only after the neutral layer is stable.
7. Add laws, expansion checks, and shared scenarios.
8. Update architecture, DD guidance, READMEs, and migration notes when the public concept changes.
9. Run the full CI gate.

Use Conventional Commits 1.0.0. Format subjects as
`<type>(<scope>): <imperative summary>`.
