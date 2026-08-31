//! A declaration, folded into the layout it never stated.
//!
//! Nothing here authors a struct. A declaration states members and names them with identifiers, and
//! this interpretation answers with the Rust a person would otherwise have written — which is the
//! whole of what it means for a layout to be a carrier for a shape rather than its source.

use alux_ext::ext;
use alux_ext::macros::shape;
use alux_shape::{FieldAlg, ShapeAlg, ShapeDeclareExt, ShapeExt, ShapeProgramExt, Sorts, Spelling};
use alux_shape_rust::RustShape;
use alux_shape_text::TextShape;

/// A domain names its leaves once.
#[ext(name = LeafExt, supertraits = Sorts + Sized)]
pub impl<This> This
where
    This: ShapeAlg,
{
    /// A 32-byte checksum, as this domain writes one.
    fn checksum(&self) -> Self::Ty {
        self.bytes_hex(Some(32))
    }

    /// An amount: a `u128`, so decimal text.
    fn amount(&self) -> Self::Ty {
        self.int_decimal(false, 128)
    }
}

#[ext(name = AccountShapeExt, defunc(via = shape))]
pub impl<This> This
where
    This: ShapeAlg + FieldAlg,
{
    /// When a record was made.
    fn timestamps_shape(&self) {
        self.record().field(created_at, self.int(false, 64))
    }

    /// An account, as a surface answers one.
    fn account_shape(&self) {
        self.record()
            .field(display_name, self.text())
            .field(balance, self.amount())
            .field(checksum, self.checksum())
            .merge(self.timestamps_shape())
    }
}

fn rust() -> RustShape {
    RustShape::new(Spelling::LowerCamel)
}

#[test]
fn a_declaration_states_a_layout_no_one_wrote() {
    let layout = rust().compile_shape(rust().account_shape());

    assert_eq!(layout.ty(), "Account");
    assert_eq!(
        layout.module(),
        [
            "#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]",
            "#[serde(rename_all = \"camelCase\")]",
            "pub struct Account {",
            "    pub display_name: String,",
            "    pub balance: Decimal<u128>,",
            "    pub checksum: Hex<[u8; 32]>,",
            "    #[serde(flatten)]",
            "    pub timestamps: Timestamps,",
            "}",
            "",
            "#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]",
            "#[serde(rename_all = \"camelCase\")]",
            "pub struct Timestamps {",
            "    pub created_at: u64,",
            "}",
        ]
        .join("\n"),
    );
}

#[test]
fn one_declaration_answers_to_whichever_interpretation_folds_it() {
    // The same declaration, and nothing in it prefers a language.
    assert_eq!(rust().compile_shape(rust().account_shape()).ty(), "Account");
    assert_eq!(
        TextShape.compile_shape(TextShape.account_shape()),
        "account { display_name: text, balance: decimal u128, checksum: hex bytes<32>, \
         ..timestamps { created_at: u64 } }",
    );
}
