use alux_ext::ext;

/// Extends options with a predicate and a conversion.
#[ext(name = OptionExt)]
pub impl<T> Option<T> {
    /// Returns `true` if the option is a [`Some`] and the value inside is the same as supplied.
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_sdk::*;
    ///
    /// let x: Option<u32> = Some(2);
    /// assert_eq!(x.is_some_eq(&2), true);
    ///
    /// let x: Option<u32> = Some(0);
    /// assert_eq!(x.is_some_eq(&2), false);
    ///
    /// let x: Option<u32> = None;
    /// assert_eq!(x.is_some_eq(&2), false);
    /// ```
    #[inline]
    fn is_some_eq(&self, other: &T) -> bool
    where
        T: PartialEq<T>,
    {
        self.as_ref().is_some_and(|t| t == other)
    }

    /// Maps [Option] value using the [`Into`] trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_sdk::*;
    ///
    /// let _: Option<f64> = Some(42i32).map_into::<f64>();
    /// ```
    #[inline]
    fn map_into<R>(self) -> Option<R>
    where
        T: Into<R>,
    {
        self.map(Into::into)
    }
}
