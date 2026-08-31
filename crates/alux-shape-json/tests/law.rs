//! The law that keeps a derived shape honest.
//!
//! A shape read out of a layout is worth nothing unless it agrees with what that layout writes, so
//! every type here is serialized and then judged against its own term. A member forgotten by the
//! derive, a name segmented wrongly, or a writing the shape states and `serde` does not, all show up
//! as a value the shape does not describe.

use alux_shape::{Shape, ShapeOf, Spelling};
use alux_shape_json::{Judge, Judgement, Verdict};
use serde::Serialize;

/// A value's serialization is described by its shape.
fn law<T>(value: &T) -> Verdict
where
    T: Serialize + ShapeOf<Judge, Shape = Judgement>,
{
    let json = serde_json::to_value(value).expect("serializes");

    T::shape_of(&Judge::new(Spelling::LowerCamel)).holds(&json)
}

#[derive(Serialize, Shape)]
#[serde(rename_all = "camelCase")]
struct Timestamps {
    created_at: u64,
    updated_at: u64,
}

#[derive(Serialize, Shape)]
#[serde(rename_all = "camelCase")]
struct Tags {
    labels: Vec<String>,
}

#[derive(Serialize, Shape)]
#[serde(rename_all = "camelCase")]
struct Post {
    #[serde(flatten)]
    stamps: Timestamps,
    #[serde(flatten)]
    tags: Tags,
}

#[derive(Serialize, Shape)]
#[serde(rename_all = "camelCase")]
struct User {
    id: u64,
    display_name: String,
    email: Option<String>,
    #[serde(rename = "avatarBytes")]
    avatar: Vec<u8>,
    // Skipped, so the shape describes no member for it — and nothing reads it here either.
    #[serde(skip)]
    #[allow(dead_code)]
    secret: u64,
}

#[derive(Serialize, Shape)]
#[serde(rename_all = "camelCase")]
enum Role {
    Admin,
    Member,
}

#[derive(Serialize, Shape)]
#[serde(untagged)]
enum Handle {
    Id(u64),
    Name(String),
}

#[derive(Serialize, Shape)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum Event {
    Published(Timestamps),
}

#[derive(Serialize, Shape)]
#[serde(rename_all = "camelCase")]
struct Membership {
    role: Role,
    who: Handle,
}

#[test]
fn a_product_is_described_by_its_shape() {
    assert_eq!(law(&Timestamps { created_at: 1, updated_at: 2 }), Ok(()));
}

#[test]
fn a_merged_product_is_described_by_its_shape() {
    let post = Post { stamps: Timestamps { created_at: 1, updated_at: 2 }, tags: Tags { labels: vec!["rust".into()] } };

    assert_eq!(law(&post), Ok(()));
}

#[test]
fn a_renamed_member_is_named_by_the_words_its_rename_states() {
    let user = User { id: 7, display_name: "ada".into(), email: None, avatar: vec![1, 2, 3], secret: 9 };

    assert_eq!(law(&user), Ok(()));
}

#[test]
fn a_present_optional_member_is_described_too() {
    let user =
        User { id: 7, display_name: "ada".into(), email: Some("ada@example.com".into()), avatar: vec![], secret: 0 };

    assert_eq!(law(&user), Ok(()));
}

#[test]
fn a_choice_between_names_is_described_by_its_shape() {
    assert_eq!(law(&Role::Admin), Ok(()));
    assert_eq!(law(&Role::Member), Ok(()));
}

#[test]
fn an_untagged_choice_is_described_by_its_shape() {
    assert_eq!(law(&Handle::Id(7)), Ok(()));
    assert_eq!(law(&Handle::Name("ada".into())), Ok(()));
}

#[test]
fn an_internally_tagged_choice_is_described_by_its_shape() {
    assert_eq!(law(&Event::Published(Timestamps { created_at: 1, updated_at: 2 })), Ok(()));
}

#[test]
fn a_nested_choice_is_described_where_it_is_used() {
    assert_eq!(law(&Membership { role: Role::Member, who: Handle::Id(3) }), Ok(()));
}

#[test]
fn the_law_has_teeth() {
    // The same value, judged by a shape whose names are spelled another way, is not described.
    let stamps = Timestamps { created_at: 1, updated_at: 2 };
    let json = serde_json::to_value(&stamps).expect("serializes");
    let wrong = <Timestamps as ShapeOf<Judge>>::shape_of(&Judge::new(Spelling::Snake))
        .holds(&json)
        .expect_err("the value spells its names in camel case");

    assert_eq!(wrong.at, "created_at");
}

#[test]
fn a_member_the_shape_forgot_would_be_caught() {
    // A term stating one member where the layout writes two is exactly what the law rejects.
    let alg = Judge::new(Spelling::LowerCamel);
    let partial = {
        use alux_shape::{FieldAlg as _, ShapeAlg as _, ShapeExt as _};

        alg.named_product(&["timestamps"], vec![alg.field(&["created", "at"], alg.int(false, 64))])
    };
    let json = serde_json::to_value(Timestamps { created_at: 1, updated_at: 2 }).expect("serializes");

    assert!(partial.holds(&json).is_err());
}
