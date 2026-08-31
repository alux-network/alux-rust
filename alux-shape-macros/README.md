# alux-shape-macros

`alux-shape-macros` reads a shape out of a Rust layout. It states one edge of
[`alux-shape`](../alux-shape): a type that already carries `serde` attributes states its term through
them, so nothing is written twice.

```rust ignore
use alux_shape::Shape;
use serde::Serialize;

#[derive(Serialize, Shape)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u64,
    pub display_name: String,
    pub email: Option<String>,
    #[serde(flatten)]
    pub stamps: Timestamps,
    #[serde(skip)]
    pub secret: u64,
}
```

That derive states the term as one impl, over any interpretation rather than a chosen one:

```rust ignore
impl<Alg> ::alux_shape::ShapeOf<Alg> for User
where
    Alg: ::alux_shape::ShapeAlg + ::alux_shape::FieldAlg,
{
    type Shape = <Alg as ::alux_shape::Sorts>::Ty;
    fn shape_of(alg: &Alg) -> Self::Shape {
        use ::alux_shape::{FieldAlg as _, ShapeAlg as _, ShapeExt as _, ShapeTaggedExt as _};
        let body = {
            let member_0 = alg.field(&["id"], <u64 as ::alux_shape::ShapeOf<Alg>>::shape_of(alg));
            let member_1 = alg.field(&["display", "name"], <String as ::alux_shape::ShapeOf<Alg>>::shape_of(alg));
            let member_2 = alg.field(&["email"], <Option<String> as ::alux_shape::ShapeOf<Alg>>::shape_of(alg));
            let member_3 = alg.merge(<Timestamps as ::alux_shape::ShapeOf<Alg>>::shape_of(alg));
            alg.product(vec![member_0, member_1, member_2, member_3])
        };
        alg.named(&["user"], body)
    }
}
```

Three things in that expansion matter.

`Alg` stays a parameter, so the impl holds for every interpretation. Deriving `Shape` does not commit
the type to JSON, to TypeScript, or to a Rust layout.

Each member's shape comes from `ShapeOf` of that member's own type. `Timestamps` describes itself, and
this impl only assembles the pieces, so nesting needs no cooperation from the derive.

`rename_all` appears nowhere in it. That attribute says how names are spelled, and a term keeps only
the words, so whoever renders the term picks the spelling.

## What the derive reads

| In the layout | What it becomes |
|---|---|
| a member's identifier | its words, split on `_` |
| `#[serde(rename = "avatarBytes")]` | its words, read back out of the string |
| `#[serde(flatten)]` | a merge, so the inner product's members appear here |
| `#[serde(skip)]` or `skip_serializing` | nothing, the member is dropped |
| `#[serde(untagged)]`, `tag`, `tag` with `content` | which encoding of a choice to use |
| a variant's identifier | its words, split where the case changes |
| all-unit variants | a choice between names |
| `rename_all` | nothing, it is read while working out the words and then dropped |

Two of those work the words out instead of reading them off. A cased identifier splits where the case
changes, keeping a run of capitals whole, so `HTTPServer` gives `[http, server]`. A `rename` string can
be spelled any way, so its words are read back out of it. Where either would get it wrong, state the
shape as a declaration and give the words directly.

Four layouts are refused, each with a compile error rather than a shape that is quietly wrong:

- a type with type parameters, since a generic shape is written as a declaration instead
- a `union`, which describes no values
- a newtype or tuple struct, since a product is read from named members
- a tuple or struct variant, which is not read yet
