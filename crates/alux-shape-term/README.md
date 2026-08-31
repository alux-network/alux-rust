# alux-shape-term

`alux-shape-term` interprets an [`alux-shape`](../../alux-shape) term as the term itself, so a shape can
be written down and read back.

```rust
use alux_shape::{FieldAlg, ShapeAlg, ShapeExt};
use alux_shape_term::{Term, TermShape};

let alg = TermShape;
let display_name = alg.field(&["display", "name"], alg.text());
let written = alg.named_product(&["user"], vec![display_name]);

// The written term is a value, so it travels as one.
let document = serde_json::to_string(&written).expect("a term serializes");
let read: Term = serde_json::from_str(&document).expect("and reads back");

assert_eq!(read, written);
```

Every other interpretation answers with something that is not a shape — text, a judgement, a
declaration in another language. This one answers with the shape, which is what lets a term leave the
host that stated it: written here, read there, and folded into whichever algebra that host carries.

`Term::fold` is the reader. Folding a written term with any interpretation answers what interpreting
the original directly would, which is the law this crate is checked by — and the reason a host needs
only a writer and a reader rather than one adapter per pair of hosts.
