# alux-shape-morph

`alux-shape-morph` transforms an [`alux-shape`](../../alux-shape) term by standing between it and
another interpretation.

A term is polymorphic in its algebra, so an algebra can be interposed. That is what a transformation
is: not an edit applied to a term already built, but an interpretation that rebuilds one.

```rust
use alux_shape::{FieldAlg, ShapeAlg, ShapeExt, Spelling};
use alux_shape_morph::Patch;
use alux_shape_typescript::TsShape;

/// One shape, stated once, folded by whichever algebra it is handed.
fn user<A>(alg: &A) -> A::Ty
where
    A: ShapeAlg + FieldAlg,
{
    let display_name = alg.field(&["display", "name"], alg.text());

    alg.named_product(&["user"], vec![display_name])
}

let alg = TsShape::new(Spelling::LowerCamel);

assert_eq!(user(&alg).expr(), "User");
assert_eq!(user(&Patch(alg)).expr(), "UserPatch");
```

Both declarations come out of one statement, and they can be told apart because the name moved with
the shape. Appending a word composes where editing a string would not, which is what names-as-words
buys a second time.

`Patch` makes every member optional. `Prefixed` puts one word before every name, which keeps two
surfaces' declarations apart where both are emitted into one place. They compose, since each is only
another algebra:

```rust ignore
user(&Prefixed::new(Patch(alg), "admin")).expr() == "AdminUserPatch"
```

## What a transformation is not

It does not remove a member. The member sort has no empty member, so a shape with one fewer member is
a *different* shape — and `merge` already states that relation the other way round: the lesser shape
is stated, and the greater one merges it. That only ever appends, which is the direction a positional
reading survives, and it makes the shared part a declaration rather than something two shapes happen
to agree about.

One limit worth knowing: a transformation of `named` reaches every name in the term, not only the
outermost. A patch of a shape whose members are themselves named shapes renames those too, which is
right when a nested record is also being patched and wrong when it is not. Telling the outermost name
from an inner one needs the fold to carry depth, which it does not.
