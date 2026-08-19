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
operation and interface-program vocabulary without centralizing their domain algebras:

| Crate | Version | Docs | Responsibility |
| --- | --- | --- | --- |
| [`alux-http`](alux-http) | [![crates.io][v-http]][c-http] | [![docs.rs][d-http]][r-http] | Typed, composable HTTP programs,<br>independent of a web framework |
| [`alux-jsonrpc`](alux-jsonrpc) | [![crates.io][v-rpc]][c-rpc] | [![docs.rs][d-rpc]][r-rpc] | Typed, composable JSON-RPC programs,<br>independent of an RPC framework |
| [`alux-ext`](alux-ext)<br>[`alux-ext-macros`](alux-ext-macros) | [![crates.io][v-ext]][c-ext]<br>[![crates.io][v-macros]][c-macros] | [![docs.rs][d-ext]][r-ext]<br>[![docs.rs][d-macros]][r-macros] | The `ext` attribute, first-order operations, and context handles<br>The procedural macros that lower them |

The specification crates carry no interpreter and depend only on `alux-ext`. Each interpretation is
published separately as `alux-<program>-<interpreter>`:

| Crate | Version | Docs | Interpretation |
| --- | --- | --- | --- |
| [`alux-http-text`](alux-http-text) | [![crates.io][v-text]][c-text] | [![docs.rs][d-text]][r-text] | Interprets an HTTP program as documentation or metadata |
| [`alux-http-poem`](alux-http-poem) | [![crates.io][v-poem]][c-poem] | [![docs.rs][d-poem]][r-poem] | Interprets the same HTTP program as executable Poem routes |
| [`alux-jsonrpc-jsonrpsee`](alux-jsonrpc-jsonrpsee) | [![crates.io][v-rpsee]][c-rpsee] | [![docs.rs][d-rpsee]][r-rpsee] | Interprets a JSON-RPC program as jsonrpsee `Methods` |

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
cargo fmt --all -- --check
cargo build --workspace --all-features --all-targets
cargo clippy --workspace --all-features --all-targets -- -D warnings
cargo nextest run --workspace --all-features --no-fail-fast
cargo test --workspace --all-features --doc
```

The equivalent command is `just ci`.

## Publication

Packages are licensed under MIT. Publish and verify them in this order, allowing the crates.io index
to update between dependent packages:

1. `alux-ext-macros`
2. `alux-ext`
3. `alux-http` and `alux-jsonrpc` in either order
4. `alux-http-text`, `alux-http-poem`, and `alux-jsonrpc-jsonrpsee` in any order

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
[v-text]: https://img.shields.io/crates/v/alux-http-text
[c-text]: https://crates.io/crates/alux-http-text
[d-text]: https://docs.rs/alux-http-text/badge.svg
[r-text]: https://docs.rs/alux-http-text
[v-poem]: https://img.shields.io/crates/v/alux-http-poem
[c-poem]: https://crates.io/crates/alux-http-poem
[d-poem]: https://docs.rs/alux-http-poem/badge.svg
[r-poem]: https://docs.rs/alux-http-poem
[v-rpsee]: https://img.shields.io/crates/v/alux-jsonrpc-jsonrpsee
[c-rpsee]: https://crates.io/crates/alux-jsonrpc-jsonrpsee
[d-rpsee]: https://docs.rs/alux-jsonrpc-jsonrpsee/badge.svg
[r-rpsee]: https://docs.rs/alux-jsonrpc-jsonrpsee
