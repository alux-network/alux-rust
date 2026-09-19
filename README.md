# ALUX Rust

[![Build and Test][ga-badge]][ga-url]
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Reusable Design by Meaning infrastructure for typed programs and their interpreters.

ALUX expects many independently published specification crates. This workspace provides the common
operation and interface-program vocabulary without centralizing their domain algebras. Each crate has
one role: it either defines a meaning, or interprets that meaning for a particular purpose.

## Specifications

What a surface means, with no interpreter in it. These depend only on `alux-ext` and sit at the top
level of the repository. The attribute or derive that goes with one is a separate crate, because a
procedural macro has to be, and those live under [`macros/`](macros).

| Crate | | Responsibility |
| --- | --- | --- |
| [`alux-http`](alux-http) | [![crates.io][v-http]][c-http] [![docs.rs][d-http]][r-http] | HTTP programs and server, no web framework |
| [`alux-jsonrpc`](alux-jsonrpc) | [![crates.io][v-rpc]][c-rpc] [![docs.rs][d-rpc]][r-rpc] | JSON-RPC programs, no RPC framework |
| [`alux-shape`](alux-shape)<br>[`alux-shape-macros`](macros/alux-shape-macros) | [![crates.io][v-shape]][c-shape] [![docs.rs][d-shape]][r-shape]<br>[![crates.io][v-shape-macros]][c-shape-macros] [![docs.rs][d-shape-macros]][r-shape-macros] | Data shapes, no encoder<br>The derive reading one out of a layout |
| [`alux-ext`](alux-ext)<br>[`alux-ext-macros`](macros/alux-ext-macros) | [![crates.io][v-ext]][c-ext] [![docs.rs][d-ext]][r-ext]<br>[![crates.io][v-macros]][c-macros] [![docs.rs][d-macros]][r-macros] | First-order operations and context handles<br>The `ext` attribute and its macros |
| [`alux-sdk`](alux-sdk)<br>[`alux-sdk-macros`](macros/alux-sdk-macros) | [![crates.io][v-sdk]][c-sdk] [![docs.rs][d-sdk]][r-sdk]<br>[![crates.io][v-sdk-macros]][c-sdk-macros] [![docs.rs][d-sdk-macros]][r-sdk-macros] | Transformations kept as expressions<br>The macros it exports |
| [`alux-traversable`](alux-traversable) | [![crates.io][v-trav]][c-trav] [![docs.rs][d-trav]][r-trav] | `traverse` over `Option` and iterators |
| [`alux-bench`](alux-bench) | [![crates.io][v-bench]][c-bench] [![docs.rs][d-bench]][r-bench] | What a benchmark measures, no harness |

## Interpretations

These crates are interpreters, or implementations, of the specifications above. They consume a
specification and interpret its meaning for a concrete target, such as a web framework, a document,
or a client. Each is published separately under [`crates/`](crates), so specifications can be used
without any particular interpreter and the same specification can have several implementations.

### Of an HTTP program

Framework-backed execution:

| Crate | | Interprets a program as |
| --- | --- | --- |
| [`alux-http-axum`](crates/alux-http-axum) | [![crates.io][v-axum]][c-axum] [![docs.rs][d-axum]][r-axum] | executable [axum](https://docs.rs/axum) routes |
| [`alux-http-actix`](crates/alux-http-actix) | [![crates.io][v-actix]][c-actix] [![docs.rs][d-actix]][r-actix] | executable [Actix Web](https://docs.rs/actix-web) routes |
| [`alux-http-rocket`](crates/alux-http-rocket) | [![crates.io][v-rocket]][c-rocket] [![docs.rs][d-rocket]][r-rocket] | executable [Rocket](https://docs.rs/rocket) routes |
| [`alux-http-warp`](crates/alux-http-warp) | [![crates.io][v-warp]][c-warp] [![docs.rs][d-warp]][r-warp] | executable [warp](https://docs.rs/warp) filters |
| [`alux-http-poem`](crates/alux-http-poem) | [![crates.io][v-poem]][c-poem] [![docs.rs][d-poem]][r-poem] | executable [Poem](https://docs.rs/poem) routes |
| [`alux-http-salvo`](crates/alux-http-salvo) | [![crates.io][v-salvo]][c-salvo] [![docs.rs][d-salvo]][r-salvo] | executable [Salvo](https://docs.rs/salvo) routes |

Framework-free execution:

| Crate | | Interprets a program as |
| --- | --- | --- |
| [`alux-http-hyper`](crates/alux-http-hyper) | [![crates.io][v-hyper]][c-hyper] [![docs.rs][d-hyper]][r-hyper] | a [hyper](https://docs.rs/hyper) service around the direct interpretation |
| [`alux-http-direct`](crates/alux-http-direct) | [![crates.io][v-http-direct]][c-http-direct] [![docs.rs][d-http-direct]][r-http-direct] | direct request and response handling |

Documentation and clients:

| Crate | | Interprets a program as |
| --- | --- | --- |
| [`alux-http-typescript`](crates/alux-http-typescript) | [![crates.io][v-http-ts]][c-http-ts] [![docs.rs][d-http-ts]][r-http-ts] | a TypeScript client module |
| [`alux-http-openapi`](crates/alux-http-openapi) | [![crates.io][v-openapi]][c-openapi] [![docs.rs][d-openapi]][r-openapi] | an OpenAPI document |
| [`alux-http-text`](crates/alux-http-text) | [![crates.io][v-text]][c-text] [![docs.rs][d-text]][r-text] | documentation or metadata |

Shared HTTP support:

| Crate | | Provides |
| --- | --- | --- |
| [`alux-http-parts`](crates/alux-http-parts) | [![crates.io][v-parts]][c-parts] [![docs.rs][d-parts]][r-parts] | shared multipart reading |
| [`alux-http-conformance`](crates/alux-http-conformance) | [![crates.io][v-conformance]][c-conformance] [![docs.rs][d-conformance]][r-conformance] | shared HTTP scenarios |

### Of a JSON-RPC program

| Crate | | Interprets a program as |
| --- | --- | --- |
| [`alux-jsonrpc-typescript`](crates/alux-jsonrpc-typescript) | [![crates.io][v-rpc-ts]][c-rpc-ts] [![docs.rs][d-rpc-ts]][r-rpc-ts] | a TypeScript client module |
| [`alux-jsonrpc-jsonrpsee`](crates/alux-jsonrpc-jsonrpsee) | [![crates.io][v-rpsee]][c-rpsee] [![docs.rs][d-rpsee]][r-rpsee] | [jsonrpsee](https://docs.rs/jsonrpsee) `Methods` |
| [`alux-jsonrpc-direct`](crates/alux-jsonrpc-direct) | [![crates.io][v-rpc-direct]][c-rpc-direct] [![docs.rs][d-rpc-direct]][r-rpc-direct] | a message handler, no framework |

### Of a data shape

| Crate | | Interprets a shape as |
| --- | --- | --- |
| [`alux-shape-rust`](crates/alux-shape-rust) | [![crates.io][v-shape-rust]][c-shape-rust] [![docs.rs][d-shape-rust]][r-shape-rust] | a Rust layout |
| [`alux-shape-typescript`](crates/alux-shape-typescript) | [![crates.io][v-shape-ts]][c-shape-ts] [![docs.rs][d-shape-ts]][r-shape-ts] | TypeScript declarations |
| [`alux-shape-json`](crates/alux-shape-json) | [![crates.io][v-shape-json]][c-shape-json] [![docs.rs][d-shape-json]][r-shape-json] | a decision about a JSON value |
| [`alux-shape-jsonschema`](crates/alux-shape-jsonschema) | [![crates.io][v-shape-jsonschema]][c-shape-jsonschema] [![docs.rs][d-shape-jsonschema]][r-shape-jsonschema] | a JSON Schema document |
| [`alux-shape-text`](crates/alux-shape-text) | [![crates.io][v-shape-text]][c-shape-text] [![docs.rs][d-shape-text]][r-shape-text] | a readable description |
| [`alux-shape-term`](crates/alux-shape-term) | [![crates.io][v-shape-term]][c-shape-term] [![docs.rs][d-shape-term]][r-shape-term] | the term itself |
| [`alux-shape-morph`](crates/alux-shape-morph) | [![crates.io][v-shape-morph]][c-shape-morph] [![docs.rs][d-shape-morph]][r-shape-morph] | another shape |

### Of a benchmark

| Crate | | Measures a stated bench with |
| --- | --- | --- |
| [`alux-bench-criterion`](crates/alux-bench-criterion) | [![crates.io][v-bench-criterion]][c-bench-criterion] [![docs.rs][d-bench-criterion]][r-bench-criterion] | [criterion](https://docs.rs/criterion) groups and functions |
| [`alux-bench-direct`](crates/alux-bench-direct) | [![crates.io][v-bench-direct]][c-bench-direct] [![docs.rs][d-bench-direct]][r-bench-direct] | its own runner, saying each case as it finishes |

### Other interpretations

| Crate | | Interprets a value as |
| --- | --- | --- |
| [`alux-tokio`](crates/alux-tokio) | [![crates.io][v-tokio]][c-tokio] [![docs.rs][d-tokio]][r-tokio] | bounded [Tokio](https://docs.rs/tokio) channels |

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
3. `alux-bench`, `alux-http`, `alux-jsonrpc`, `alux-shape`, and `alux-traversable` in any order
4. `alux-sdk`, `alux-shape-json`, `alux-shape-text`, and `alux-shape-typescript` in any order
5. `alux-shape-jsonschema`, `alux-http-parts`, `alux-http-text`, `alux-http-direct`,
   `alux-http-conformance`, `alux-http-poem`, `alux-http-actix`, `alux-http-axum`,
   `alux-http-hyper`, `alux-http-openapi`, `alux-http-rocket`, `alux-http-salvo`,
   `alux-http-typescript`, `alux-http-warp`, `alux-jsonrpc-jsonrpsee`, `alux-jsonrpc-direct`,
   `alux-jsonrpc-typescript`, `alux-shape-rust`, `alux-shape-term`, `alux-shape-morph`,
   `alux-bench-direct`, `alux-bench-criterion`, and `alux-tokio` in dependency order where needed

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
[v-actix]: https://img.shields.io/crates/v/alux-http-actix
[c-actix]: https://crates.io/crates/alux-http-actix
[d-actix]: https://docs.rs/alux-http-actix/badge.svg
[r-actix]: https://docs.rs/alux-http-actix
[v-axum]: https://img.shields.io/crates/v/alux-http-axum
[c-axum]: https://crates.io/crates/alux-http-axum
[d-axum]: https://docs.rs/alux-http-axum/badge.svg
[r-axum]: https://docs.rs/alux-http-axum
[v-conformance]: https://img.shields.io/crates/v/alux-http-conformance
[c-conformance]: https://crates.io/crates/alux-http-conformance
[d-conformance]: https://docs.rs/alux-http-conformance/badge.svg
[r-conformance]: https://docs.rs/alux-http-conformance
[v-http-direct]: https://img.shields.io/crates/v/alux-http-direct
[c-http-direct]: https://crates.io/crates/alux-http-direct
[d-http-direct]: https://docs.rs/alux-http-direct/badge.svg
[r-http-direct]: https://docs.rs/alux-http-direct
[v-hyper]: https://img.shields.io/crates/v/alux-http-hyper
[c-hyper]: https://crates.io/crates/alux-http-hyper
[d-hyper]: https://docs.rs/alux-http-hyper/badge.svg
[r-hyper]: https://docs.rs/alux-http-hyper
[v-openapi]: https://img.shields.io/crates/v/alux-http-openapi
[c-openapi]: https://crates.io/crates/alux-http-openapi
[d-openapi]: https://docs.rs/alux-http-openapi/badge.svg
[r-openapi]: https://docs.rs/alux-http-openapi
[v-bench]: https://img.shields.io/crates/v/alux-bench
[c-bench]: https://crates.io/crates/alux-bench
[d-bench]: https://docs.rs/alux-bench/badge.svg
[r-bench]: https://docs.rs/alux-bench
[v-bench-direct]: https://img.shields.io/crates/v/alux-bench-direct
[c-bench-direct]: https://crates.io/crates/alux-bench-direct
[d-bench-direct]: https://docs.rs/alux-bench-direct/badge.svg
[r-bench-direct]: https://docs.rs/alux-bench-direct
[v-bench-criterion]: https://img.shields.io/crates/v/alux-bench-criterion
[c-bench-criterion]: https://crates.io/crates/alux-bench-criterion
[d-bench-criterion]: https://docs.rs/alux-bench-criterion/badge.svg
[r-bench-criterion]: https://docs.rs/alux-bench-criterion
[v-parts]: https://img.shields.io/crates/v/alux-http-parts
[c-parts]: https://crates.io/crates/alux-http-parts
[d-parts]: https://docs.rs/alux-http-parts/badge.svg
[r-parts]: https://docs.rs/alux-http-parts
[v-rocket]: https://img.shields.io/crates/v/alux-http-rocket
[c-rocket]: https://crates.io/crates/alux-http-rocket
[d-rocket]: https://docs.rs/alux-http-rocket/badge.svg
[r-rocket]: https://docs.rs/alux-http-rocket
[v-salvo]: https://img.shields.io/crates/v/alux-http-salvo
[c-salvo]: https://crates.io/crates/alux-http-salvo
[d-salvo]: https://docs.rs/alux-http-salvo/badge.svg
[r-salvo]: https://docs.rs/alux-http-salvo
[v-http-ts]: https://img.shields.io/crates/v/alux-http-typescript
[c-http-ts]: https://crates.io/crates/alux-http-typescript
[d-http-ts]: https://docs.rs/alux-http-typescript/badge.svg
[r-http-ts]: https://docs.rs/alux-http-typescript
[v-warp]: https://img.shields.io/crates/v/alux-http-warp
[c-warp]: https://crates.io/crates/alux-http-warp
[d-warp]: https://docs.rs/alux-http-warp/badge.svg
[r-warp]: https://docs.rs/alux-http-warp
[v-rpc-direct]: https://img.shields.io/crates/v/alux-jsonrpc-direct
[c-rpc-direct]: https://crates.io/crates/alux-jsonrpc-direct
[d-rpc-direct]: https://docs.rs/alux-jsonrpc-direct/badge.svg
[r-rpc-direct]: https://docs.rs/alux-jsonrpc-direct
[v-shape-jsonschema]: https://img.shields.io/crates/v/alux-shape-jsonschema
[c-shape-jsonschema]: https://crates.io/crates/alux-shape-jsonschema
[d-shape-jsonschema]: https://docs.rs/alux-shape-jsonschema/badge.svg
[r-shape-jsonschema]: https://docs.rs/alux-shape-jsonschema
[v-direct]: https://img.shields.io/crates/v/alux-jsonrpc-direct
[c-direct]: https://crates.io/crates/alux-jsonrpc-direct
[d-direct]: https://docs.rs/alux-jsonrpc-direct/badge.svg
[r-direct]: https://docs.rs/alux-jsonrpc-direct

[v-rpsee]: https://img.shields.io/crates/v/alux-jsonrpc-jsonrpsee
[c-rpsee]: https://crates.io/crates/alux-jsonrpc-jsonrpsee
[d-rpsee]: https://docs.rs/alux-jsonrpc-jsonrpsee/badge.svg
[r-rpsee]: https://docs.rs/alux-jsonrpc-jsonrpsee
