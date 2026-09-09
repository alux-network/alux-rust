use alux_ext::ext;
use core::convert::identity;
use core::iter::{FlatMap, Flatten};

/// Extends optional values with dependent composition.
#[ext(name = OptionMonadExt)]
pub impl<T> Option<T> {
    /// Applies the next optional computation to a present value, preserving absence.
    #[inline]
    fn bind<R, F>(self, f: F) -> Option<R>
    where
        F: FnOnce(T) -> Option<R>,
    {
        self.and_then(f)
    }
}

/// Extends nested optional values with flattening.
#[ext(name = OptionJoinExt)]
pub impl<T> Option<Option<T>> {
    /// Removes one optional layer, returning absence if either layer is absent.
    #[inline]
    fn join(self) -> Option<T> {
        self.bind(identity)
    }
}

/// Extends results with dependent composition in one error type.
#[ext(name = ResultMonadExt)]
pub impl<T, E> Result<T, E> {
    /// Applies the next fallible computation to a success, preserving the first error.
    #[inline]
    fn bind<R, F>(self, f: F) -> Result<R, E>
    where
        F: FnOnce(T) -> Result<R, E>,
    {
        self.and_then(f)
    }
}

/// Extends nested results sharing an error type with flattening.
#[ext(name = ResultJoinExt)]
pub impl<T, E> Result<Result<T, E>, E> {
    /// Removes one result layer, preserving the outer error or the inner result.
    #[inline]
    fn join(self) -> Result<T, E> {
        self.bind(identity)
    }
}

/// Extends iterators with lazy, ordered dependent composition.
#[ext(name = IterMonadExt)]
pub impl<This> This
where
    This: Iterator,
{
    /// Concatenates each next computation in source order as items are requested.
    ///
    /// Accepts stateful closures; each inner iterator is exhausted before advancing
    /// the outer iterator. An infinite inner iterator can prevent later inputs being visited.
    #[inline]
    fn bind<T, I, F>(self, f: F) -> FlatMap<This, I, F>
    where
        This: Iterator<Item = T> + Sized,
        I: IntoIterator,
        F: FnMut(T) -> I,
    {
        self.flat_map(f)
    }

    /// Removes one iterable layer lazily, preserving outer and inner order.
    ///
    /// Denotes binding the identity function, using the standard flatten adapter.
    #[inline]
    fn join<I>(self) -> Flatten<This>
    where
        This: Iterator<Item = I> + Sized,
        I: IntoIterator,
    {
        self.flatten()
    }
}
