use alux_ext::ext;

/// Extends optional values with traversal operations.
#[ext(name = OptionTraversableExt)]
pub impl<T> Option<T> {
    /// Sequencing operation on [Option] type when inner type is `Applicative` or `Monad` like [Result].
    /// See [sequence](OptionResultExt::sequence) for traverse with identity closure.
    /// Defined by [Conor McBride](https://doi.org/10.1017/S0956796807006326) (2005) in Haskell2010 base
    /// [Data.Traversable](https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html).
    /// > **Traversable** structures support element-wise **sequencing** of **Applicative** effects
    /// (thus also **Monad** effects) to construct new structures of the **same shape** as the input.
    ///
    /// ```hs
    /// class (Functor t, Foldable t) => Traversable t where
    ///   traverse :: Applicative f => (a -> f b) -> t a -> f (t b)
    /// ```
    /// From this Haskell definition `t` is [Option] and `f` is [Result].
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = Some(42).traverse(|x| Ok(x + 100));
    ///
    /// assert_eq!(r, Ok(Some(142)));
    /// ```
    #[inline]
    fn traverse<F, R, E>(self, f: F) -> Result<Option<R>, E>
    where
        F: FnOnce(T) -> Result<R, E>,
    {
        // Traverse defined in terms of `sequence`.
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.map(f).sequence()

        // Or defined directly by pattern matching.
        match self {
            Some(t) => f(t).map(Some),
            None => Ok(None),
        }
    }

    /// Similar to [traverse](OptionTraversableExt::traverse), but with inner value wrapped inside
    /// [Option] so it has effect of filtering None values.
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = Some(42).traverse_opt(|x| Ok(Some(x + 100)));
    ///
    /// assert_eq!(r, Ok(Some(142)));
    /// ```
    #[inline]
    fn traverse_opt<F, R, E>(self, f: F) -> Result<Option<R>, E>
    where
        F: FnOnce(T) -> Result<Option<R>, E>,
    {
        // Traverse (opt) defined in terms of `sequence` (opt).
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.map(f).sequence_opt()

        // Or defined directly by pattern matching.
        match self {
            Some(t) => f(t),
            None => Ok(None),
        }
    }
}

/// Extends optional results with sequencing.
#[ext(name = OptionResultExt)]
pub impl<T, E> Option<Result<T, E>> {
    /// An alias for [transpose](Option::transpose), a _correct_ name for this function, although written for
    /// the fixed data types ([Option] and [Result]). See also [traverse](OptionTraversableExt::traverse) variant
    /// that accepts a mapping closure.
    /// Defined by [Conor McBride](https://doi.org/10.1017/S0956796807006326) (2005) in Haskell2010 base
    /// [Data.Traversable](https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html).
    /// > **Traversable** structures support element-wise **sequencing** of **Applicative** effects
    /// (thus also **Monad** effects) to construct new structures of the **same shape** as the input.
    ///
    /// ```hs
    /// class (Functor t, Foldable t) => Traversable t where
    ///   sequence :: Applicative f => t (f a) -> f (t a)
    /// ```
    /// From this Haskell definition `t` is [Option] and `f` is [Result].
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = Some(Ok(42)).sequence();
    ///
    /// assert_eq!(r, Ok(Some(42)));
    /// ```
    #[inline]
    fn sequence(self) -> Result<Option<T>, E> {
        // 1. Sequence defined in terms of `traverse`.
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.traverse(identity)

        // 2. Sequence defined as alias for `Option::transpose`.
        self.transpose()

        // 3. Similar implementation as `Option::transpose`.
        // match self {
        //     Some(r) => r.map(Some),
        //     //            ^- Result::map (Functor)
        //     None => Ok(None),
        //     //       ^- Result::pure (Applicative)
        // }

        // Other implementations using _fold_.

        // 4. Using `fold` on [Option] type.
        // self.into_iter().fold(Ok(None), |_, r| r.map(Some))

        // 5. Using `unwrap` on [Option] type. Unwrap is fold in disguise!
        // self.map(|r| r.map(Some)).unwrap_or_else(|| Ok(None))
    }
}

/// Extends optional results containing optional values with filtered sequencing.
#[ext(name = OptionResultOptionExt)]
pub impl<T, E> Option<Result<Option<T>, E>> {
    /// Similar to [sequence](OptionResultExt::sequence), but with inner value wrapped inside
    /// [Option] so it has effect of filtering None values.
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = Some(Ok(Some(42))).sequence_opt();
    ///
    /// assert_eq!(r, Ok(Some(42)));
    /// ```
    #[inline]
    fn sequence_opt(self) -> Result<Option<T>, E> {
        // Sequence (opt) defined in terms of `traverse` (opt).
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.traverse_opt(identity)

        // Or defined directly by pattern matching.
        match self {
            Some(r) => Ok(r?),
            None => Ok(None),
        }
    }
}

/// Extends iterators with traversal and sequencing operations.
#[ext(name = IterTraversableExt)]
pub impl<This> This
where
    This: Iterator,
{
    /// Sequencing operation on [Iterator] type when inner type is `Applicative` or `Monad` like [Result].
    /// See [`IterTraversableExt::sequence`] for traverse with identity closure.
    /// Defined by [Conor McBride](https://doi.org/10.1017/S0956796807006326) (2005) in Haskell2010 base
    /// [Data.Traversable](https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html).
    /// > **Traversable** structures support element-wise **sequencing** of **Applicative** effects
    /// (thus also **Monad** effects) to construct new structures of the **same shape** as the input.
    ///
    /// ```hs
    /// class (Functor t, Foldable t) => Traversable t where
    ///   traverse :: Applicative f => (a -> f b) -> t a -> f (t b)
    /// ```
    /// From this Haskell definition `t` is [Iterator] and `f` is [Result].
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = [1, 2, 3].into_iter().traverse(|x| Ok(x + x));
    ///
    /// assert_eq!(r, Ok(vec![2, 4, 6]));
    ///
    /// let r: Result<_, ()> = Some(42).into_iter().traverse(|x| Ok(x + x));
    ///
    /// assert_eq!(r, Ok(vec![84]));
    /// ```
    #[inline]
    fn traverse<F, T, R, E>(self, f: F) -> Result<Vec<R>, E>
    where
        This: Iterator<Item = T>,
        F: FnMut(T) -> Result<R, E>,
    {
        // Traverse defined in terms of `sequence`.
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.map(f).sequence()

        // Or defined directly, which is what `sequence` here is anyway: collecting into one
        // `Result` stops at the first error and sizes the vector from the iterator's own hint.
        self.map(f).collect()
    }

    /// Sequencing operation on [Iterator] type when inner type is `Applicative` or `Monad` like [Result].
    /// See [`IterTraversableExt::sequence`] for traverse with identity closure.
    /// Defined by [Conor McBride](https://doi.org/10.1017/S0956796807006326) (2005) in Haskell2010 base
    /// [Data.Traversable](https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html).
    /// > **Traversable** structures support element-wise **sequencing** of **Applicative** effects
    /// (thus also **Monad** effects) to construct new structures of the **same shape** as the input.
    ///
    /// ```hs
    /// class (Functor t, Foldable t) => Traversable t where
    ///   traverse :: Applicative f => (a -> f b) -> t a -> f (t b)
    /// ```
    /// From this Haskell definition `t` is [Iterator] and `f` is [Result].
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = [1, 2, 3].into_iter().traverse_iter(|x| Ok(Some(x + x)));
    ///
    /// assert_eq!(r, Ok(vec![2, 4, 6]));
    ///
    /// let r: Result<_, ()> = Some(42).into_iter().traverse_iter(|x| Ok(Some(x + x)));
    ///
    /// assert_eq!(r, Ok(vec![84]));
    /// ```
    #[inline]
    fn traverse_opt<F, T, R, E>(self, mut f: F) -> Result<Vec<R>, E>
    where
        This: Iterator<Item = T>,
        F: FnMut(T) -> Result<Option<R>, E>,
    {
        // Traverse defined in terms of `sequence`.
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.map(f).sequence_opt()

        // Or defined directly by pattern matching.
        let mut acc = vec![];
        for x in self {
            if let Some(r) = f(x)? {
                acc.push(r);
            }
        }
        Ok(acc)
    }

    /// The same as [`IterTraversableExt::traverse_opt`], but accepts more general result
    /// value as `Iterator`.
    ///
    /// NOTE: The end goal is to have general definition like this for traverse/sequence of Traversable interface (API).
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = [1, 2, 3].into_iter().traverse_iter(|x| Ok(Some(x + x)));
    ///
    /// assert_eq!(r, Ok(vec![2, 4, 6]));
    ///
    /// let r: Result<_, ()> = [1, 2, 3].into_iter().traverse_iter(|x| Ok(vec![x, x + x]));
    ///
    /// assert_eq!(r, Ok(vec![1, 2, 2, 4, 3, 6]));
    /// ```
    #[inline]
    fn traverse_iter<F, T, I, R, E>(self, mut f: F) -> Result<Vec<R>, E>
    where
        This: Iterator<Item = T>,
        F: FnMut(T) -> Result<I, E>,
        I: IntoIterator<Item = R>,
    {
        // Traverse defined in terms of `sequence`.
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.map(f).sequence_iter()

        // Or defined directly by pattern matching.
        let mut acc = vec![];
        for x in self {
            acc.extend(f(x)?);
        }
        Ok(acc)
    }

    /// Sequencing operation on [Iterator] type when inner type is `Applicative` or `Monad` like [Result].
    /// See [`IterTraversableExt::sequence`] for traverse with identity closure.
    /// Defined by [Conor McBride](https://doi.org/10.1017/S0956796807006326) (2005) in Haskell2010 base
    /// [Data.Traversable](https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html).
    /// > **Traversable** structures support element-wise **sequencing** of **Applicative** effects
    /// (thus also **Monad** effects) to construct new structures of the **same shape** as the input.
    ///
    /// ```hs
    /// class (Functor t, Foldable t) => Traversable t where
    ///   traverse :: Applicative f => (a -> f b) -> t a -> f (t b)
    /// ```
    /// From this Haskell definition `t` is [Iterator] and `f` is [Result].
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = [Ok(1), Ok(2), Ok(3)].into_iter().sequence();
    ///
    /// assert_eq!(r, Ok(vec![1, 2, 3]));
    /// ```
    #[inline]
    fn sequence<T, E>(self) -> Result<Vec<T>, E>
    where
        This: Iterator<Item = Result<T, E>>,
    {
        // Sequence defined in terms of `traverse`.
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.traverse(identity)

        self.collect()
    }

    /// Sequencing operation on [Iterator] type when inner type is `Applicative` or `Monad` like [Result].
    /// See [`IterTraversableExt::sequence`] for traverse with identity closure.
    /// Defined by [Conor McBride](https://doi.org/10.1017/S0956796807006326) (2005) in Haskell2010 base
    /// [Data.Traversable](https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html).
    /// > **Traversable** structures support element-wise **sequencing** of **Applicative** effects
    /// (thus also **Monad** effects) to construct new structures of the **same shape** as the input.
    ///
    /// ```hs
    /// class (Functor t, Foldable t) => Traversable t where
    ///   traverse :: Applicative f => (a -> f b) -> t a -> f (t b)
    /// ```
    /// From this Haskell definition `t` is [Iterator] and `f` is [Result].
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = [Ok(Some(1)), Ok(Some(2)), Ok(None), Ok(Some(3))].into_iter().sequence_opt();
    ///
    /// assert_eq!(r, Ok(vec![1, 2, 3]));
    /// ```
    #[inline]
    fn sequence_opt<T, E>(self) -> Result<Vec<T>, E>
    where
        This: Iterator<Item = Result<Option<T>, E>>,
    {
        // Sequence (opt) defined in terms of `traverse` (opt).
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.traverse_opt(identity)

        let mut acc = vec![];
        for x in self {
            if let Some(r) = x? {
                acc.push(r);
            }
        }
        Ok(acc)
    }

    /// The same as [`IterTraversableExt::sequence_opt`], but accepts more general result
    /// value as `Iterator`.
    ///
    /// NOTE: The end goal is to have general definition like this for traverse/sequence of Traversable interface (API).
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_traversable::*;
    ///
    /// let r: Result<_, ()> = [Ok(Some(1)), Ok(Some(2)), Ok(None), Ok(Some(3))].into_iter().sequence_iter();
    ///
    /// assert_eq!(r, Ok(vec![1, 2, 3]));
    ///
    /// let r: Result<_, ()> = [Ok(vec![1, 2]), Ok(vec![]), Ok(vec![3])].into_iter().sequence_iter();
    ///
    /// assert_eq!(r, Ok(vec![1, 2, 3]));
    /// ```
    #[inline]
    fn sequence_iter<T, I, E>(self) -> Result<Vec<T>, E>
    where
        This: Iterator<Item = Result<I, E>>,
        I: IntoIterator<Item = T>,
    {
        // Sequence defined in terms of `traverse`.
        // NOTE: Traversable minimal definition is `traverse` or `sequence` so only one needs to be
        //       implemented and other can be derived.
        //       https://hackage.haskell.org/package/base-4.21.0.0/docs/Data-Traversable.html
        // self.traverse_iter(identity)

        // Or defined directly by pattern matching.
        let mut acc = vec![];
        for x in self {
            acc.extend(x?);
        }
        Ok(acc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traverse() {
        // Result Ok

        for input in [Some(42), None] {
            let res: Result<_, ()> = input.traverse(Ok);

            assert_eq!(res, Ok(input));
        }

        let res: Result<Option<i32>, ()> = None.traverse(|()| Err(()));

        assert_eq!(res, Ok(None));

        // Result Err

        let res: Result<Option<i32>, ()> = Some(42).traverse(|_| Err(()));

        assert_eq!(res, Err(()));
    }

    #[test]
    fn test_traverse_opt() {
        // Result Ok

        let res: Result<_, ()> = Some(42).traverse_opt(|x| Ok(Some(x + x)));

        assert_eq!(res, Ok(Some(84)));

        let res: Result<_, ()> = Option::<i32>::None.traverse_opt(|x| Ok(Some(x + x)));

        assert_eq!(res, Ok(None));

        let res: Result<Option<i32>, ()> = Some(42).traverse_opt(|_| Ok(None));

        assert_eq!(res, Ok(None));

        let res: Result<Option<i32>, ()> = None.traverse_opt(|_: u32| Err(()));

        assert_eq!(res, Ok(None));

        // Result Err

        let res: Result<Option<i32>, ()> = Some(42).traverse_opt(|_| Err(()));

        assert_eq!(res, Err(()));
    }

    #[test]
    fn test_traverse_iter() {
        // Result Ok
        let input_vec = [vec![1, 2], vec![], vec![3]];
        let input_opt = [Some(1), Some(2), None, Some(3)];

        let res_vec: Result<Vec<i32>, ()> =
            input_vec.clone().into_iter().traverse_iter(|xs| Ok(xs.into_iter().map(|x| x + x)));

        let res_opt: Result<Vec<i32>, ()> = input_opt.into_iter().traverse_iter(|_| Ok(vec![].into_iter()));

        assert_eq!(res_vec, Ok(vec![2, 4, 6]));
        assert_eq!(res_opt, Ok(vec![]));

        // Simplest error
        let err = Result::<Vec<i32>, ()>::Err(());

        // Traverse empty
        let res_vec: Result<Vec<i32>, ()> = [].into_iter().traverse_iter(|_: i32| err.clone());
        let res_opt: Result<Vec<i32>, ()> = None.into_iter().traverse_iter(|_: i32| err.clone());

        assert_eq!(res_vec, Ok(vec![]));
        assert_eq!(res_opt, Ok(vec![]));

        // Result Err

        let res_vec: Result<Vec<i32>, ()> = [1].into_iter().traverse_iter(|_| err.clone());
        let res_opt: Result<Vec<i32>, ()> = Some(1).into_iter().traverse_iter(|_| err.clone());

        assert_eq!(res_vec, Err(()));
        assert_eq!(res_opt, Err(()));
    }

    #[test]
    fn test_sequence() {
        for (input, expected) in [
            // Result Ok
            (Some(Ok(42)), Ok(Some(42))),
            (None, Ok(None)),
            // Result Err
            (Some(Err(())), Err(())),
        ] {
            let res = input.sequence();

            assert_eq!(res, expected);
        }
    }

    #[test]
    fn test_sequence_opt() {
        for (input, expected) in [
            // Result Ok
            (Some(Ok(Some(42))), Ok(Some(42))),
            (Some(Ok(None)), Ok(None)),
            (None, Ok(None)),
            // Result Err
            (Some(Err(())), Err(())),
        ] {
            let res = input.sequence_opt();

            assert_eq!(res, expected);
        }
    }

    #[test]
    fn test_sequence_iter() {
        for (input_vec, input_opt, expected) in [
            // Result Ok
            (
                // Input Vec
                vec![Ok(vec![1, 2]), Ok(vec![]), Ok(vec![3])],
                // Input Option
                vec![Ok(Some(1)), Ok(Some(2)), Ok(None), Ok(Some(3))],
                // Expected result
                Ok(vec![1, 2, 3]),
            ),
            (vec![Ok(vec![])], vec![Ok(None)], Ok(vec![])),
            // Result Err
            (vec![Err(())], vec![Err(())], Err(())),
        ] {
            let res_vec = input_vec.into_iter().sequence_iter();
            let res_opt = input_opt.into_iter().sequence_iter();

            assert_eq!(res_vec, expected);
            assert_eq!(res_opt, expected);
        }
    }
}

#[cfg(test)]
mod iterator_instance_tests {
    use super::IterTraversableExt;

    #[test]
    fn iterator_traverse_preserves_shape_and_error() {
        let success: Result<Vec<_>, ()> = [1, 2, 3].into_iter().traverse(|x| Ok(x + x));
        assert_eq!(success, Ok(vec![2, 4, 6]));

        let empty: Result<Vec<i32>, ()> = [].into_iter().traverse(|x: i32| Ok(x));
        assert_eq!(empty, Ok(vec![]));

        let failure: Result<Vec<i32>, ()> = [1].into_iter().traverse(|_| Err(()));
        assert_eq!(failure, Err(()));

        let mut total = 0;
        let stateful: Result<Vec<_>, ()> = [1, 2, 3].into_iter().traverse(|value| {
            total += value;
            Ok(total)
        });
        assert_eq!(stateful, Ok(vec![1, 3, 6]));
    }

    #[test]
    fn iterator_traverse_opt_filters_none_and_preserves_error() {
        let success: Result<Vec<_>, ()> = [Some(1), None, Some(3)].into_iter().traverse_opt(Ok);
        assert_eq!(success, Ok(vec![1, 3]));

        let failure: Result<Vec<i32>, ()> = [1].into_iter().traverse_opt(|_| Err(()));
        assert_eq!(failure, Err(()));
    }

    #[test]
    fn iterator_sequence_preserves_shape_and_error() {
        let success = [Ok(1), Ok(2), Ok(3)].into_iter().sequence();
        assert_eq!(success, Ok::<_, ()>(vec![1, 2, 3]));

        let failure = [Ok(1), Err(()), Ok(3)].into_iter().sequence();
        assert_eq!(failure, Err(()));
    }

    #[test]
    fn iterator_sequence_opt_filters_none_and_preserves_error() {
        let success = [Ok(Some(1)), Ok(None), Ok(Some(3))].into_iter().sequence_opt();
        assert_eq!(success, Ok::<_, ()>(vec![1, 3]));

        let failure = [Ok(Some(1)), Err(())].into_iter().sequence_opt();
        assert_eq!(failure, Err(()));
    }
}
