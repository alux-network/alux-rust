//! A declaration and a function, stating one shape.
//!
//! The backend in `alux-ext-macros` lowers a declaration into a program type whose `compile_shape`
//! runs the authored body against the algebra. The programs below are written by hand in the form that
//! backend emits — a block bound to a name, the closing call after it, `let builder = alg;`, names
//! inline — so the contract is exercised before the macro is available.
//!
//! What the law compares is the folded shape and not the text that produced it. A transcription and an
//! expansion may differ in ways no interpretation can observe, so when the macro's own output goes in
//! beside these, the assertion worth making is that every authoring form folds to one shape.

use alux_shape::{FieldAlg, ShapeAlg, ShapeDeclareExt, ShapeExt, ShapeProgramAlg, ShapeProgramExt, Sorts, Words};
use alux_shape_text::TextShape;
use std::marker::PhantomData;

/// What `fn user_shape(&self)` lowers to.
#[derive(Default)]
struct UserShapeProgram(PhantomData<()>);

impl<This> ShapeProgramAlg<This> for UserShapeProgram
where
    This: ShapeAlg + FieldAlg,
{
    type Ty = <This as Sorts>::Ty;

    fn compile_shape(self, alg: &This) -> Self::Ty {
        let _ = self;
        let builder = alg;
        let record = {
            builder
                .record(&["user"])
                .field(&["display", "name"], builder.text())
                .field(&["email"], builder.opt(builder.text()))
        };

        record.into_shape()
    }
}

/// What `fn timestamps_shape(&self)` lowers to.
#[derive(Default)]
struct TimestampsShapeProgram(PhantomData<()>);

impl<This> ShapeProgramAlg<This> for TimestampsShapeProgram
where
    This: ShapeAlg + FieldAlg,
{
    type Ty = <This as Sorts>::Ty;

    fn compile_shape(self, alg: &This) -> Self::Ty {
        let _ = self;
        let builder = alg;
        let record = { builder.record(&["timestamps"]).field(&["created", "at"], builder.int(false, 64)) };

        record.into_shape()
    }
}

/// What `fn stored_user_shape(&self)` lowers to, nesting the two above.
#[derive(Default)]
struct StoredUserShapeProgram(PhantomData<()>);

impl<This> ShapeProgramAlg<This> for StoredUserShapeProgram
where
    This: ShapeAlg + FieldAlg,
    UserShapeProgram: ShapeProgramAlg<This, Ty = <This as Sorts>::Ty>,
    TimestampsShapeProgram: ShapeProgramAlg<This, Ty = <This as Sorts>::Ty>,
{
    type Ty = <This as Sorts>::Ty;

    fn compile_shape(self, alg: &This) -> Self::Ty {
        let _ = self;
        let builder = alg;
        // The backend writes `builder.program(builder.user_shape())` here, reaching the nested shape
        // through the method the declaration was written as — which is what the `_shape` suffix is
        // for, and which a file with no `#[ext]` block cannot say. The value is the same one, so the
        // folded shape is the same shape.
        let record = {
            builder
                .record(&["stored", "user"])
                .merge(builder.program(UserShapeProgram::default()))
                .merge(builder.program(TimestampsShapeProgram::default()))
        };

        record.into_shape()
    }
}

const USER: Words<'static> = &["user"];
const STORED_USER: Words<'static> = &["stored", "user"];
const TIMESTAMPS: Words<'static> = &["timestamps"];
const DISPLAY_NAME: Words<'static> = &["display", "name"];
const EMAIL: Words<'static> = &["email"];
const CREATED_AT: Words<'static> = &["created", "at"];

/// The same user shape, stated as a function.
fn user<A>(alg: &A) -> A::Ty
where
    A: ShapeAlg + FieldAlg,
{
    let display_name = alg.field(DISPLAY_NAME, alg.text());
    let email = alg.field(EMAIL, alg.opt(alg.text()));

    alg.named_product(USER, vec![display_name, email])
}

/// The same timestamps shape, stated as a function.
fn timestamps<A>(alg: &A) -> A::Ty
where
    A: ShapeAlg + FieldAlg,
{
    alg.named_product(TIMESTAMPS, vec![alg.field(CREATED_AT, alg.int(false, 64))])
}

/// The same stored shape, stated as a function.
fn stored_user<A>(alg: &A) -> A::Ty
where
    A: ShapeAlg + FieldAlg,
{
    let members = vec![alg.merge(user(alg)), alg.merge(timestamps(alg))];

    alg.named_product(STORED_USER, members)
}

#[test]
fn every_authoring_form_folds_to_one_shape() {
    // The law, and the only assertion the law needs: what a declaration states and what a function
    // states are the same shape. Whatever text produced either is below this.
    assert_eq!(TextShape.compile_shape(UserShapeProgram::default()), user(&TextShape));
    assert_eq!(TextShape.compile_shape(TimestampsShapeProgram::default()), timestamps(&TextShape));
    assert_eq!(TextShape.compile_shape(StoredUserShapeProgram::default()), stored_user(&TextShape));
}

#[test]
fn a_declaration_takes_its_name_from_the_declaration() {
    // The name reaches the term from the method the declaration was written as, so it is in the shape
    // rather than in anything the body said.
    assert_eq!(TextShape.compile_shape(UserShapeProgram::default()), "user { display_name: text, email: text? }",);
}

#[test]
fn a_nested_declaration_is_merged_and_not_nested() {
    assert_eq!(
        TextShape.compile_shape(StoredUserShapeProgram::default()),
        "stored_user { ..user { display_name: text, email: text? }, \
         ..timestamps { created_at: u64 } }",
    );
}
