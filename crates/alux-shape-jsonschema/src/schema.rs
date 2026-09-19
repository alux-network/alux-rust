//! Renders a shape as the JSON Schema that describes it.

use alux_shape::{FieldAlg, ShapeAlg, Sorts, Spelling, Words};
use core::cell::RefCell;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;

/// A schema, and whether a member carrying it may be absent.
///
/// JSON Schema states absence on the product rather than on the member, so what `opt` means has to
/// travel with the shape until the product it belongs to is stated.
#[derive(Debug, Clone)]
pub struct JsonSchema {
    schema: Value,
    optional: bool,
}

impl JsonSchema {
    /// Returns the schema this states.
    pub fn into_value(self) -> Value {
        self.schema
    }

    /// Returns whether a member carrying this may be absent.
    pub fn is_optional(&self) -> bool {
        self.optional
    }

    fn stated(schema: Value) -> Self {
        Self { schema, optional: false }
    }
}

/// One member of a product, or another product's members observed as this one's.
#[derive(Debug, Clone)]
pub enum JsonMember {
    /// A member under the name it is written with.
    Named {
        /// The name this member is written under.
        name: String,
        /// The schema this member carries.
        schema: Value,
        /// Whether this member must be present.
        required: bool,
    },
    /// Another product's members, observed as this product's own.
    Merged(Value),
}

/// Renders a shape as JSON Schema, naming shapes under one reference prefix.
#[derive(Debug)]
pub struct JsonSchemaShape {
    members: Spelling,
    prefix: String,
    definitions: RefCell<BTreeMap<String, Value>>,
}

impl Default for JsonSchemaShape {
    fn default() -> Self {
        Self::new(Spelling::Snake, "#/$defs/")
    }
}

impl JsonSchemaShape {
    /// Renders shapes, spelling members as `members` does and naming them under `prefix`.
    ///
    /// A shape that states a name is a type, which a document names the way a type is named, so only
    /// members take the spelling a surface chose. The prefix is where a document keeps the shapes it
    /// names: a schema on its own keeps them in `#/$defs/`, and an `OpenAPI` document keeps them in
    /// `#/components/schemas/`.
    pub fn new(members: Spelling, prefix: &str) -> Self {
        Self { members, prefix: prefix.to_owned(), definitions: RefCell::new(BTreeMap::new()) }
    }

    /// Returns every shape that stated a name, under the name it stated.
    pub fn definitions(&self) -> BTreeMap<String, Value> {
        self.definitions.borrow().clone()
    }

    fn spell(&self, words: Words<'_>) -> String {
        self.members.spell(words)
    }

    fn name(words: Words<'_>) -> String {
        Spelling::UpperCamel.spell(words)
    }

    fn refers_to(&self, name: &str) -> Value {
        json!({ "$ref": format!("{}{name}", self.prefix) })
    }
}

impl Sorts for JsonSchemaShape {
    type Ty = JsonSchema;
    type Field = JsonMember;
}

impl ShapeAlg for JsonSchemaShape {
    fn truth(&self) -> JsonSchema {
        JsonSchema::stated(json!({ "type": "boolean" }))
    }

    fn unit(&self) -> JsonSchema {
        JsonSchema::stated(json!({ "type": "null" }))
    }

    fn text(&self) -> JsonSchema {
        JsonSchema::stated(json!({ "type": "string" }))
    }

    fn literal(&self, text: &str) -> JsonSchema {
        JsonSchema::stated(json!({ "const": text }))
    }

    fn name_word(&self, words: Words<'_>) -> JsonSchema {
        JsonSchema::stated(json!({ "const": self.spell(words) }))
    }

    fn int(&self, signed: bool, bits: u16) -> JsonSchema {
        let mut schema = Map::new();
        schema.insert("type".into(), "integer".into());
        if matches!(bits, 32 | 64) {
            schema.insert("format".into(), format!("int{bits}").into());
        }
        if !signed {
            schema.insert("minimum".into(), 0.into());
        }

        JsonSchema::stated(Value::Object(schema))
    }

    fn float(&self, bits: u16) -> JsonSchema {
        let format = if bits <= 32 { "float" } else { "double" };

        JsonSchema::stated(json!({ "type": "number", "format": format }))
    }

    fn bytes(&self, len: Option<usize>) -> JsonSchema {
        let mut schema = json!({ "type": "array", "items": { "type": "integer", "minimum": 0, "maximum": 255 } });
        if let (Some(len), Some(schema)) = (len, schema.as_object_mut()) {
            schema.insert("minItems".into(), len.into());
            schema.insert("maxItems".into(), len.into());
        }

        JsonSchema::stated(schema)
    }

    fn hex(&self, _item: JsonSchema) -> JsonSchema {
        JsonSchema::stated(json!({ "type": "string", "pattern": "^0x[0-9a-fA-F]*$" }))
    }

    fn decimal(&self, _item: JsonSchema) -> JsonSchema {
        JsonSchema::stated(json!({ "type": "string", "pattern": "^-?[0-9]+$" }))
    }

    fn base64(&self, _item: JsonSchema) -> JsonSchema {
        JsonSchema::stated(json!({ "type": "string", "contentEncoding": "base64" }))
    }

    fn opt(&self, item: JsonSchema) -> JsonSchema {
        JsonSchema { schema: json!({ "anyOf": [item.schema, { "type": "null" }] }), optional: true }
    }

    fn seq(&self, item: JsonSchema) -> JsonSchema {
        JsonSchema::stated(json!({ "type": "array", "items": item.schema }))
    }

    fn map(&self, key: JsonSchema, value: JsonSchema) -> JsonSchema {
        JsonSchema::stated(json!({
            "type": "object",
            "propertyNames": key.schema,
            "additionalProperties": value.schema,
        }))
    }

    fn product(&self, fields: Vec<JsonMember>) -> JsonSchema {
        let mut properties = Map::new();
        let mut required = Vec::new();
        let mut merged = Vec::new();
        for field in fields {
            match field {
                JsonMember::Named { name, schema, required: needed } => {
                    if needed {
                        required.push(Value::from(name.clone()));
                    }
                    properties.insert(name, schema);
                }
                JsonMember::Merged(schema) => merged.push(schema),
            }
        }

        let mut stated = Map::new();
        stated.insert("type".into(), "object".into());
        stated.insert("properties".into(), Value::Object(properties));
        if !required.is_empty() {
            stated.insert("required".into(), Value::Array(required));
        }

        if merged.is_empty() {
            return JsonSchema::stated(Value::Object(stated));
        }

        // Two products seen as one is what every schema of them has to hold at once.
        merged.insert(0, Value::Object(stated));

        JsonSchema::stated(json!({ "allOf": merged }))
    }

    fn choice(&self, alternatives: Vec<JsonSchema>) -> JsonSchema {
        let alternatives = alternatives.into_iter().map(JsonSchema::into_value).collect::<Vec<_>>();

        JsonSchema::stated(json!({ "anyOf": alternatives }))
    }

    fn named(&self, words: Words<'_>, body: JsonSchema) -> JsonSchema {
        let name = Self::name(words);
        self.definitions.borrow_mut().insert(name.clone(), body.schema);

        JsonSchema::stated(self.refers_to(&name))
    }

    fn reference(&self, words: Words<'_>) -> JsonSchema {
        JsonSchema::stated(self.refers_to(&Self::name(words)))
    }
}

impl FieldAlg for JsonSchemaShape {
    fn field(&self, words: Words<'_>, shape: JsonSchema) -> JsonMember {
        JsonMember::Named { name: self.spell(words), required: !shape.optional, schema: shape.schema }
    }

    fn merge(&self, shape: JsonSchema) -> JsonMember {
        JsonMember::Merged(shape.schema)
    }
}
