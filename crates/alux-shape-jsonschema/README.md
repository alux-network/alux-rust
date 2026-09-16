# alux-shape-jsonschema

`alux-shape-jsonschema` interprets an [`alux-shape`](https://docs.rs/alux-shape) shape as the JSON
Schema that describes it.

A shape states what data is; this crate expresses the same meaning as JSON Schema. Named shapes are
collected as definitions and referenced under the appropriate prefix: `#/$defs/` for a standalone
schema, or `#/components/schemas/` for an `OpenAPI` document.

```rust ignore
use alux_shape::{ShapeOf, Spelling};
use alux_shape_jsonschema::JsonSchemaShape;

let schema = JsonSchemaShape::new(Spelling::LowerCamel, "#/components/schemas/");
let reading = u32::shape_of(&schema).into_value();

assert_eq!(reading, serde_json::json!({ "type": "integer", "format": "int32", "minimum": 0 }));
```
