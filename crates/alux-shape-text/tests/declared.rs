//! A shape stated as a declaration, and the same shape stated as a function.
//!
//! Experimental. The declaration form is exercised here rather than in a domain: what a surface states
//! today is a layout with `#[derive(Shape)]`, and this is where the other direction is kept honest.
//!
//! A declaration names its members with identifiers and takes its own name from the method it is
//! written as. What it states is a shape, so it folds like any other.

use alux_ext::ext;
use alux_ext::macros::shape;
use alux_shape::{FieldAlg, ShapeAlg, ShapeDeclareExt, ShapeExt, ShapeProgramExt, Sorts};
use alux_shape_text::TextShape;

/// A domain names its leaves once, and its declarations read as declarations.
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

#[ext(name = UserShapeExt, defunc(via = shape))]
pub impl<This> This
where
    This: ShapeAlg + FieldAlg,
{
    /// A user, as a surface answers one.
    fn user_shape(&self) {
        self.record()
            .field(id, self.int(false, 64))
            .field(display_name, self.text())
            .field(balance, self.amount())
            .field(checksum, self.checksum())
    }

    /// When a record was made.
    fn timestamps_shape(&self) {
        self.record().field(created_at, self.int(false, 64))
    }

    /// A user as it is stored: the user and its stamps, observed as one.
    fn stored_user_shape(&self) {
        self.record().merge(self.user_shape()).merge(self.timestamps_shape())
    }
}

#[test]
fn a_declaration_states_a_shape() {
    let text = TextShape.compile_shape(TextShape.user_shape());

    assert_eq!(text, "user { id: u64, display_name: text, balance: decimal u128, checksum: hex bytes<32> }",);
}

#[test]
fn a_nested_declaration_merges_what_it_names() {
    let text = TextShape.compile_shape(TextShape.stored_user_shape());

    assert_eq!(
        text,
        "stored_user { ..user { id: u64, display_name: text, balance: decimal u128, \
         checksum: hex bytes<32> }, ..timestamps { created_at: u64 } }",
    );
}

/// The layout a declaration states, emitted beside it.
///
/// `#[shape_layout]` reads each member's name and the type stating it, so a declaration is the only
/// thing written. It belongs in the `shape` backend; until it is there, it is an attribute of its own.
#[alux_shape::shape_layout]
#[ext(name = AccountShapeExt, defunc(via = shape))]
pub impl<This> This
where
    This: ShapeAlg + FieldAlg,
{
    /// An account, as a surface answers one.
    fn account_shape(&self) {
        self.record().field::<String>(display_name, self.text()).field::<u64>(created_at, self.int(false, 64))
    }
}

#[test]
fn a_declaration_states_its_layout_as_well() {
    // The layout is emitted from the declaration, and states the shape the declaration states.
    let account = Account { display_name: "ada".into(), created_at: 1 };

    assert_eq!(TextShape.compile_shape(TextShape.account_shape()), "account { display_name: text, created_at: u64 }",);
    assert_eq!(account.display_name, "ada");
    assert_eq!(
        <Account as alux_shape::ShapeOf<TextShape>>::shape_of(&TextShape),
        "account { display_name: text, created_at: u64 }",
    );
}
