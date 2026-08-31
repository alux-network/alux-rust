//! Interpreting a shape as TypeScript declarations.

use alux_shape::{FieldAlg, ShapeAlg, Sorts, Spelling, Words};
use std::collections::BTreeMap;

/// A shape, as this interpretation carries one: what to write where it is used, the declarations that
/// use requires, and — when it is a product — the members another product would merge.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TsType {
    /// What stands for this shape at a use site.
    expr: String,
    /// Every declaration this use depends on, by the name it declares.
    declarations: BTreeMap<String, String>,
    /// The members of a product, kept so that merging one into another is expressible.
    product: Option<Product>,
}

/// The two ways a product states its members.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Product {
    /// Members written here, as `name: type`.
    members: Vec<String>,
    /// Products merged in, as the types they are written by.
    merges: Vec<String>,
}

/// One member of a product.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TsMember {
    /// A member written under its own name.
    Named { text: String, declarations: BTreeMap<String, String> },
    /// Another product, observed as this one's members.
    Merged { expr: String, declarations: BTreeMap<String, String> },
}

impl TsMember {
    /// The declarations this member's shape depends on.
    fn declarations(&self) -> &BTreeMap<String, String> {
        match self {
            Self::Named { declarations, .. } | Self::Merged { declarations, .. } => declarations,
        }
    }
}

impl TsType {
    /// What stands for this shape where it is used.
    #[must_use]
    pub fn expr(&self) -> &str {
        &self.expr
    }

    /// Every declaration this shape depends on, as a name and the declaration it states.
    pub fn declarations(&self) -> impl Iterator<Item = (&str, &str)> {
        self.declarations.iter().map(|(name, declaration)| (name.as_str(), declaration.as_str()))
    }

    /// The module this shape needs: every declaration it depends on, in name order.
    #[must_use]
    pub fn module(&self) -> String {
        let declarations: Vec<&str> = self.declarations.values().map(String::as_str).collect();

        declarations.join("\n\n")
    }

    /// A shape written by an expression, depending on the declarations gathered from `parts`.
    fn of(expr: impl Into<String>, parts: &[&Self]) -> Self {
        let mut declarations = BTreeMap::new();

        for part in parts {
            declarations.extend(part.declarations.clone());
        }

        Self { expr: expr.into(), declarations, product: None }
    }
}

/// Wraps an expression where TypeScript reads it as more than one type.
fn grouped(expr: &str) -> String {
    if expr.contains(['|', '&']) { format!("({expr})") } else { expr.to_owned() }
}

/// Interprets a shape as TypeScript, spelling member names as the surface spells them.
///
/// A type's own name is always written in pascal case, which is TypeScript's convention rather than
/// this shape's statement.
#[derive(Debug, Clone, Copy)]
pub struct TsShape {
    members: Spelling,
}

impl TsShape {
    /// Emits declarations whose members are spelled this way.
    #[must_use]
    pub fn new(members: Spelling) -> Self {
        Self { members }
    }
}

impl Sorts for TsShape {
    type Ty = TsType;
    type Field = TsMember;
}

impl ShapeAlg for TsShape {
    fn truth(&self) -> TsType {
        TsType::of("boolean", &[])
    }

    fn unit(&self) -> TsType {
        TsType::of("null", &[])
    }

    fn text(&self) -> TsType {
        TsType::of("string", &[])
    }

    fn literal(&self, text: &str) -> TsType {
        TsType::of(format!("\"{text}\""), &[])
    }

    fn name_word(&self, words: Words<'_>) -> TsType {
        TsType::of(format!("\"{}\"", self.members.spell(words)), &[])
    }

    fn int(&self, _signed: bool, _bits: u16) -> TsType {
        // A JSON number is a `number` at every width. A width beyond what that holds exactly is why
        // a domain writes such a quantity as text instead, which reads as `string` here.
        TsType::of("number", &[])
    }

    fn float(&self, _bits: u16) -> TsType {
        TsType::of("number", &[])
    }

    fn bytes(&self, _len: Option<usize>) -> TsType {
        // Bytes alone are written no way at all, so no value inhabits them.
        TsType::of("never", &[])
    }

    fn hex(&self, item: TsType) -> TsType {
        TsType::of("string", &[&item])
    }

    fn decimal(&self, item: TsType) -> TsType {
        TsType::of("string", &[&item])
    }

    fn base64(&self, item: TsType) -> TsType {
        TsType::of("string", &[&item])
    }

    fn opt(&self, item: TsType) -> TsType {
        TsType::of(format!("{} | null", item.expr), &[&item])
    }

    fn seq(&self, item: TsType) -> TsType {
        TsType::of(format!("{}[]", grouped(&item.expr)), &[&item])
    }

    fn map(&self, key: TsType, value: TsType) -> TsType {
        // A JSON key is text however the shape describes it, so the key states the value's type.
        TsType::of(format!("Record<string, {}>", value.expr), &[&key, &value])
    }

    fn product(&self, fields: Vec<TsMember>) -> TsType {
        let mut product = Product::default();
        let mut declarations = BTreeMap::new();

        for field in &fields {
            declarations.extend(field.declarations().clone());

            match field {
                TsMember::Named { text, .. } => product.members.push(text.clone()),
                TsMember::Merged { expr, .. } => product.merges.push(expr.clone()),
            }
        }

        let expr = intersection(&product);

        TsType { expr, declarations, product: Some(product) }
    }

    fn choice(&self, alternatives: Vec<TsType>) -> TsType {
        let expr = alternatives.iter().map(|alternative| alternative.expr.clone()).collect::<Vec<_>>().join(" | ");
        let parts: Vec<&TsType> = alternatives.iter().collect();

        TsType::of(expr, &parts)
    }

    fn named(&self, words: Words<'_>, body: TsType) -> TsType {
        let name = Spelling::UpperCamel.spell(words);
        let declaration = declare(&name, &body);
        let mut shape = TsType::of(name.clone(), &[&body]);
        shape.declarations.insert(name, declaration);
        // A named product is still a product, so another product can merge it.
        shape.product = body.product;

        shape
    }

    fn reference(&self, words: Words<'_>) -> TsType {
        // The name alone, since whatever introduced it declares it.
        TsType::of(Spelling::UpperCamel.spell(words), &[])
    }
}

/// Writes a product as one type: its own members, and whatever is merged into it.
fn intersection(product: &Product) -> String {
    let own = format!("{{ {} }}", product.members.join("; "));

    match (product.members.is_empty(), product.merges.is_empty()) {
        (_, true) => own,
        (true, false) => product.merges.join(" & "),
        (false, false) => format!("{own} & {}", product.merges.join(" & ")),
    }
}

/// Declares a name for a shape, as an interface where TypeScript has one and an alias otherwise.
fn declare(name: &str, body: &TsType) -> String {
    match &body.product {
        // An interface reads better than an alias, and only a product with nothing merged in is one.
        Some(product) if product.merges.is_empty() => {
            let members = product.members.iter().map(|member| format!("  {member}")).collect::<Vec<_>>().join("\n");

            format!("export interface {name} {{\n{members}\n}}")
        }
        _ => format!("export type {name} = {}", body.expr),
    }
}

impl FieldAlg for TsShape {
    fn field(&self, words: Words<'_>, shape: TsType) -> TsMember {
        TsMember::Named {
            text: format!("{}: {}", self.members.spell(words), shape.expr),
            declarations: shape.declarations,
        }
    }

    fn merge(&self, shape: TsType) -> TsMember {
        TsMember::Merged { expr: shape.expr, declarations: shape.declarations }
    }
}
