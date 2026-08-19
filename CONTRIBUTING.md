# Contributing to ALUX Rust

Read [`AGENTS.md`](AGENTS.md) before changing code. It defines the repository's authority order,
Design by Meaning rules, Rust conventions, publication boundaries, and required checks. Read
[`DENOTATIONAL_DESIGN.md`](DENOTATIONAL_DESIGN.md) for the methodology and
[`ARCHITECTURE.md`](ARCHITECTURE.md) for the concrete workspace structure.

## Development

Install Rust 1.97, `just`, and `cargo-nextest`, then run:

```sh
just build
just test
just clippy
just doc
```

Dependencies belong in the root workspace manifest. Member crates inherit them with
`dependency.workspace = true`. Framework dependencies must remain optional and disabled by default.

## Pull requests

A semantic change should include:

- the meaning being added or changed
- the smallest algebra or first-order syntax required by that meaning
- the generic fold or derived operation that interprets the new syntax
- a reusable law or focused invariant check
- at least two interpretations when the change claims interpreter independence
- documentation updates when terminology, public API, or architecture changes

A procedural-macro change must also include expansion tests and a downstream compile/use test when
it changes generated public code.

Before submitting, run `just ci`. Use Conventional Commits 1.0.0 for commit messages.
