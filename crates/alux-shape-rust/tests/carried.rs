//! A member stating what carries it, beside how it is written.
//!
//! One statement holds three facts: the member's name, the shape it is written as, and the type a
//! layout keeps it in. The third is read by an expansion and by nothing else — a Rust path means
//! nothing to a wire, a client, or a judgement — so at run time it is erased, and the tests below
//! pin that erasure. What a runtime layout interpretation knows about types it learns from names.

use alux_shape::{FieldAlg, Shape, ShapeAlg, ShapeDeclareExt, ShapeProgramAlg, ShapeProgramExt, Sorts, Spelling};
use alux_shape_rust::{Decimal, RustShape};
use alux_shape_text::TextShape;
use serde::Serialize;
use std::marker::PhantomData;

/// A layout with a shape of its own, so a member carrying one needs no shape stated for it.
#[derive(Debug, Clone, Serialize, Shape)]
#[serde(rename_all = "camelCase")]
struct Stamp {
    created_at: u64,
}

/// What the lowering of a declaration emits. `Carrier` stands for whatever type the declaration
/// named, so the same program can be built twice with different carriers and compared.
#[derive(Default)]
struct SummaryShapeProgram<Carrier>(PhantomData<Carrier>);

impl<This, Carrier> ShapeProgramAlg<This> for SummaryShapeProgram<Carrier>
where
    This: ShapeAlg + FieldAlg,
{
    type Ty = <This as Sorts>::Ty;

    fn compile_shape(self, alg: &This) -> Self::Ty {
        let _ = self;
        let builder = alg;
        let record = {
            builder
                .record(&["summary"])
                // `field::<Carrier>(number, fringe_number())` lowers to this: a carrier for the
                // expansion, a shape for everyone.
                .field_as::<Carrier>(&["number"], builder.named(&["fringe", "number"], builder.int(false, 64)))
                // `field::<Stamp>(stamp)` lowers to this: the type states the shape, so none is given.
                .field_of::<Stamp>(&["stamp"])
        };

        record.into_shape()
    }
}

#[test]
fn a_carrier_is_erased_by_every_interpretation() {
    // Two declarations differing only in what carries a member state one shape. Nothing at run time
    // can tell them apart, which is what it means for the carrier to be the expansion's business.
    let text = TextShape.compile_shape(SummaryShapeProgram::<u64>::default());

    assert_eq!(text, TextShape.compile_shape(SummaryShapeProgram::<Decimal<u128>>::default()));
    assert_eq!(text, "summary { number: fringe_number u64, stamp: stamp { created_at: u64 } }");
}

#[test]
fn a_runtime_layout_learns_its_types_from_names() {
    // With the carrier erased, a layout interpretation reads the name: one it has a type for it
    // writes as that type, and one it does not it declares.
    let rust = RustShape::new(Spelling::LowerCamel).known(&["fringe", "number"], "model::FringeNumber");
    let module = rust.compile_shape(SummaryShapeProgram::<u64>::default()).module();

    assert!(module.contains("    pub number: model::FringeNumber,"));
    assert!(!module.contains("pub type FringeNumber"));

    let unknown = RustShape::new(Spelling::LowerCamel);
    let declared = unknown.compile_shape(SummaryShapeProgram::<u64>::default()).module();

    assert!(declared.contains("pub type FringeNumber = u64;"));
    assert!(declared.contains("    pub number: FringeNumber,"));
}

#[test]
fn a_type_stating_its_own_shape_needs_none_stated_for_it() {
    // `field_of` reads the shape from the type, which is available exactly when the type says what it
    // is. An alias to a primitive says what the primitive says, so a leaf wanting a name of its own
    // states its shape instead.
    let text = TextShape.compile_shape(SummaryShapeProgram::<u64>::default());

    assert!(text.contains("stamp: stamp { created_at: u64 }"));
}
