//! What a term reads as, in Rust.

use alux_shape::{FieldAlg, Shape, ShapeAlg, ShapeExt, ShapeOf, Spelling, Words};
use alux_shape_rust::RustShape;
use alux_shape_text::TextShape;
use serde::Serialize;

const USER: Words<'static> = &["user"];
const TIMESTAMPS: Words<'static> = &["timestamps"];
const DISPLAY_NAME: Words<'static> = &["display", "name"];
const CHECKSUM: Words<'static> = &["checksum"];

fn alg() -> RustShape {
    RustShape::new(Spelling::LowerCamel)
}

#[test]
fn a_product_is_a_struct_whose_fields_rust_names_its_own_way() {
    let alg = alg();
    let user =
        alg.named_product(USER, vec![alg.field(DISPLAY_NAME, alg.text()), alg.field(&["email"], alg.opt(alg.text()))]);

    assert_eq!(user.ty(), "User");
    assert_eq!(
        user.module(),
        [
            "#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]",
            "#[serde(rename_all = \"camelCase\")]",
            "pub struct User {",
            "    pub display_name: String,",
            "    pub email: Option<String>,",
            "}",
        ]
        .join("\n"),
    );
}

#[test]
fn a_surface_that_writes_names_as_rust_does_states_no_attribute() {
    let alg = RustShape::new(Spelling::Snake);
    let user = alg.named_product(USER, vec![alg.field(DISPLAY_NAME, alg.text())]);

    assert!(!user.module().contains("rename_all"));
}

#[test]
fn a_writing_is_a_wrapper_so_the_layout_keeps_the_value() {
    let alg = alg();

    assert_eq!(alg.bytes(Some(32)).ty(), "[u8; 32]");
    assert_eq!(alg.bytes_hex(Some(32)).ty(), "Hex<[u8; 32]>");
    assert_eq!(alg.int_decimal(false, 128).ty(), "Decimal<u128>");
    assert_eq!(alg.bytes_array().ty(), "Vec<u8>");
}

#[test]
fn a_merged_product_is_a_field_that_carries_the_members_it_merges() {
    let alg = alg();
    let stamps = alg.named_product(TIMESTAMPS, vec![alg.field(&["created", "at"], alg.int(false, 64))]);
    let user = alg.named_product(USER, vec![alg.field(DISPLAY_NAME, alg.text()), alg.merge(stamps)]);

    assert_eq!(
        user.module(),
        [
            "#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]",
            "#[serde(rename_all = \"camelCase\")]",
            "pub struct Timestamps {",
            "    pub created_at: u64,",
            "}",
            "",
            "#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]",
            "#[serde(rename_all = \"camelCase\")]",
            "pub struct User {",
            "    pub display_name: String,",
            "    #[serde(flatten)]",
            "    pub timestamps: Timestamps,",
            "}",
        ]
        .join("\n"),
    );
}

#[test]
fn a_named_leaf_is_an_alias() {
    let alg = alg();
    let checksum = alg.named(CHECKSUM, alg.bytes_hex(Some(32)));

    assert_eq!(checksum.ty(), "Checksum");
    assert_eq!(checksum.module(), "pub type Checksum = Hex<[u8; 32]>;");
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
    #[serde(flatten)]
    stamps: Timestamps,
}

#[test]
fn a_layout_read_from_a_term_states_the_layout_it_was_read_from() {
    // The derive reads a term out of this layout; the interpretation reads a layout back out of the
    // term. What comes back is the same declaration, written the way this interpretation writes one.
    let module = <User as ShapeOf<RustShape>>::shape_of(&alg()).module();

    assert!(module.contains("pub struct User {"));
    assert!(module.contains("    pub display_name: String,"));
    assert!(module.contains(
        r"    #[serde(flatten)]
    pub timestamps: Timestamps,"
    ));
    assert!(module.contains("pub struct Timestamps {"));
    assert!(module.contains("    pub created_at: u64,"));
}

/// One shape, stated by a layout, folded by whichever interpretation is handed to it.
fn user_shape<A>(alg: &A) -> <User as ShapeOf<A>>::Shape
where
    User: ShapeOf<A>,
{
    <User as ShapeOf<A>>::shape_of(alg)
}

#[test]
fn one_term_reads_as_a_layout_and_as_a_description_of_the_same_shape() {
    // Two interpretations of one term, neither privileged.
    assert_eq!(user_shape(&alg()).ty(), "User");
    assert_eq!(
        user_shape(&TextShape),
        "user { display_name: text, ..timestamps { created_at: u64, updated_at: u64 } }",
    );
}
