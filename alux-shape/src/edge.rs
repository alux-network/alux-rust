//! Where a Rust type meets a term.
//!
//! A term states a shape without naming a Rust type, and mostly that is the whole story. An
//! interpretation that holds a *type* rather than a term — a JSON-RPC generator reading an
//! operation's `Args` and `Output` — needs the one bridge stated here.

/// States a type's shape in whichever vocabulary an interpretation provides.
///
/// The bound sits on the impl, so the trait itself demands nothing of its algebra.
pub trait ShapeOf<Alg> {
    /// The shape the interpretation builds for this type.
    type Shape;

    /// Describes this type to the interpretation.
    fn shape_of(alg: &Alg) -> Self::Shape;
}
