# ALUX Rust

[![Build and Test][ga-badge]][ga-url]
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> [!WARNING]
> **Operational developers, welcome.** There is very little code here. Nothing to step through, no god
> object, no framework to blame, nothing to prove the work was hard.
>
> Side effects may include angst, dizziness, and phantom breakpoints when a whole subsystem fits on one
> screen. All temporary. Do not operate heavy inheritance hierarchies until they pass.
>
> Treatment: read the [ALUX programming guidelines](https://alux-network.github.io/alux-programming/)
> and [`DENOTATIONAL_DESIGN.md`](DENOTATIONAL_DESIGN.md). The meaning is in the trait signatures and
> the `where` clauses. The code you were looking for is the code that did not have to exist.
>
> Most developers recover fully and go on to delete things happily.

Reusable Design by Meaning infrastructure for ALUX specifications.

ALUX expects many independently published specification crates. This workspace provides the common
operation and interface-program vocabulary without centralizing their domain algebras. Each crate is
one or the other: it states what something means, or it makes something of that statement.

## Specifications

What a surface means, with no interpreter in it. These depend only on `alux-ext` and sit at the top
level of the repository.

| Crate | | Responsibility |
| --- | --- | --- |
| [`alux-http`](alux-http) | [![crates.io][v-http]][c-http] [![docs.rs][d-http]][r-http] | HTTP programs, independent of a web framework |
| [`alux-jsonrpc`](alux-jsonrpc) | [![crates.io][v-rpc]][c-rpc] [![docs.rs][d-rpc]][r-rpc] | JSON-RPC programs, independent of an RPC framework |
| [`alux-shape`](alux-shape)<br>[`alux-shape-macros`](alux-shape-macros) | [![crates.io][v-shape]][c-shape] [![docs.rs][d-shape]][r-shape]<br>[![crates.io][v-shape-macros]][c-shape-macros] [![docs.rs][d-shape-macros]][r-shape-macros] | Data shapes, independent of any encoder<br>The derive reading a shape out of a layout |
| [`alux-ext`](alux-ext)<br>[`alux-ext-macros`](alux-ext-macros) | [![crates.io][v-ext]][c-ext] [![docs.rs][d-ext]][r-ext]<br>[![crates.io][v-macros]][c-macros] [![docs.rs][d-macros]][r-macros] | First-order operations and context handles<br>The `ext` attribute and the macros that lower it |
| [`alux-sdk`](alux-sdk)<br>[`alux-sdk-macros`](alux-sdk-macros)<br>[`alux-traversable`](alux-traversable) | [![crates.io][v-sdk]][c-sdk] [![docs.rs][d-sdk]][r-sdk]<br>[![crates.io][v-sdk-macros]][c-sdk-macros] [![docs.rs][d-sdk-macros]][r-sdk-macros]<br>[![crates.io][v-trav]][c-trav] [![docs.rs][d-trav]][r-trav] | Transformations kept as expressions<br>The macros it exports<br>`traverse` over `Option` and iterators, re-exported by `alux-sdk` |

## Interpretations

What a library makes of a specification. Each is published separately and lives under
[`crates/`](crates), so the distinction shows in the layout and not only in the dependency graph. A
specification never names one of these, which is what lets the same statement have several.

### Of a transport program

| Crate | | Interprets a program as |
| --- | --- | --- |
| [`alux-http-poem`](crates/alux-http-poem) | [![crates.io][v-poem]][c-poem] [![docs.rs][d-poem]][r-poem] | executable Poem routes |
| [`alux-http-text`](crates/alux-http-text) | [![crates.io][v-text]][c-text] [![docs.rs][d-text]][r-text] | documentation or metadata |
| [`alux-jsonrpc-jsonrpsee`](crates/alux-jsonrpc-jsonrpsee) | [![crates.io][v-rpsee]][c-rpsee] [![docs.rs][d-rpsee]][r-rpsee] | jsonrpsee `Methods` |
| [`alux-jsonrpc-direct`](crates/alux-jsonrpc-direct) | [![crates.io][v-direct]][c-direct] [![docs.rs][d-direct]][r-direct] | a JSON-RPC 2.0 message handler, with no framework |
| [`alux-jsonrpc-typescript`](crates/alux-jsonrpc-typescript) | [![crates.io][v-rpc-ts]][c-rpc-ts] [![docs.rs][d-rpc-ts]][r-rpc-ts] | a TypeScript client module |
| [`alux-tokio`](crates/alux-tokio) | [![crates.io][v-tokio]][c-tokio] [![docs.rs][d-tokio]][r-tokio] | delivery over bounded Tokio channels |

### Of a data shape

| Crate | | Interprets a shape as |
| --- | --- | --- |
| [`alux-shape-rust`](crates/alux-shape-rust) | [![crates.io][v-shape-rust]][c-shape-rust] [![docs.rs][d-shape-rust]][r-shape-rust] | a Rust layout |
| [`alux-shape-typescript`](crates/alux-shape-typescript) | [![crates.io][v-shape-ts]][c-shape-ts] [![docs.rs][d-shape-ts]][r-shape-ts] | TypeScript declarations |
| [`alux-shape-json`](crates/alux-shape-json) | [![crates.io][v-shape-json]][c-shape-json] [![docs.rs][d-shape-json]][r-shape-json] | a decision about a JSON value |
| [`alux-shape-text`](crates/alux-shape-text) | [![crates.io][v-shape-text]][c-shape-text] [![docs.rs][d-shape-text]][r-shape-text] | a readable description |
| [`alux-shape-term`](crates/alux-shape-term) | [![crates.io][v-shape-term]][c-shape-term] [![docs.rs][d-shape-term]][r-shape-term] | the term itself |
| [`alux-shape-morph`](crates/alux-shape-morph) | [![crates.io][v-shape-morph]][c-shape-morph] [![docs.rs][d-shape-morph]][r-shape-morph] | another shape, by standing between |

## Semantic shape

```text
published domain spec
    -> tiny capability traits and derived extensions
    -> first-order operations
    -> portable HTTP / JSON-RPC programs
    -> separately published interpreter crate
```

Domain specifications depend on `alux-ext` and whichever transport program crates they expose.
Applications add the interpreter crates they run and configure them.

## Documentation

The [ALUX programming guidelines](https://alux-network.github.io/alux-programming/) teach the method
this workspace follows: designing programs by meaning first, in the style of Conal Elliott's
Denotational Design, with a concrete path into Rust. Read them for the reasoning behind the rules
below.

- [`DENOTATIONAL_DESIGN.md`](DENOTATIONAL_DESIGN.md) defines the meaning-first methodology and review rules.
- [`ARCHITECTURE.md`](ARCHITECTURE.md) maps those rules to the operation and transport-program algebras.
- [`AGENTS.md`](AGENTS.md) is the compact engineering authority and CI checklist.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) describes contribution expectations.

## Development

```sh
just ci
```

That runs, in order, `just fmt`, `just build`, `just clippy`, `just doc`, `just test`, and
`just package`. Each is also runnable on its own during development. The [`Justfile`](Justfile) holds
the full command every recipe stands for.

## Publication

Packages are licensed under MIT. Publish and verify them in this order, allowing the crates.io index
to update between dependent packages:

1. `alux-ext-macros`, `alux-sdk-macros`, and `alux-shape-macros` in any order
2. `alux-ext`
3. `alux-http`, `alux-jsonrpc`, `alux-shape`, and `alux-traversable` in any order
4. `alux-sdk`, `alux-shape-json`, `alux-shape-text`, and `alux-shape-typescript` in any order
5. `alux-http-text`, `alux-http-poem`, `alux-jsonrpc-jsonrpsee`, `alux-jsonrpc-direct`,
   `alux-jsonrpc-typescript`, `alux-shape-rust`, `alux-shape-term`, `alux-shape-morph`, and
   `alux-tokio` in any order

Cargo cannot fully package later steps against crates.io until the preceding package version is
available there.

[ga-badge]: https://github.com/alux-network/alux-rust/actions/workflows/rust.yml/badge.svg?branch=master
[ga-url]: https://github.com/alux-network/alux-rust/actions?query=branch:master

[v-http]: https://img.shields.io/crates/v/alux-http
[c-http]: https://crates.io/crates/alux-http
[d-http]: https://docs.rs/alux-http/badge.svg
[r-http]: https://docs.rs/alux-http
[v-rpc]: https://img.shields.io/crates/v/alux-jsonrpc
[c-rpc]: https://crates.io/crates/alux-jsonrpc
[d-rpc]: https://docs.rs/alux-jsonrpc/badge.svg
[r-rpc]: https://docs.rs/alux-jsonrpc
[v-ext]: https://img.shields.io/crates/v/alux-ext
[c-ext]: https://crates.io/crates/alux-ext
[d-ext]: https://docs.rs/alux-ext/badge.svg
[r-ext]: https://docs.rs/alux-ext
[v-macros]: https://img.shields.io/crates/v/alux-ext-macros
[c-macros]: https://crates.io/crates/alux-ext-macros
[d-macros]: https://docs.rs/alux-ext-macros/badge.svg
[r-macros]: https://docs.rs/alux-ext-macros
[v-sdk]: https://img.shields.io/crates/v/alux-sdk
[c-sdk]: https://crates.io/crates/alux-sdk
[d-sdk]: https://docs.rs/alux-sdk/badge.svg
[r-sdk]: https://docs.rs/alux-sdk

[v-sdk-macros]: https://img.shields.io/crates/v/alux-sdk-macros
[c-sdk-macros]: https://crates.io/crates/alux-sdk-macros
[d-sdk-macros]: https://docs.rs/alux-sdk-macros/badge.svg
[r-sdk-macros]: https://docs.rs/alux-sdk-macros

[v-trav]: https://img.shields.io/crates/v/alux-traversable
[c-trav]: https://crates.io/crates/alux-traversable
[d-trav]: https://docs.rs/alux-traversable/badge.svg
[r-trav]: https://docs.rs/alux-traversable

[v-tokio]: https://img.shields.io/crates/v/alux-tokio
[c-tokio]: https://crates.io/crates/alux-tokio
[d-tokio]: https://docs.rs/alux-tokio/badge.svg
[r-tokio]: https://docs.rs/alux-tokio

[v-shape]: https://img.shields.io/crates/v/alux-shape
[c-shape]: https://crates.io/crates/alux-shape
[d-shape]: https://docs.rs/alux-shape/badge.svg
[r-shape]: https://docs.rs/alux-shape

[v-shape-macros]: https://img.shields.io/crates/v/alux-shape-macros
[c-shape-macros]: https://crates.io/crates/alux-shape-macros
[d-shape-macros]: https://docs.rs/alux-shape-macros/badge.svg
[r-shape-macros]: https://docs.rs/alux-shape-macros

[v-shape-text]: https://img.shields.io/crates/v/alux-shape-text
[c-shape-text]: https://crates.io/crates/alux-shape-text
[d-shape-text]: https://docs.rs/alux-shape-text/badge.svg
[r-shape-text]: https://docs.rs/alux-shape-text

[v-shape-json]: https://img.shields.io/crates/v/alux-shape-json
[c-shape-json]: https://crates.io/crates/alux-shape-json
[d-shape-json]: https://docs.rs/alux-shape-json/badge.svg
[r-shape-json]: https://docs.rs/alux-shape-json

[v-shape-rust]: https://img.shields.io/crates/v/alux-shape-rust
[c-shape-rust]: https://crates.io/crates/alux-shape-rust
[d-shape-rust]: https://docs.rs/alux-shape-rust/badge.svg
[r-shape-rust]: https://docs.rs/alux-shape-rust

[v-shape-ts]: https://img.shields.io/crates/v/alux-shape-typescript
[c-shape-ts]: https://crates.io/crates/alux-shape-typescript
[d-shape-ts]: https://docs.rs/alux-shape-typescript/badge.svg
[r-shape-ts]: https://docs.rs/alux-shape-typescript

[v-shape-term]: https://img.shields.io/crates/v/alux-shape-term
[c-shape-term]: https://crates.io/crates/alux-shape-term
[d-shape-term]: https://docs.rs/alux-shape-term/badge.svg
[r-shape-term]: https://docs.rs/alux-shape-term

[v-shape-morph]: https://img.shields.io/crates/v/alux-shape-morph
[c-shape-morph]: https://crates.io/crates/alux-shape-morph
[d-shape-morph]: https://docs.rs/alux-shape-morph/badge.svg
[r-shape-morph]: https://docs.rs/alux-shape-morph

[v-rpc-ts]: https://img.shields.io/crates/v/alux-jsonrpc-typescript
[c-rpc-ts]: https://crates.io/crates/alux-jsonrpc-typescript
[d-rpc-ts]: https://docs.rs/alux-jsonrpc-typescript/badge.svg
[r-rpc-ts]: https://docs.rs/alux-jsonrpc-typescript

[v-text]: https://img.shields.io/crates/v/alux-http-text
[c-text]: https://crates.io/crates/alux-http-text
[d-text]: https://docs.rs/alux-http-text/badge.svg
[r-text]: https://docs.rs/alux-http-text
[v-poem]: https://img.shields.io/crates/v/alux-http-poem
[c-poem]: https://crates.io/crates/alux-http-poem
[d-poem]: https://docs.rs/alux-http-poem/badge.svg
[r-poem]: https://docs.rs/alux-http-poem
[v-direct]: https://img.shields.io/crates/v/alux-jsonrpc-direct
[c-direct]: https://crates.io/crates/alux-jsonrpc-direct
[d-direct]: https://docs.rs/alux-jsonrpc-direct/badge.svg
[r-direct]: https://docs.rs/alux-jsonrpc-direct

[v-rpsee]: https://img.shields.io/crates/v/alux-jsonrpc-jsonrpsee
[c-rpsee]: https://crates.io/crates/alux-jsonrpc-jsonrpsee
[d-rpsee]: https://docs.rs/alux-jsonrpc-jsonrpsee/badge.svg
[r-rpsee]: https://docs.rs/alux-jsonrpc-jsonrpsee
