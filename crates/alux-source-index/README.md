# alux-source-index

`alux-source-index` interprets [`alux-source`](https://docs.rs/alux-source) as plain values: `Index` reads a source term into a list of `Declared`, each a declaration with the file and modules it is written in, names and types kept as written. `IndexQuestionsExt` answers questions over the list:

```text
declared.laws_of("CounterAlg")     the `via = laws` extensions whose carrier must implement it
declared.impls_of("CounterAlg")    the implementations of it
declared.bounded_trait_defs()      trait definitions stating bounds beyond their supertraits,
                                   other than Sized, Send, Sync, Clone and Debug
declared.extensions()              every extension block
```

A trait is asked for by its name, the last segment of its path, which every spelling of it shares.

```rust
use alux_source::{SourceAlg, SourceFoldExt};
use alux_source_index::{Index, IndexQuestionsExt};

let requires = Index.bound(Index.ty("This"), vec![Index.name("CounterAlg")]);
let declared = Index.file("counter.rs", Index.source_all([
    Index.extension(Index.name("CounterLaws"), Index.ty("This"), vec![requires], Some(Index.name("laws")), Vec::new()),
    Index.trait_impl(Index.name("crate::CounterAlg"), Index.ty("Good"), Vec::new()),
]));

assert_eq!(declared.laws_of("CounterAlg")[0].name(), "CounterLaws");
assert_eq!(declared.impls_of("CounterAlg")[0].name(), "Good");
assert_eq!(declared.impls_of("CounterAlg")[0].file, "counter.rs");
```
