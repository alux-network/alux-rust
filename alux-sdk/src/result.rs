use alux_ext::ext;

/// Extends results with conversion operations.
#[ext(name = ResultExt)]
pub impl<T, E> Result<T, E> {
    /// Maps [Result] value using the [`Into`] trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_sdk::*;
    ///
    /// let _: Result<f64, ()> = Ok::<_, ()>(42i32).map_into::<f64>();
    /// ```
    #[inline]
    fn map_into<R>(self) -> Result<R, E>
    where
        T: Into<R>,
    {
        self.map(Into::into)
    }

    /// Converts the `Err` variant of this `Result` into another error type using `Into`,
    /// leaving the `Ok` value untouched.
    ///
    /// This is essentially the manual form of the `?` operator’s automatic error conversion
    /// (`?` calls `From::from` under the hood).
    /// Use it when you can’t write `?`:
    ///
    /// * inside a closure or iterator adaptor that must return a non-`Result` type,
    /// * in a function whose return type doesn’t match the error you need to convert.
    ///
    /// # Examples
    /// ```
    /// use alux_sdk::*;
    ///
    /// fn parse(data: &str) -> Result<u32, String> {
    ///     // `parse_inner` returns Result<u32, &'static str>
    ///     parse_inner(data).map_err_into()
    /// }
    ///
    /// fn parse2(data: &str) -> Result<u32, String> {
    ///     Ok(parse_inner(data)?)
    /// }
    ///
    /// fn parse_inner(_: &str) -> Result<u32, &'static str> {
    ///     Ok(42)
    /// }
    /// ```
    fn map_err_into<E2>(self) -> Result<T, E2>
    where
        E: Into<E2>,
    {
        self.map_err(E::into)
    }
}

/// Extends results containing optional values.
#[ext(name = ResultOptionExt)]
pub impl<T, E> Result<Option<T>, E> {
    /// Maps [Option] value inside [Result] with provided closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use alux_sdk::*;
    ///
    /// let r: Result<Option<i32>, ()> = Ok(Some(42)).map_opt(|x| x + 100);
    ///
    /// assert_eq!(r, Ok(Some(142)));
    /// ```
    #[inline]
    fn map_opt<R, F>(self, f: F) -> Result<Option<R>, E>
    where
        F: FnOnce(T) -> R,
    {
        self.map(|t| t.map(f))
    }
}
