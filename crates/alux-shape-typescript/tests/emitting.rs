//! What a shape reads as, in TypeScript.

use alux_shape::{FieldAlg, Shape, ShapeAlg, ShapeExt, ShapeOf, Spelling, Words};
use alux_shape_typescript::{TsShape, TsType};
use serde::Serialize;

const USER: Words<'static> = &["user"];
const ADDRESS: Words<'static> = &["address"];
const TIMESTAMPS: Words<'static> = &["timestamps"];
const CHECKSUM: Words<'static> = &["checksum"];
const DISPLAY_NAME: Words<'static> = &["display", "name"];

fn alg() -> TsShape {
    TsShape::new(Spelling::LowerCamel)
}

#[test]
fn a_product_is_an_interface_and_its_use_is_its_name() {
    let alg = alg();
    let user = alg
        .named_product(USER, vec![alg.field(CHECKSUM, alg.bytes_hex(Some(32))), alg.field(DISPLAY_NAME, alg.text())]);

    assert_eq!(user.expr(), "User");
    assert_eq!(
        user.module(),
        r"export interface User {
  checksum: string
  displayName: string
}",
    );
}

#[test]
fn a_merged_product_is_an_intersection() {
    let alg = alg();
    let stamps = alg.named_product(TIMESTAMPS, vec![alg.field(&["created", "at"], alg.int(false, 64))]);
    let address = alg.named_product(ADDRESS, vec![alg.field(&["city"], alg.text())]);
    let user = alg.named_product(USER, vec![alg.merge(stamps), alg.merge(address)]);

    assert_eq!(user.expr(), "User");
    assert_eq!(
        // Declarations come out in name order, which TypeScript is free to receive in any.
        user.module(),
        [
            r"export interface Address {
  city: string
}",
            r"export interface Timestamps {
  createdAt: number
}",
            "export type User = Timestamps & Address",
        ]
        .join("\n\n"),
    );
}

#[test]
fn a_name_is_declared_once_however_often_it_is_used() {
    let alg = alg();
    let checksum = || alg.named(CHECKSUM, alg.bytes_hex(Some(32)));
    let pair = alg.named_product(USER, vec![alg.field(&["avatar"], checksum()), alg.field(&["banner"], checksum())]);

    assert_eq!(
        pair.module(),
        [
            "export type Checksum = string",
            r"export interface User {
  avatar: Checksum
  banner: Checksum
}",
        ]
        .join("\n\n"),
    );
}

#[test]
fn a_choice_between_names_is_a_union_of_string_literals() {
    let alg = alg();
    let role = alg.named(&["role"], alg.sum_of_names(vec![&["admin"], &["member"]]));

    assert_eq!(role.module(), "export type Role = \"admin\" | \"member\"");
}

#[test]
fn an_optional_member_admits_null_and_a_sequence_of_a_union_is_grouped() {
    let alg = alg();
    let shape = alg.named_product(
        USER,
        vec![
            alg.field(&["email"], alg.opt(alg.text())),
            alg.field(&["either"], alg.seq(alg.choice(vec![alg.text(), alg.truth()]))),
        ],
    );

    assert_eq!(
        shape.module(),
        r"export interface User {
  email: string | null
  either: (string | boolean)[]
}",
    );
}

#[test]
fn bare_bytes_are_uninhabited_and_written_bytes_are_text() {
    let alg = alg();

    assert_eq!(alg.bytes(Some(32)).expr(), "never");
    assert_eq!(alg.bytes_hex(Some(32)).expr(), "string");
    assert_eq!(alg.bytes_array().expr(), "number[]");
}

#[test]
fn a_wide_quantity_written_as_text_reads_as_text() {
    let alg = alg();

    assert_eq!(alg.int(false, 128).expr(), "number");
    assert_eq!(alg.int_decimal(false, 128).expr(), "string");
}

#[derive(Serialize, Shape)]
#[serde(rename_all = "camelCase")]
struct Timestamps {
    created_at: u64,
    updated_at: u64,
}

#[derive(Serialize, Shape)]
#[serde(rename_all = "camelCase")]
struct User {
    display_name: String,
    email: Option<String>,
    #[serde(flatten)]
    stamps: Timestamps,
    #[serde(skip)]
    #[allow(dead_code)]
    secret: u64,
}

#[test]
fn a_layout_reads_as_the_declarations_a_client_would_write() {
    let module = <User as ShapeOf<TsShape>>::shape_of(&alg()).module();

    assert_eq!(
        module,
        [
            r"export interface Timestamps {
  createdAt: number
  updatedAt: number
}",
            "export type User = { displayName: string; email: string | null } & Timestamps",
        ]
        .join("\n\n"),
    );
}

#[test]
fn a_shape_used_at_a_site_states_only_its_name() {
    let shape: TsType = <User as ShapeOf<TsShape>>::shape_of(&alg());

    assert_eq!(shape.expr(), "User");
}
