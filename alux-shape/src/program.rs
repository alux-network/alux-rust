//! A shape stated as a declaration.
//!
//! A declaration becomes a program: a value that answers with a shape once an algebra is handed to
//! it. The backend that lowers one rewrites every `self` in the body to the algebra, so a declaration
//! reaches its primitives and its members through the same receiver — which is why the operations
//! below are extensions over the algebra rather than methods on a builder of their own.

use crate::{FieldAlg, ShapeAlg, ShapeExt, Sorts, Words};
use alux_ext::ext;

/// States the shape a declaration denotes, once an algebra interprets it.
pub trait ShapeProgramAlg<Alg> {
    /// The shape the algebra builds.
    type Ty;

    /// Folds this declaration with the algebra.
    fn compile_shape(self, alg: &Alg) -> Self::Ty;
}

/// Compiles a shape declaration with an interpretation.
#[ext(name = ShapeProgramExt, supertraits = Sized)]
pub impl<This> This {
    /// Folds a declaration with this interpretation.
    fn compile_shape<Program>(&self, program: Program) -> Program::Ty
    where
        Program: ShapeProgramAlg<This>,
    {
        program.compile_shape(self)
    }
}

/// The operations a declaration's body states.
#[ext(name = ShapeDeclareExt, supertraits = Sorts + Sized)]
pub impl<This> This
where
    This: ShapeAlg + FieldAlg,
{
    /// Opens the product a declaration states, under the name the declaration carries.
    fn record<'a>(&'a self, words: Words<'a>) -> Record<'a, Self> {
        Record { alg: self, words, members: Vec::new() }
    }

    /// The shape another declaration states, folded here.
    fn program<Program>(&self, program: Program) -> Self::Ty
    where
        Program: ShapeProgramAlg<Self, Ty = Self::Ty>,
    {
        program.compile_shape(self)
    }
}

/// A product being stated, holding the algebra it is stated against.
///
/// Not part of this specification: it exists between what a backend reads and what it writes, so it
/// needs no sort, no primitive, and no mention in the algebra. Members are built as they arrive,
/// since a declaration has the algebra in hand from its first call.
pub struct Record<'a, A>
where
    A: Sorts,
{
    alg: &'a A,
    words: Words<'a>,
    members: Vec<A::Field>,
}

impl<'a, A> Record<'a, A>
where
    A: ShapeAlg + FieldAlg,
{
    /// States one member, under the name it is written with.
    #[must_use]
    pub fn field(mut self, words: Words<'_>, shape: A::Ty) -> Self {
        self.members.push(self.alg.field(words, shape));

        self
    }

    /// States one member, and the type a layout carries it in.
    ///
    /// The type is the layout's business and not this one's: nothing here reads it, and an expansion
    /// emits a field of that type. A declaration writes `field::<Carrier>(name, shape)`, which lowers
    /// to this.
    #[must_use]
    pub fn field_as<Carrier>(mut self, words: Words<'_>, shape: A::Ty) -> Self {
        self.members.push(self.alg.field(words, shape));

        self
    }

    /// States one member whose shape its own type states.
    ///
    /// Available where a type says what it is — a layout with a shape of its own, or a leaf whose
    /// domain states one. A name that is only an alias states what the type it names states, so a
    /// member wanting a name of its own gives its shape instead.
    #[must_use]
    pub fn field_of<Carrier>(mut self, words: Words<'_>) -> Self
    where
        Carrier: crate::ShapeOf<A, Shape = A::Ty>,
    {
        let shape = Carrier::shape_of(self.alg);
        self.members.push(self.alg.field(words, shape));

        self
    }

    /// States another product's members as this one's own.
    #[must_use]
    pub fn merge(mut self, shape: A::Ty) -> Self {
        self.members.push(self.alg.merge(shape));

        self
    }

    /// Answers with the shape this product states.
    #[must_use]
    pub fn into_shape(self) -> A::Ty {
        self.alg.named_product(self.words, self.members)
    }
}
