//! Sequencing the `Result` effect through a tuple: the heterogeneous structure, whose output keeps
//! each element's own type.

use alux_ext::for_tuples;

/// Sequences the `Result` effect through a tuple of `Result`s sharing the error `E`.
///
/// `(Ok(a), Ok(b)).sequence()` is `Ok((a, b))`, and any `Err` makes it the leftmost `Err`. The
/// error is a parameter, not an associated type, so the empty tuple can sequence to `Ok(())` for
/// any error a caller generic over the width names.
pub trait TupleSequence<E> {
    /// The tuple of `Ok` values.
    type Output;

    /// Returns the tuple of `Ok` values, or the leftmost `Err`.
    ///
    /// # Errors
    ///
    /// Returns the leftmost element's error.
    fn sequence(self) -> Result<Self::Output, E>;
}

/// States `TupleSequence` for one width.
macro_rules! tuple_sequence {
    ($($element:ident => $index:tt),*) => {
        impl<E, $($element),*> TupleSequence<E> for ($(Result<$element, E>,)*) {
            type Output = ($($element,)*);

            #[allow(non_snake_case, reason = "each binding is named after its element's type")]
            fn sequence(self) -> Result<Self::Output, E> {
                let ($($element,)*) = self;
                Ok(($($element?,)*))
            }
        }
    };
}

for_tuples!(tuple_sequence, 0..=16);
