//! What a shape decides about a value.

use alux_shape::{FieldAlg, ShapeAlg, ShapeExt, ShapeTaggedExt, Spelling, Words};
use alux_shape_json::Judge;
use serde_json::json;

const USER: Words<'static> = &["user"];
const ID: Words<'static> = &["id"];
const DISPLAY_NAME: Words<'static> = &["display", "name"];
const CHECKSUM: Words<'static> = &["checksum"];
const BALANCE: Words<'static> = &["balance"];
const ROLE: Words<'static> = &["role"];
const ADMIN: Words<'static> = &["admin"];
const MEMBER: Words<'static> = &["member"];

fn judge() -> Judge {
    Judge::new(Spelling::LowerCamel)
}

#[test]
fn a_name_is_read_as_the_surface_spells_it() {
    let alg = judge();
    let shape = alg.named_product(USER, vec![alg.field(DISPLAY_NAME, alg.text())]);

    assert!(shape.holds(&json!({"displayName": "ada"})).is_ok());
    assert!(shape.holds(&json!({"display_name": "ada"})).is_err());
}

#[test]
fn a_described_member_must_be_present() {
    let alg = judge();
    let shape = alg.named_product(USER, vec![alg.field(ID, alg.int(false, 64))]);
    let missing = shape.holds(&json!({})).expect_err("a member is described");

    assert_eq!(missing.at, "id");
    assert_eq!(missing.expected, "a member that is present");
}

#[test]
fn a_member_the_shape_does_not_describe_is_a_disagreement() {
    let alg = judge();
    let shape = alg.named_product(USER, vec![alg.field(ID, alg.int(false, 64))]);
    let extra = shape.holds(&json!({"id": 7, "surprise": true})).expect_err("undescribed");

    assert_eq!(extra.at, "surprise");
}

#[test]
fn an_optional_member_may_be_absent_or_null() {
    let alg = judge();
    let shape = alg.named_product(USER, vec![alg.field(ID, alg.opt(alg.int(false, 64)))]);

    assert!(shape.holds(&json!({})).is_ok());
    assert!(shape.holds(&json!({"id": null})).is_ok());
    assert!(shape.holds(&json!({"id": 7})).is_ok());
    assert!(shape.holds(&json!({"id": "seven"})).is_err());
}

#[test]
fn hexadecimal_bytes_carry_the_length_the_shape_states() {
    let alg = judge();
    let shape = alg.bytes_hex(Some(2));

    assert!(shape.holds(&json!("0xbeef")).is_ok());
    assert!(shape.holds(&json!("0xbe")).is_err());
    assert!(shape.holds(&json!("beef")).is_err());
    assert!(shape.holds(&json!("0xzz")).is_err());
}

#[test]
fn bare_bytes_describe_no_value_until_their_writing_is_stated() {
    let alg = judge();

    assert!(alg.bytes(Some(2)).holds(&json!("0xbeef")).is_err());
    assert!(alg.hex(alg.bytes(Some(2))).holds(&json!("0xbeef")).is_ok());
}

#[test]
fn a_decimal_integer_is_written_as_text_and_not_as_a_number() {
    let alg = judge();
    let shape = alg.int_decimal(false, 128);

    assert!(shape.holds(&json!("340282366920938463463374607431768211455")).is_ok());
    assert!(shape.holds(&json!(1_000)).is_err());
}

#[test]
fn merging_two_products_reads_one_object() {
    let alg = judge();
    let stamps = alg.named_product(&["timestamps"], vec![alg.field(&["created", "at"], alg.int(false, 64))]);
    let account = alg.named_product(&["account"], vec![alg.field(BALANCE, alg.int(false, 64))]);
    let user = alg.named_product(USER, vec![alg.merge(stamps), alg.merge(account)]);

    assert!(user.holds(&json!({"createdAt": 1, "balance": 10})).is_ok());
    // Merged, not nested: the members belong to this object.
    assert!(user.holds(&json!({"timestamps": {"createdAt": 1}})).is_err());
}

#[test]
fn merging_something_that_is_not_a_product_describes_nothing() {
    let alg = judge();
    let shape = alg.named_product(USER, vec![alg.merge(alg.text())]);

    assert!(shape.holds(&json!({})).is_err());
}

#[test]
fn a_choice_between_names_reads_one_of_them() {
    let alg = judge();
    let shape = alg.sum_of_names(vec![ADMIN, MEMBER]);

    assert!(shape.holds(&json!("admin")).is_ok());
    assert!(shape.holds(&json!("member")).is_ok());
    assert!(shape.holds(&json!("other")).is_err());
}

#[test]
fn an_internally_tagged_choice_reads_its_discriminant_beside_its_members() {
    let alg = judge();
    let admin = alg.named_product(ADMIN, vec![alg.field(ID, alg.int(false, 64))]);
    let shape = alg.sum_internal(ROLE, vec![(ADMIN, admin)]);

    assert!(shape.holds(&json!({"role": "admin", "id": 7})).is_ok());
    assert!(shape.holds(&json!({"role": "member", "id": 7})).is_err());
}

#[test]
fn an_untagged_choice_reads_whichever_alternative_fits() {
    let alg = judge();
    let shape = alg.sum_untagged(vec![(ID, alg.int(false, 64)), (DISPLAY_NAME, alg.text())]);

    assert!(shape.holds(&json!(7)).is_ok());
    assert!(shape.holds(&json!("ada")).is_ok());
    assert!(shape.holds(&json!(true)).is_err());
}

#[test]
fn a_sequence_names_where_it_disagreed() {
    let alg = judge();
    let shape = alg.named_product(USER, vec![alg.field(CHECKSUM, alg.seq(alg.bytes_hex(Some(2))))]);
    let wrong = shape.holds(&json!({"checksum": ["0xbeef", "0xbe"]})).expect_err("second is short");

    assert_eq!(wrong.at, "checksum.1");
}
