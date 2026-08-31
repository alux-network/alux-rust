# alux-shape-text

`alux-shape-text` interprets an [`alux-shape`](../../alux-shape) term as a readable description of the
shape it states. It encodes nothing and decodes nothing, so it witnesses that a shape term denotes
leaves, members, alternatives and names rather than any one encoder's behavior.

```rust
use alux_shape::{FieldAlg, ShapeAlg, ShapeExt};
use alux_shape_text::TextShape;

let alg = TextShape;
let checksum = alg.field(&["checksum"], alg.bytes_hex(Some(32)));
let display_name = alg.field(&["display", "name"], alg.text());

let user = alg.named_product(&["user"], vec![checksum, display_name]);

assert_eq!(user, "user { checksum: hex bytes<32>, display_name: text }");
```

Names are spelled here as the words they are, joined by `_`. That is this interpretation's choice and
not the term's: another interpretation of the same term spells `displayName`.

Being an ordinary interpretation, it is also what the derived operations' laws are checked against —
two terms that denote one shape render one text, so `tests/laws.rs` states each derivation as an
equality between a derived operation and the primitives it stands for.
