use alux_ext::ext;
use std::array::TryFromSliceError;

/// Extends iterators with exact and fallible collection helpers.
#[ext(name = IntoIteratorExt)]
pub impl<This> This {
    /// A variant of the `try_collect` and `collect_vec` combined function from the `Itertools`
    /// library. The materialized value is a [`Vec`] inside a [`Result`] object.
    ///
    /// Returns the first error without collecting later values.
    ///
    /// ```
    /// use alux_sdk::*;
    ///
    /// let values = [Ok::<_, &str>(1), Ok(2), Ok(3)]
    ///     .into_iter()
    ///     .try_collect_vec();
    ///
    /// assert_eq!(values, Ok(vec![1, 2, 3]));
    /// ```
    fn try_collect_vec<T, E>(self) -> Result<Vec<T>, E>
    where
        This: Iterator<Item = Result<T, E>>,
    {
        self.collect()
    }

    /// Takes exactly `n` elements from the iterator.
    ///
    /// Returns `Some(Vec<T>)` if the iterator yields exactly `n` elements,
    /// otherwise returns `None`.
    ///
    /// ```
    /// use alux_sdk::*;
    ///
    /// let v = [1, 2, 3];
    ///
    /// assert_eq!(v.iter().copied().collect_exact(3), Some(vec![1, 2, 3]));
    /// assert_eq!(v.iter().copied().collect_exact(4), None);
    /// assert_eq!(v.iter().copied().collect_exact(2), None);
    /// ```
    fn collect_exact<T>(self, n: usize) -> Option<Vec<T>>
    where
        This: Iterator<Item = T>,
    {
        let mut iterator = self;
        let result = iterator.by_ref().take(n).collect::<Vec<_>>();

        (result.len() == n && iterator.next().is_none()).then_some(result)
    }

    /// Efficiently splits an iterator of `Result<(A, B), E>` into two `Vec`s, short-circuiting on error.
    ///
    /// Consumes the iterator, collecting all `A` elements into one vector and all `B` elements into another,
    /// without first allocating an intermediate vector of tuples. Returns the tuple `(Vec<A>, Vec<B>)` on success,
    /// or the first error encountered.
    ///
    /// ```
    /// use alux_sdk::*;
    ///
    /// let input = vec![Ok((1, "x")), Ok((2, "y"))];
    /// let (xs, ys) = input.try_unzip::<_, _, ()>().unwrap();
    ///
    /// assert_eq!(xs, vec![1, 2]);
    /// assert_eq!(ys, vec!["x", "y"]);
    /// ```
    fn try_unzip<A, B, E>(self) -> Result<(Vec<A>, Vec<B>), E>
    where
        This: IntoIterator<Item = Result<(A, B), E>>,
    {
        // Allocate two output vectors; their size will grow as we process items.
        let mut va = vec![];
        let mut vb = vec![];

        // Iterate over each item in the input iterator.
        for res in self {
            // Propagate the first error immediately, if encountered.
            let (a, b) = res?;
            // Push the first and second element to their respective vectors.
            va.push(a);
            vb.push(b);
        }

        // Return both vectors if all results were successful.
        Ok((va, vb))
    }
}

/// Extends immutable slices with prefix conversion to arrays.
#[ext(name = SliceExt)]
pub impl<'a, Item> &'a [Item] {
    /// Helper method to get a constant-length slice from a collection of equal or greater size.
    ///
    /// For the mutable variant, see [`SliceMutExt::try_to_const_mut`].
    ///
    /// ```
    /// use alux_sdk::*;
    ///
    /// let inp = &[1, 2, 3][..];
    ///
    /// let res = inp.try_to_const();
    /// assert_eq!(&[1, 2], res.unwrap());
    ///
    /// let res = inp.try_to_const::<3>();
    /// assert_eq!(&[1, 2, 3], res.unwrap());
    ///
    /// let res = inp.try_to_const::<4>();
    /// assert!(res.is_err());
    /// ```
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn try_to_const<const N: usize>(self) -> Result<&'a [Item; N], TryFromSliceError> {
        if self.len() > N { self[..N].try_into() } else { self.try_into() }
    }
}

/// Extends mutable slices with prefix conversion to arrays.
#[ext(name = SliceMutExt)]
pub impl<'a, Item> &'a mut [Item] {
    /// Helper method to get a constant-length slice from a collection of equal or greater size.
    ///
    /// It's the mutable variant of [`SliceExt::try_to_const`] method.
    ///
    /// ```
    /// use alux_sdk::*;
    ///
    /// let inp = &mut [1, 2, 3][..];
    ///
    /// let res = inp.try_to_const_mut();
    /// assert_eq!(&mut [1, 2], res.unwrap());
    ///
    /// let res = inp.try_to_const_mut();
    /// assert_eq!(&mut [1, 2, 3], res.unwrap());
    ///
    /// let res = inp.try_to_const_mut::<4>();
    /// assert!(res.is_err());
    /// ```
    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn try_to_const_mut<const N: usize>(self) -> Result<&'a mut [Item; N], TryFromSliceError> {
        if self.len() > N { (&mut self[..N]).try_into() } else { self.try_into() }
    }
}

#[cfg(test)]
mod tests {
    use super::{IntoIteratorExt, SliceExt, SliceMutExt};

    #[test]
    fn try_to_const() {
        let inp = &[1, 2, 3][..];

        let res = inp.try_to_const();
        assert_eq!(&[1, 2], res.unwrap());

        let res = inp.try_to_const();
        assert_eq!(&[1, 2, 3], res.unwrap());

        let res = inp.try_to_const::<4>();
        assert!(res.is_err());
    }

    #[test]
    fn try_to_const_mut() {
        let inp = &mut [1, 2, 3][..];

        let res = inp.try_to_const_mut();
        assert_eq!(&mut [1, 2], res.unwrap());

        let res = inp.try_to_const_mut();
        assert_eq!(&mut [1, 2, 3], res.unwrap());

        let res = inp.try_to_const_mut::<4>();
        assert!(res.is_err());
    }

    #[test]
    fn collect_exact_accepts_only_the_requested_shape() {
        assert_eq!((0..3).collect_exact(3), Some(vec![0, 1, 2]));
        assert_eq!((0..3).collect_exact(2), None);
        assert_eq!((0..3).collect_exact(4), None);
    }

    #[test]
    fn try_collect_vec_stops_at_the_first_error() {
        let result = [Ok(1), Ok(2), Err("stop"), Ok(3)].into_iter().try_collect_vec();
        assert_eq!(result, Err("stop"));
    }

    #[test]
    fn try_unzip_preserves_pair_order() {
        let result = [Ok::<_, ()>((1, "a")), Ok((2, "b"))].into_iter().try_unzip();
        assert_eq!(result, Ok((vec![1, 2], vec!["a", "b"])));
    }

    #[test]
    fn try_unzip_stops_at_the_first_error() {
        let result = [Ok((1, "a")), Err("stop"), Ok((3, "c"))].into_iter().try_unzip();
        assert_eq!(result, Err("stop"));
    }

    #[test]
    fn try_to_const_borrows_a_prefix() {
        assert_eq!([1, 2, 3].as_slice().try_to_const::<2>().expect("two-item prefix"), &[1, 2]);
        assert!([1, 2, 3].as_slice().try_to_const::<4>().is_err());
    }

    #[test]
    fn try_to_const_mut_borrows_a_mutable_prefix() {
        let mut values = [1, 2, 3];
        let prefix = values.as_mut_slice().try_to_const_mut::<2>().expect("two-item mutable prefix");
        prefix[0] = 7;
        assert_eq!(values, [7, 2, 3]);
    }
}
