//! One shape, folded through a transformation.

use alux_shape::{FieldAlg, ShapeAlg, ShapeExt, Spelling, Words};
use alux_shape_json::Judge;
use alux_shape_morph::{Patch, Prefixed};
use alux_shape_text::TextShape;
use alux_shape_typescript::TsShape;
use serde_json::json;

const USER: Words<'static> = &["user"];
const DISPLAY_NAME: Words<'static> = &["display", "name"];
const EMAIL: Words<'static> = &["email"];

/// One shape, stated once, folded below by whichever algebra it is handed.
fn user<A>(alg: &A) -> A::Ty
where
    A: ShapeAlg + FieldAlg,
{
    let display_name = alg.field(DISPLAY_NAME, alg.text());
    let email = alg.field(EMAIL, alg.text());

    alg.named_product(USER, vec![display_name, email])
}

#[test]
fn a_transformation_answers_beside_the_shape_it_transformed() {
    // The same term, folded twice: once as stated, once through a transformation. Both answer,
    // and they can be told apart because the name moved with the shape.
    assert_eq!(user(&TextShape), "user { display_name: text, email: text }");
    assert_eq!(user(&Patch(TextShape)), "user_patch { display_name: text?, email: text? }",);
}

#[test]
fn a_patch_requires_nothing_where_the_shape_required_everything() {
    let judge = Judge::new(Spelling::LowerCamel);
    let whole = user(&judge);
    let patch = user(&Patch(judge));
    let partial = json!({"displayName": "ada"});

    assert!(whole.holds(&partial).is_err());
    assert!(patch.holds(&partial).is_ok());
    assert!(patch.holds(&json!({})).is_ok());
    // What it does not do is admit a member the shape never described.
    assert!(patch.holds(&json!({"surprise": true})).is_err());
    // Nor a member of the wrong shape, merely because it may be absent.
    assert!(patch.holds(&json!({"displayName": 7})).is_err());
}

#[test]
fn both_declarations_come_out_of_one_statement() {
    let alg = TsShape::new(Spelling::LowerCamel);

    assert_eq!(
        user(&alg).module(),
        r"export interface User {
  displayName: string
  email: string
}",
    );
    assert_eq!(
        user(&Patch(alg)).module(),
        r"export interface UserPatch {
  displayName: string | null
  email: string | null
}",
    );
}

#[test]
fn a_prefix_keeps_two_surfaces_declarations_apart() {
    let alg = TsShape::new(Spelling::LowerCamel);

    assert_eq!(user(&Prefixed::new(alg, "admin")).expr(), "AdminUser");
    assert_eq!(user(&Prefixed::new(alg, "public")).expr(), "PublicUser");
}

#[test]
fn transformations_compose_because_each_is_only_another_algebra() {
    let alg = TsShape::new(Spelling::LowerCamel);
    let both = user(&Prefixed::new(Patch(alg), "admin"));

    // Read outward: the patch names the shape, and the prefix names what the patch answered under.
    assert_eq!(both.expr(), "AdminUserPatch");
    assert_eq!(
        both.module(),
        r"export interface AdminUserPatch {
  displayName: string | null
  email: string | null
}",
    );
}
