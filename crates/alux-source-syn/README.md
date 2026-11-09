# alux-source-syn

`alux-source-syn` reads Rust source with [`syn`](https://docs.rs/syn) as a term over [`alux-source`](https://docs.rs/alux-source)'s `SourceAlg`, for any interpreter of it: `alg.read_source(file, text)` for one source, `alg.read_tree(dir)` for every `.rs` file under a directory, each declaration placed in its file and inline modules.

It reads trait definitions, trait implementations, and `#[ext]` blocks, `pub impl` included, which `syn` alone refuses. It reads syntax only: no name is resolved and no macro expanded, so an item a macro generates is not seen.

```rust
use alux_source_index::{Index, IndexQuestionsExt};
use alux_source_syn::ReadSourceExt;

let source = r#"
    pub trait CounterAlg {
        fn bump(&mut self);
    }

    #[ext(name = CounterLaws, defunc(via = laws))]
    pub impl<This> This
    where
        This: CounterAlg,
    {
        fn bump_adds_one(&mut self) {}
    }

    impl CounterAlg for Good {
        fn bump(&mut self) {}
    }
"#;
let declared = Index.read_source("counter.rs", source).unwrap();

assert_eq!(declared.laws_of("CounterAlg")[0].name(), "CounterLaws");
assert_eq!(declared.impls_of("CounterAlg")[0].name(), "Good");
```
