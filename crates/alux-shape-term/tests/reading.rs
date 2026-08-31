//! What a written term agrees with, and what reading one back adds.

use alux_shape::{FieldAlg, ShapeAlg, ShapeExt, Spelling, Words};
use alux_shape_json::Judge;
use alux_shape_term::{Term, TermShape};
use alux_shape_text::TextShape;
use serde_json::json;

const USER: Words<'static> = &["user"];
const CHECKSUM: Words<'static> = &["checksum"];
const DISPLAY_NAME: Words<'static> = &["display", "name"];
const ADDRESS: Words<'static> = &["address"];
const CITY: Words<'static> = &["city"];

/// One shape, stated once, so that every interpretation below folds the same thing.
fn user<A>(alg: &A) -> A::Ty
where
    A: ShapeAlg + FieldAlg,
{
    let checksum = alg.field(CHECKSUM, alg.bytes_hex(Some(2)));
    let display_name = alg.field(DISPLAY_NAME, alg.text());

    alg.named_product(USER, vec![checksum, display_name])
}

#[test]
fn reading_a_written_term_answers_what_interpreting_it_directly_does() {
    // The law: writing then folding is interpreting. Checked against two interpretations, since a
    // reader that agreed with only one would be agreeing with that interpretation and not the term.
    assert_eq!(user(&TermShape).fold(&TextShape), user(&TextShape));

    let value = json!({"checksum": "0xbeef", "displayName": "ada"});
    let judge = Judge::new(Spelling::LowerCamel);

    assert_eq!(user(&TermShape).fold(&judge).holds(&value), user(&judge).holds(&value));
}

#[test]
fn a_term_travels_as_a_value() {
    let written = user(&TermShape);
    let document = serde_json::to_string(&written).expect("a term serializes");
    let read: Term = serde_json::from_str(&document).expect("and reads back");

    assert_eq!(read, written);
    assert_eq!(read.fold(&TextShape), user(&TextShape));
}

#[test]
fn a_reference_describes_the_shape_its_name_introduces() {
    let alg = TermShape;
    let address = alg.named_product(ADDRESS, vec![alg.field(CITY, alg.text())]);
    let referring = alg.named_product(
        USER,
        vec![alg.field(ADDRESS, address), alg.field(&["billing", "address"], alg.reference(ADDRESS))],
    );

    // Unresolved, a reference describes anything, so a wrong value passes.
    let judge = Judge::new(Spelling::LowerCamel);
    let wrong = json!({"address": {"city": "vienna"}, "billingAddress": 7});

    assert!(referring.fold(&judge).holds(&wrong).is_ok());

    // Resolved, the name states what it introduced, and the same value does not.
    assert!(referring.resolved().fold(&judge).holds(&wrong).is_err());
    assert!(
        referring
            .resolved()
            .fold(&judge)
            .holds(&json!({
                "address": {"city": "vienna"},
                "billingAddress": {"city": "zagreb"},
            }))
            .is_ok()
    );
}

#[test]
fn a_name_that_would_resolve_into_itself_is_left_alone() {
    let alg = TermShape;
    // A shape whose member refers to the shape itself: expanding it never ends, so it stays a
    // reference and the resolution terminates.
    let recursive = alg.named_product(
        &["node"],
        vec![alg.field(DISPLAY_NAME, alg.text()), alg.field(&["parent"], alg.opt(alg.reference(&["node"])))],
    );

    let resolved = recursive.resolved();

    assert!(format!("{resolved:?}").contains("Reference"));
    assert_eq!(resolved.fold(&TextShape), recursive.fold(&TextShape));
}
