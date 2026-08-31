# alux-shape-rust

`alux-shape-rust` interprets an [`alux-shape`](../../alux-shape) term as a Rust layout.

The derive in [`alux-shape-macros`](../../alux-shape-macros) reads a term out of a layout; this reads a
layout out of a term. Neither direction is privileged, which is what it means for a struct to be a
carrier for a shape rather than its source — so a shape stated with no layout behind it can be given
one when domain code wants a value to hold.

```rust
use alux_shape::{FieldAlg, ShapeAlg, ShapeExt, Spelling};
use alux_shape_rust::RustShape;

let alg = RustShape::new(Spelling::LowerCamel);
let display_name = alg.field(&["display", "name"], alg.text());
let user = alg.named_product(&["user"], vec![display_name]);

assert_eq!(user.ty(), "User");
println!("{}", user.module());
```

```rust ignore
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub display_name: String,
}
```

A field is named in snake case, as Rust names one, and how the surface writes those names is stated
once for the whole declaration. That attribute is available to state precisely because a term carries
words rather than a spelling — nothing had to be recovered from the members to know they were all
written the same way.

## What a layout can and cannot recover

A term states less than a layout does, so some of what this writes is a decision:

- **A writing is a wrapper.** `hex(bytes(32))` reads as `Hex<[u8; 32]>`, so the layout keeps the value
  and a wrapper states how it is written. A generated layout is compiled against wrappers that
  serialize that way.
- **A constant is not a type.** `literal` and `name_word` read as `String`, since Rust states no type
  whose only value is one piece of text. What the constant states is left to whatever writes it.
- **A choice needs a name.** An unnamed product or choice reads as `()`, because Rust states no
  anonymous record: a name is what turns either into a declaration.
- **A tagged choice does not come back.** The tagged encodings are derived, so a term holds a choice of
  products and not the attribute that produced it. A layout generated from one describes the same
  values, but by different means than the `serde` attribute a person would have written.

That last one is the same blindness every interpretation has. A term states what is written, and
whichever attribute a layout would have used to write it is not part of that.
