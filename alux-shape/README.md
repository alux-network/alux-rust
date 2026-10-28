# alux-shape

`alux-shape` describes the shape of data independently of any encoder.

The crate is a specification. It carries no interpreter and depends only on
[`alux-ext`](https://docs.rs/alux-ext): a shape is described here as first-order syntax over two
sorts, and an interpretation gives that description reality — a declaration in another language, a
judgement about a value, a writer of bytes, a memory layout. None of those is *the* type of the thing
described; each is what the term means to that interpreter.

```rust ignore
use alux_shape::{FieldAlg, ShapeAlg, ShapeExt, Words};

const USER: Words<'static> = &["user"];

/// A shape is a term, polymorphic in the algebra that interprets it.
fn user<A>(alg: &A) -> A::Ty
where
    A: ShapeAlg + FieldAlg,
{
    let id = alg.field(&["id"], alg.int(false, 64));
    let display_name = alg.field(&["display", "name"], alg.text());
    let checksum = alg.field(&["checksum"], alg.bytes_hex(Some(32)));

    alg.named_product(USER, vec![id, display_name, checksum])
}
```

## A name is a sequence of words

`[display, name]` is a name. `displayName`, `display_name`, and `DisplayName` are three spellings
of it, and which one appears is settled where the term is interpreted. Holding words rather than a cased string is
what makes that exact: converting between cases is heuristic wherever acronyms or digits meet a
boundary, while joining words is not, so the same term spells identically in every host because none
of them re-segments anything.

`ShapeAlg::named` states an identity, not an instruction. An interpretation that declares types emits
one and refers to it; one that types structurally may spell the shape at each use.

## Two ways in

A type that already exists states its term through `#[derive(Shape)]`, re-exported here and
implemented by [`alux-shape-macros`](../macros/alux-shape-macros): it reads the `serde` attributes already
present, so the shape and the serialization are two readings of one annotation. A shape with no
layout behind it is declared over the algebra instead.

Either way the term is the same, which is what lets a type move between them: fold both readings
through an interpretation and compare.

## What it owns, and what it does not

It owns eighteen primitives and the operations they derive. Every tagged encoding of a choice is a
choice of products over a name-valued discriminant, so `sum_external`, `sum_internal`,
`sum_adjacent`, `sum_untagged`, and `sum_of_names` are stated once in an extension rather than being
primitives an interpretation could disagree about.

It owns no spelling, no encoder, and no layout. Whether a member is read by its name or by its
position is not stated in a term either, and cannot be: a member is a lens, and which coordinate
addresses it is settled when a shape is composed with a transport.

## Making a value from its members

A shape describes a value and never builds one. An interpretation that reads a value member by member, from command-line words, named parameters, or a form, also needs to make it: `Product` states a value as its fields taken together, and `Sum` as the one alternative of `Sum2` through `Sum16` it carries, an alternative with no value being `()`. Each is an isomorphism, stated by `ProductLawsExt` and `SumLawsExt`, and names its members as identifiers of the words the shape names them by.

`#[derive(Shape)]` states both beside the shape, from the same layout and the same `serde` attributes:

| Layout | Shape | Construction |
|---|---|---|
| struct with named members | yes | `Product`, one field per member |
| member with `#[serde(skip)]` | left out | no field; made with `Default` |
| member with `#[serde(flatten)]` | merged | one field carrying the whole value |
| enum of 2 to 16 alternatives, each one value or none | yes | `Sum` |
| enum of one alternative, or more than 16 | yes | none |
| newtype or tuple struct, tuple or struct alternative, union | rejected where it is written | none |

A layout with type parameters is read too: its shape holds where each member's type has one, and its construction always holds. A use that needs a construction the derive did not state names the missing `Product` or `Sum` and these rules.

## Seeing one in a language

A term says nothing about how a host writes it, so what a shape looks like in a language is an
interpretation of the term rather than a property of it. Two exist for languages people read:

- [`alux-shape-rust`](../crates/alux-shape-rust) reads a term as a Rust layout, `struct` declarations with
  the `serde` attributes that make the encoding agree with the shape, and a written form for each leaf
  so `Hex<[u8; 32]>` stays distinct from the bytes it writes.
- [`alux-shape-typescript`](../crates/alux-shape-typescript) reads the same term as TypeScript declarations,
  where a product is an interface, a merge is an intersection, and an optional member is a union with
  `null`.

The same value folds through both, so a type stated once is available in either language without being
written in either. Four more interpretations exist for other purposes:
[`alux-shape-text`](../crates/alux-shape-text) for a readable description,
[`alux-shape-json`](../crates/alux-shape-json) for a decision about a JSON value,
[`alux-shape-term`](../crates/alux-shape-term) for the term itself, and
[`alux-shape-morph`](../crates/alux-shape-morph) for a shape stated in terms of another.

And you can write your own. An interpretation is two carrier types and the primitives over them,
`Sorts`, `ShapeAlg`, and `FieldAlg`. Everything derived comes with them: the five encodings of a choice
are stated once in an extension, so an interpretation that never mentions them still has them.
Nothing in this crate learns about a new language, which is the point of the term being first-order.
