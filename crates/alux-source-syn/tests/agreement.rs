//! The two readings of a layout agree: the `Shape` derive, asking the compiler at build time, and
//! the source reader, reading this very file as text, state the same shape for each type below.

use alux_shape::{FieldAlg, Shape, ShapeAlg, ShapeOf};
use alux_shape_term::{Term, TermShape};
use alux_source_index::{Declaration, Index};
use alux_source_syn::ReadSourceExt;

/// A product: renamed, skipped and written members beside plain ones.
#[allow(dead_code, reason = "only its shape is read")]
#[derive(Shape)]
struct UserProfile {
    display_name: String,
    #[serde(rename = "emailAddress")]
    email: Option<String>,
    tags: Vec<String>,
    scores: std::collections::BTreeMap<String, u32>,
    #[written(hex(32))]
    checksum: Vec<u8>,
    #[serde(skip)]
    secret: u64,
}

/// A choice of names alone.
#[allow(dead_code, reason = "only its shape is read")]
#[derive(Shape)]
enum Role {
    Admin,
    Member,
}

/// A choice carrying shapes, tagged internally.
#[allow(dead_code, reason = "only its shape is read")]
#[derive(Shape)]
#[serde(tag = "kind")]
enum Event {
    Joined(String),
    Left(u64),
}

/// A tuple struct, an array of its positions.
#[allow(dead_code, reason = "only its shape is read")]
#[derive(Shape)]
struct Pair(u32, Option<String>);

/// A newtype, its item.
#[allow(dead_code, reason = "only its shape is read")]
#[derive(Shape)]
struct Id(u64);

/// A unit struct, unit.
#[derive(Shape)]
struct Marker;

/// A product holding a Rust tuple.
#[allow(dead_code, reason = "only its shape is read")]
#[derive(Shape)]
struct Placed {
    at: (i32, i32),
}

/// The shape the reader states for the data type `name` in this file.
fn read(name: &str) -> Term {
    let declared = Index.read_source("agreement.rs", include_str!("agreement.rs")).unwrap();
    declared
        .into_iter()
        .find_map(|declared| match declared.declaration {
            Declaration::Data { name: read, shape, .. } if read == name => Some(shape),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no data named {name}"))
}

/// Each type's shape, read both ways, is one term.
#[test]
fn the_derive_and_the_source_reader_state_the_same_shapes() {
    assert_eq!(read("UserProfile"), UserProfile::shape_of(&TermShape));
    assert_eq!(read("Role"), Role::shape_of(&TermShape));
    assert_eq!(read("Event"), Event::shape_of(&TermShape));
    assert_eq!(read("Pair"), Pair::shape_of(&TermShape));
    assert_eq!(read("Id"), Id::shape_of(&TermShape));
    assert_eq!(read("Marker"), Marker::shape_of(&TermShape));
    assert_eq!(read("Placed"), Placed::shape_of(&TermShape));
}

/// A member whose type is another declared type is a reference to its name: the reader resolves
/// nothing, where the derive inlines that type's shape.
#[test]
fn another_declared_type_is_referred_to_by_name() {
    #[allow(dead_code, reason = "only its shape is read")]
    struct Holder {
        role: Role,
    }
    let declared = Index.read_source("holder.rs", "struct Holder { role: Role }").unwrap();
    let Declaration::Data { shape, .. } = &declared[0].declaration else { panic!("{declared:?}") };
    let expected = TermShape
        .named(&["holder"], TermShape.product(vec![TermShape.field(&["role"], TermShape.reference(&["role"]))]));
    assert_eq!(*shape, expected);
}
