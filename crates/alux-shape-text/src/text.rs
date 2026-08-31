//! Renders a shape as the text that describes it.

use alux_shape::{FieldAlg, ShapeAlg, Sorts, Words};

/// Renders a shape as text, spelling names as the words they are, joined by `_`.
#[derive(Debug, Clone, Copy, Default)]
pub struct TextShape;

/// Joins a name's words as this interpretation spells them.
fn spell(words: Words<'_>) -> String {
    words.join("_")
}

impl Sorts for TextShape {
    type Ty = String;
    type Field = String;
}

impl ShapeAlg for TextShape {
    fn truth(&self) -> String {
        "bool".into()
    }

    fn unit(&self) -> String {
        "unit".into()
    }

    fn text(&self) -> String {
        "text".into()
    }

    fn literal(&self, text: &str) -> String {
        format!("\"{text}\"")
    }

    fn name_word(&self, words: Words<'_>) -> String {
        format!("\"{}\"", spell(words))
    }

    fn int(&self, signed: bool, bits: u16) -> String {
        let sign = if signed { 'i' } else { 'u' };

        format!("{sign}{bits}")
    }

    fn float(&self, bits: u16) -> String {
        format!("f{bits}")
    }

    fn bytes(&self, len: Option<usize>) -> String {
        match len {
            Some(len) => format!("bytes<{len}>"),
            None => "bytes".into(),
        }
    }

    fn hex(&self, item: String) -> String {
        format!("hex {item}")
    }

    fn decimal(&self, item: String) -> String {
        format!("decimal {item}")
    }

    fn base64(&self, item: String) -> String {
        format!("base64 {item}")
    }

    fn opt(&self, item: String) -> String {
        format!("{item}?")
    }

    fn seq(&self, item: String) -> String {
        format!("[{item}]")
    }

    fn map(&self, key: String, value: String) -> String {
        format!("{{{key}: {value}}}")
    }

    fn product(&self, fields: Vec<String>) -> String {
        format!("{{ {} }}", fields.join(", "))
    }

    fn choice(&self, alternatives: Vec<String>) -> String {
        format!("({})", alternatives.join(" | "))
    }

    fn named(&self, words: Words, body: String) -> String {
        format!("{} {body}", spell(words))
    }

    fn reference(&self, words: Words<'_>) -> String {
        spell(words)
    }
}

impl FieldAlg for TextShape {
    fn field(&self, words: Words, shape: String) -> String {
        format!("{}: {shape}", spell(words))
    }

    fn merge(&self, shape: String) -> String {
        format!("..{shape}")
    }
}
