//! Every member of a shape, made optional.

use crate::delegate::delegate_shape_except_named;
use alux_shape::{FieldAlg, ShapeAlg, Words};

/// A shape whose members may all be absent, under a name of its own.
///
/// What a caller sends to change part of a value: the same shape, with nothing required. The name
/// moves with it, so the patch and the whole can be told apart wherever both are declared.
#[derive(Debug, Clone, Copy, Default)]
pub struct Patch<A>(pub A);

impl<A> Patch<A> {
    /// The algebra beneath this transformation.
    fn inner(&self) -> &A {
        &self.0
    }
}

impl<A> Patch<A>
where
    A: ShapeAlg,
{
    /// States the name this transformation answers under: the shape's own, and one word more.
    fn rename(&self, words: Words<'_>, body: A::Ty) -> A::Ty {
        let mut named: Vec<&str> = words.to_vec();
        named.push("patch");

        self.inner().named(&named, body)
    }
}

delegate_shape_except_named!(Patch);

impl<A> FieldAlg for Patch<A>
where
    A: ShapeAlg + FieldAlg,
{
    fn field(&self, words: Words<'_>, shape: A::Ty) -> A::Field {
        // The one operation this transformation is: a member that need not be there.
        self.inner().field(words, self.inner().opt(shape))
    }

    fn merge(&self, shape: A::Ty) -> A::Field {
        // A merged product has already been transformed, since it was folded by this algebra too.
        self.inner().merge(shape)
    }
}
