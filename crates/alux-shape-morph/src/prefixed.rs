//! Every name in a shape, under a word of its own.

use crate::delegate::{delegate_members, delegate_shape_except_named};
use alux_shape::{ShapeAlg, Words};

/// A shape whose names all begin with one word.
///
/// What keeps two surfaces' declarations apart where both are emitted into one place, without either
/// surface stating anything about the other.
#[derive(Debug, Clone, Copy)]
pub struct Prefixed<A> {
    inner: A,
    word: &'static str,
}

impl<A> Prefixed<A> {
    /// Puts `word` before every name this shape states.
    pub fn new(inner: A, word: &'static str) -> Self {
        Self { inner, word }
    }

    /// The algebra beneath this transformation.
    fn inner(&self) -> &A {
        &self.inner
    }
}

impl<A> Prefixed<A>
where
    A: ShapeAlg,
{
    /// States the name this transformation answers under.
    fn rename(&self, words: Words<'_>, body: A::Ty) -> A::Ty {
        let mut named = vec![self.word];
        named.extend_from_slice(words);

        self.inner().named(&named, body)
    }
}

delegate_shape_except_named!(Prefixed);
delegate_members!(Prefixed);
