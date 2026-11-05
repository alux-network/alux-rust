# alux-shape-syn

`alux-shape-syn` reads the [`alux-shape`](https://docs.rs/alux-shape) shape a Rust layout states, as syntax: a struct's members or an enum's alternatives, named by `serde`'s `rename` or by the words of their identifiers, with `skip`, `flatten`, tagging (`tag`, `content`, `untagged`), and the forms a member states with `#[written(..)]`.

It is one reading for every reader of a layout. The `Shape` derive renders a `Layout` as code calling the shape algebra, and a reader of Rust source renders it as a term. It depends on no part of `alux-shape`, so the derive that `alux-shape` re-exports can use it.

A member's type is kept as syntax (`Form::Type`): what shape a type states is for the reader to say, the derive by asking the compiler for its `ShapeOf`, a source reader by its spelling.

```rust
use alux_shape_syn::{Body, Member, read_layout};

let input = syn::parse_quote! {
    #[serde(rename_all = "camelCase")]
    struct UserProfile {
        display_name: String,
        #[serde(skip)]
        secret: u64,
    }
};
let layout = read_layout(&input).unwrap();

assert_eq!(layout.words, ["user", "profile"]);
let Body::Product(members) = &layout.body else { panic!() };
assert!(matches!(&members[..], [Member::Field(words, _)] if words == &["display", "name"]));
```
