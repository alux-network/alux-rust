# alux-source

`alux-source` states what Rust source declares: which traits there are, which types implement them, and which extension blocks derive operations over which carriers, with what each requires and where it is written. Checking a specification by its meaning (which law suites a capability has, which types implement it, what a composition requires) reads these declarations, with no code compiled.

The crate is a specification. It depends on [`alux-ext`](https://docs.rs/alux-ext), [`alux-laws`](https://docs.rs/alux-laws) and [`alux-shape`](https://docs.rs/alux-shape), names no parser, and fixes no representation: a reader of source builds a term over `SourceAlg`, and each interpreter chooses what names, types, bounds and sources are.

## What source declares

| Item | Meaning |
| --- | --- |
| `SourceAlg::name`, `SourceAlg::ty` | A name or a type as the source writes it, before anything is resolved |
| `SourceAlg::bound` | A requirement that a type implement traits |
| `SourceAlg::trait_def` | A trait, the traits it extends, and every other bound its definition states |
| `SourceAlg::trait_impl` | A type implementing a trait, under bounds |
| `SourceAlg::extension` | An extension block: its methods over a carrier, the bounds they depend on, and the backend that reifies it |
| `DataAlg::data` | A struct or an enum: its name, its type parameters, and the `alux-shape` term it holds, a product for a struct and a choice for an enum |
| `SourceAlg::file`, `SourceAlg::module` | Where declarations are written |
| `SourceAlg::source_empty`, `SourceAlg::source_merge` | Declaring nothing, and declaring one source after another |

Data is stated over `alux-shape`: what a struct or an enum holds is a shape, as the `Shape` derive and every shape interpreter read it, and a member whose type is another declared type refers to it by name (`ShapeAlg::reference`).

A name is what the source spells, a path such as `alux_ext::OperationAlg`: which item it refers to is a later reading, so two spellings of one trait are two names here.

## Derived meaning

| Extension | Derives |
| --- | --- |
| `SourceFoldExt` | What several sources declare, in order |

## Laws

| Suite | States |
| --- | --- |
| `SourceLaws` | Declaring nothing is the identity of merging, merging is associative, a module distributes over merging, and an empty module declares nothing |

A source is a term over the algebra, stated once and read by any interpreter:

```rust
use alux_ext::ext;
use alux_source::{SourceAlg, SourceFoldExt};

/// What a counter's specification declares: the capability, its law suite, and a reference impl.
#[ext(name = CounterSourceExt)]
pub impl<This, Source> This
where
    This: SourceAlg<Source = Source>,
{
    fn counter_source(&self) -> Source {
        let carrier = self.ty("This");
        let requires = self.bound(carrier, vec![self.name("CounterAlg")]);
        self.file("counter.rs", self.source_all([
            self.trait_def(self.name("CounterAlg"), Vec::new(), Vec::new()),
            self.extension(
                self.name("CounterLaws"),
                self.ty("This"),
                vec![requires],
                Some(self.name("laws")),
                vec![self.name("bump_adds_one")],
            ),
            self.trait_impl(self.name("CounterAlg"), self.ty("Good"), Vec::new()),
        ]))
    }
}
```

[`alux-source-syn`](https://docs.rs/alux-source-syn) builds such terms by reading source, and [`alux-source-index`](https://docs.rs/alux-source-index) interprets them as plain values that answer questions.
