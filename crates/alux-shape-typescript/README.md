# alux-shape-typescript

`alux-shape-typescript` interprets an [`alux-shape`](../../alux-shape) term as TypeScript declarations.

```rust
use alux_shape::{FieldAlg, ShapeAlg, ShapeExt, Spelling};
use alux_shape_typescript::TsShape;

let alg = TsShape::new(Spelling::LowerCamel);
let checksum = alg.field(&["checksum"], alg.bytes_hex(Some(32)));
let display_name = alg.field(&["display", "name"], alg.text());

let user = alg.named_product(&["user"], vec![checksum, display_name]);

assert_eq!(user.expr(), "User");
println!("{}", user.module());
```

```ts
export interface User {
  checksum: string
  displayName: string
}
```

A shape carries what stands for it where it is used, and the declarations that use depends on. So a
name is declared once however often it appears, and `module()` answers with every declaration a shape
needs, in name order.

Members are spelled as the surface spells them. A type's own name is written in pascal case, which is
TypeScript's convention rather than the shape's statement.

Three readings worth stating, because each is a decision rather than a translation:

- An integer is `number` at every width, since that is what a JSON number is. A width beyond what
  that holds exactly is the reason a domain writes such a quantity as text, which reads as `string`.
- Bare bytes are `never`. Nothing states how they are written, so no value inhabits them;
  `bytes_hex` reads as `string`.
- A merged product becomes an intersection, so a product with nothing merged in is an `interface` and
  one with something merged in is a type alias. Both are the same shape; only the declaration differs.
