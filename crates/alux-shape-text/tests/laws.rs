//! The derived operations, checked against the primitives they are stated in terms of.
//!
//! Every assertion here is a law of the extension rather than a property of this interpretation: two
//! terms that denote one shape render one text. Checking them against the witness is enough, because
//! the extension composes primitives and cannot observe which interpretation is folding it.

use alux_shape::{FieldAlg, ShapeAlg, ShapeExt, ShapeTaggedExt, Words};
use alux_shape_text::TextShape;

const USER: Words<'static> = &["user"];
const ID: Words<'static> = &["id"];
const DISPLAY_NAME: Words<'static> = &["display", "name"];
const BALANCE: Words<'static> = &["balance"];
const CHECKSUM: Words<'static> = &["checksum"];
const KIND: Words<'static> = &["kind"];
const VALUE: Words<'static> = &["value"];
const ADMIN: Words<'static> = &["admin"];
const MEMBER: Words<'static> = &["member"];

#[test]
fn hexadecimal_bytes_are_bytes_written_as_hexadecimal() {
    let alg = TextShape;

    assert_eq!(alg.bytes_hex(Some(32)), alg.hex(alg.bytes(Some(32))));
}

#[test]
fn a_byte_array_is_a_sequence_of_octets() {
    let alg = TextShape;

    assert_eq!(alg.bytes_array(), alg.seq(alg.int(false, 8)));
}

#[test]
fn a_decimal_integer_is_an_integer_written_as_decimal() {
    let alg = TextShape;

    assert_eq!(alg.int_decimal(false, 128), alg.decimal(alg.int(false, 128)));
    assert_eq!(alg.int_hex(true, 64), alg.hex(alg.int(true, 64)));
}

#[test]
fn a_named_product_is_a_product_given_a_name() {
    let alg = TextShape;
    let members = || vec![alg.field(BALANCE, alg.int_decimal(false, 128))];

    assert_eq!(alg.named_product(USER, members()), alg.named(USER, alg.product(members())));
}

#[test]
fn an_untagged_choice_keeps_no_discriminant() {
    let alg = TextShape;
    let alternatives = || vec![(ID, alg.int(false, 64)), (DISPLAY_NAME, alg.text())];

    assert_eq!(
        alg.sum_untagged(alternatives()),
        alg.choice(alternatives().into_iter().map(|(_, shape)| shape).collect()),
    );
}

#[test]
fn a_choice_between_names_is_a_choice_between_name_valued_constants() {
    let alg = TextShape;

    assert_eq!(alg.sum_of_names(vec![ADMIN, MEMBER]), alg.choice(vec![alg.name_word(ADMIN), alg.name_word(MEMBER)]),);
}

#[test]
fn an_externally_tagged_choice_is_a_choice_of_single_member_products() {
    let alg = TextShape;

    assert_eq!(
        alg.sum_external(vec![(ADMIN, alg.text())]),
        alg.choice(vec![alg.product(vec![alg.field(ADMIN, alg.text())])]),
    );
}

#[test]
fn an_internally_tagged_choice_carries_its_name_beside_its_own_members() {
    let alg = TextShape;

    assert_eq!(
        alg.sum_internal(KIND, vec![(ADMIN, alg.text())]),
        alg.choice(vec![alg.product(vec![alg.field(KIND, alg.name_word(ADMIN)), alg.merge(alg.text()),])]),
    );
}

#[test]
fn an_adjacently_tagged_choice_carries_its_name_and_its_value_apart() {
    let alg = TextShape;

    assert_eq!(
        alg.sum_adjacent(KIND, VALUE, vec![(ADMIN, alg.text())]),
        alg.choice(vec![alg.product(vec![alg.field(KIND, alg.name_word(ADMIN)), alg.field(VALUE, alg.text()),])]),
    );
}

#[test]
fn a_term_renders_the_words_its_names_are_made_of() {
    let alg = TextShape;
    let user = alg.named_product(
        USER,
        vec![
            alg.field(CHECKSUM, alg.bytes_hex(Some(32))),
            alg.field(DISPLAY_NAME, alg.text()),
            alg.field(BALANCE, alg.int_decimal(false, 128)),
        ],
    );

    assert_eq!(user, "user { checksum: hex bytes<32>, display_name: text, balance: decimal u128 }",);
}

#[test]
fn merging_two_products_is_not_nesting_them() {
    let alg = TextShape;
    let stamps = alg.named_product(&["timestamps"], vec![alg.field(&["created", "at"], alg.int(false, 64))]);
    let account = alg.named_product(&["account"], vec![alg.field(BALANCE, alg.int(false, 64))]);
    let user = alg.named_product(USER, vec![alg.merge(stamps), alg.merge(account)]);

    assert_eq!(user, "user { ..timestamps { created_at: u64 }, ..account { balance: u64 } }",);
}
