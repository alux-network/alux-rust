//! Interpreting a shape as a Rust layout.
//!
//! The derive reads a term out of a layout; this reads a layout out of a term. Neither direction is
//! privileged, which is what it means for a struct to be a carrier for a shape rather than its
//! source — so a shape stated with no layout behind it can be given one when domain code wants a
//! value to hold.

use alux_shape::{FieldAlg, ShapeAlg, Sorts, Spelling, Words};
use std::collections::BTreeMap;

/// A shape, as this interpretation carries one: the type written at a use site, the declarations that
/// use requires, and — when it is a product — the members another product would merge.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Layout {
    ty: String,
    declarations: BTreeMap<String, String>,
    product: Option<Product>,
}

/// What a product states, before a name turns it into a declaration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Product {
    members: Vec<RustMember>,
}

/// One member of a product, as a field would state it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RustMember {
    /// A field of its own, under the words naming it.
    Field { words: Vec<String>, ty: String, declarations: BTreeMap<String, String> },
    /// Another product's members, held by a field that carries them.
    Merge { ty: String, declarations: BTreeMap<String, String> },
}

impl RustMember {
    /// The declarations this member's shape depends on.
    fn declarations(&self) -> &BTreeMap<String, String> {
        match self {
            Self::Field { declarations, .. } | Self::Merge { declarations, .. } => declarations,
        }
    }
}

impl Layout {
    /// The type written where this shape is used.
    #[must_use]
    pub fn ty(&self) -> &str {
        &self.ty
    }

    /// Every declaration this layout depends on, in name order.
    #[must_use]
    pub fn module(&self) -> String {
        let declarations: Vec<&str> = self.declarations.values().map(String::as_str).collect();

        declarations.join("\n\n")
    }

    /// A layout written as `ty`, depending on the declarations gathered from `parts`.
    fn of(ty: impl Into<String>, parts: &[&Self]) -> Self {
        let mut declarations = BTreeMap::new();

        for part in parts {
            declarations.extend(part.declarations.clone());
        }

        Self { ty: ty.into(), declarations, product: None }
    }
}

/// Interprets a shape as a Rust layout whose serialization writes names this way.
///
/// A field is named in snake case, as Rust names one, and the spelling the surface writes is stated
/// once for the whole declaration — which is available to state because the term carries words and
/// not a spelling.
#[derive(Debug, Clone)]
pub struct RustShape {
    wire: Spelling,
    known: Vec<(Vec<String>, String)>,
}

impl RustShape {
    /// Emits layouts whose serialization spells names this way.
    #[must_use]
    pub fn new(wire: Spelling) -> Self {
        Self { wire, known: Vec::new() }
    }

    /// States that a name is already a type here, and what it is called.
    ///
    /// A domain's leaves are not the primitives they are written as: two of them may share
    /// `int(false, 64)` and remain different types. A name is what tells them apart, so a name this
    /// host already has a type for is written as that type rather than declared again.
    #[must_use]
    pub fn known(mut self, words: Words<'_>, path: impl Into<String>) -> Self {
        self.known.push((words.iter().map(|word| (*word).to_owned()).collect(), path.into()));

        self
    }

    /// The type this host already has for a name, if it has one.
    fn known_type(&self, words: Words<'_>) -> Option<&str> {
        self.known
            .iter()
            .find(|(known, _)| known.iter().map(String::as_str).eq(words.iter().copied()))
            .map(|(_, path)| path.as_str())
    }

    /// The attribute stating how this layout's names are written, when it is not how Rust writes them.
    fn rename_all(&self) -> Option<&'static str> {
        match self.wire {
            Spelling::Snake => None,
            Spelling::LowerCamel => Some("camelCase"),
            Spelling::UpperCamel => Some("PascalCase"),
            Spelling::Kebab => Some("kebab-case"),
            Spelling::Screaming => Some("SCREAMING_SNAKE_CASE"),
        }
    }

    /// Writes the attributes a declaration carries.
    fn attributes(&self) -> String {
        let derive = "#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]";

        match self.rename_all() {
            Some(rename) => format!("{derive}\n#[serde(rename_all = \"{rename}\")]"),
            None => derive.to_owned(),
        }
    }
}

impl Sorts for RustShape {
    type Ty = Layout;
    type Field = RustMember;
}

impl ShapeAlg for RustShape {
    fn truth(&self) -> Layout {
        Layout::of("bool", &[])
    }

    fn unit(&self) -> Layout {
        Layout::of("()", &[])
    }

    fn text(&self) -> Layout {
        Layout::of("String", &[])
    }

    fn literal(&self, _text: &str) -> Layout {
        // A constant is not a type. What a layout can hold is the text, and what the constant states
        // is left to whatever writes it.
        Layout::of("String", &[])
    }

    fn name_word(&self, _words: Words<'_>) -> Layout {
        Layout::of("String", &[])
    }

    fn int(&self, signed: bool, bits: u16) -> Layout {
        let sign = if signed { 'i' } else { 'u' };

        Layout::of(format!("{sign}{bits}"), &[])
    }

    fn float(&self, bits: u16) -> Layout {
        Layout::of(format!("f{bits}"), &[])
    }

    fn bytes(&self, len: Option<usize>) -> Layout {
        match len {
            Some(len) => Layout::of(format!("[u8; {len}]"), &[]),
            None => Layout::of("Vec<u8>", &[]),
        }
    }

    fn hex(&self, item: Layout) -> Layout {
        // The writing is a wrapper, so the layout keeps the value and the wrapper states how it is
        // written. A layout is generated against wrappers that serialize accordingly.
        Layout::of(format!("Hex<{}>", item.ty), &[&item])
    }

    fn decimal(&self, item: Layout) -> Layout {
        Layout::of(format!("Decimal<{}>", item.ty), &[&item])
    }

    fn base64(&self, item: Layout) -> Layout {
        Layout::of(format!("Base64<{}>", item.ty), &[&item])
    }

    fn opt(&self, item: Layout) -> Layout {
        Layout::of(format!("Option<{}>", item.ty), &[&item])
    }

    fn seq(&self, item: Layout) -> Layout {
        Layout::of(format!("Vec<{}>", item.ty), &[&item])
    }

    fn map(&self, key: Layout, value: Layout) -> Layout {
        Layout::of(format!("std::collections::BTreeMap<{}, {}>", key.ty, value.ty), &[&key, &value])
    }

    fn product(&self, fields: Vec<RustMember>) -> Layout {
        let mut declarations = BTreeMap::new();

        for field in &fields {
            declarations.extend(field.declarations().clone());
        }

        // A product becomes a type only once it is named, since Rust states no anonymous record.
        Layout { ty: "()".to_owned(), declarations, product: Some(Product { members: fields }) }
    }

    fn choice(&self, alternatives: Vec<Layout>) -> Layout {
        // A choice is a type once it is named, and until then it is only its alternatives.
        let parts: Vec<&Layout> = alternatives.iter().collect();
        let mut layout = Layout::of("()", &parts);
        layout.product = None;
        layout.ty = alternatives.iter().map(|alternative| alternative.ty.clone()).collect::<Vec<_>>().join(" | ");

        layout
    }

    fn named(&self, words: Words<'_>, body: Layout) -> Layout {
        // A name this host already has a type for is that type; nothing is declared for it.
        if let Some(path) = self.known_type(words) {
            return Layout::of(path, &[&body]);
        }

        let name = Spelling::UpperCamel.spell(words);
        let declaration = self.declare(&name, &body);
        let mut layout = Layout::of(name.clone(), &[&body]);
        layout.declarations.insert(name, declaration);
        layout.product = body.product;

        layout
    }

    fn reference(&self, words: Words<'_>) -> Layout {
        Layout::of(Spelling::UpperCamel.spell(words), &[])
    }
}

impl RustShape {
    /// Declares a name for a shape: a struct where the shape is a product, an alias otherwise.
    fn declare(&self, name: &str, body: &Layout) -> String {
        let attributes = self.attributes();

        match &body.product {
            Some(product) => {
                let fields = product
                    .members
                    .iter()
                    .map(|member| match member {
                        RustMember::Field { words, ty, .. } => {
                            let field: Vec<&str> = words.iter().map(String::as_str).collect();

                            format!("    pub {}: {ty},", Spelling::Snake.spell(&field))
                        }
                        RustMember::Merge { ty, .. } => format!(
                            "    #[serde(flatten)]\n    pub {}: {ty},",
                            Spelling::Snake.spell(&[&camel_to_snake(ty)]),
                        ),
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                format!("{attributes}\npub struct {name} {{\n{fields}\n}}")
            }
            None => format!("pub type {name} = {};", body.ty),
        }
    }
}

/// Names a type in the case a field is named in.
fn camel_to_snake(ty: &str) -> String {
    let mut out = String::new();

    for (index, character) in ty.chars().enumerate() {
        if character.is_uppercase() && index > 0 {
            out.push('_');
        }

        out.extend(character.to_lowercase());
    }

    out
}

impl FieldAlg for RustShape {
    fn field(&self, words: Words<'_>, shape: Layout) -> RustMember {
        RustMember::Field {
            words: words.iter().map(|word| (*word).to_owned()).collect(),
            ty: shape.ty,
            declarations: shape.declarations,
        }
    }

    fn merge(&self, shape: Layout) -> RustMember {
        RustMember::Merge { ty: shape.ty, declarations: shape.declarations }
    }
}
