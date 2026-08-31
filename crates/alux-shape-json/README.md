# alux-shape-json

`alux-shape-json` interprets an [`alux-shape`](../../alux-shape) term as a decision about a JSON value.

```rust
use alux_shape::{FieldAlg, ShapeAlg, ShapeExt, Spelling};
use alux_shape_json::Judge;
use serde_json::json;

let alg = Judge::new(Spelling::LowerCamel);
let checksum = alg.field(&["checksum"], alg.bytes_hex(Some(2)));
let display_name = alg.field(&["display", "name"], alg.text());

let user = alg.named_product(&["user"], vec![checksum, display_name]);

assert!(user.holds(&json!({"checksum": "0xbeef", "displayName": "ada"})).is_ok());

// The surface says how a name is written, so a differently spelled key is a different value.
assert!(user.holds(&json!({"checksum": "0xbeef", "display_name": "ada"})).is_err());
```

This is the interpretation that makes a shape answerable rather than merely describable: a term either
describes what was written or names where it did not. Judging is strict in both directions — a member
the shape describes must be present unless it may be absent, and a member the value carries must be
described — because a shape that quietly tolerates an extra key cannot be trusted to state what a
type writes.

A name is spelled here as the surface spells it, which is why `Judge::new` takes a `Spelling` and the
term does not carry one.

Two things it does not decide on its own. Bare bytes describe no JSON value, since nothing states how
they are written; `bytes_hex` or `base64` does. And a `reference` describes anything, because
resolving a name needs the whole shape at once rather than the fold that builds it — so a term read
back by [`alux-shape-term`](../alux-shape-term) is resolved first, and then a name states what it
introduced.
