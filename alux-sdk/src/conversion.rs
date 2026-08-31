use alux_ext::ext;

/// Helper extensions implemented for any type related to [Into] interface.
///
/// NOTE: It also serves as an example of extension method pattern in Rust.
#[ext(name = IntoExt)]
pub impl<This> This {
    /// A version of [`Into::into`] function that can be used to explicitly specify the target
    /// conversion type.
    /// ```
    /// use alux_sdk::*;
    /// // `Into` requires context to infer the type,
    /// let _: i64 = Into::into(42_i8);
    /// // The opposite `From` requires context to infer the type,
    /// let _: i64 = From::from(42_i8);
    /// // `.to()` can be used in the same way as a postfix function (the same as `.into()`),
    /// let _: i64 = 42_i8.to();
    /// // but also with the type specified explicitly.
    /// let _ = 42_i8.to::<i64>();
    /// // Especially useful to chain multiple conversions inline.
    /// let _: i64 = 42_i8.to::<i16>().to::<i32>().into();
    /// ```
    fn to<R>(self) -> R
    where
        This: Into<R>,
    {
        self.into()
    }
}

/// These tests also serve as an example of two ways functions can be tested.
///
/// 1. With automatically created Mock versions of functions and checking if it's
///    called with the expected arguments.
///    Traits are just collection of functions so they are tested in a similar way.
///
/// 2. With generative testing (quick-check based) by using Arbitrary values as
///    function input and checking for expected result.
#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;
    use std::sync::Arc;

    // Testing with mock functions (mockall lib)

    use mockall::mock;
    use mockall::predicate::*;

    // Creates mock structure and its implementation for the `From` trait. It has
    //  generic parameter because testing extension is also generic.
    mock! {
        FromSt<T: 'static> {}

        impl<T: 'static> From<T> for FromSt<T> {
            fn from(t: T) -> Self;
        }
    }

    /// Proves that [`IntoExt::to`] delegates to [`From::from`].
    fn proof_to_calls_from<T: Debug + Clone + PartialEq + Send + 'static>(t: T) {
        // Mock of `From<T>::from` trait function with expectations
        let ctx = MockFromSt::<T>::from_context();
        ctx.expect()
            // From should be called once ...
            .times(1)
            // ... for any input value
            .with(eq(t.clone()))
            // Result of `from` is not tested here so return default
            .returning(|_| MockFromSt::default());

        // Call to testing extension function
        t.to::<MockFromSt<T>>();
    }

    #[test]
    fn run_proof_to_calls_from() {
        // Running mocking tests with different types even though running
        //  with only one type passes the test.
        proof_to_calls_from(42);
        proof_to_calls_from("Hello");
        proof_to_calls_from(());
        proof_to_calls_from(Arc::new(42));
    }

    // Testing with generative tests (proptest lib)

    use proptest::prelude::*;

    /// Testing structure with [From<T>] implementation
    #[derive(Debug, PartialEq)]
    struct Wrap<T>(T);

    impl<T> From<T> for Wrap<T> {
        fn from(t: T) -> Self {
            Wrap(t)
        }
    }

    fn from_and_to_fun_equal<T: PartialEq + Debug + Clone>(t: T) {
        // Creates wrapped value created with `From`
        let wrapped_expected: Wrap<T> = From::from(t.clone());

        // Creates wrapped value created with `.to()`
        let wrapped_actual = t.to::<Wrap<T>>();

        assert_eq!(wrapped_expected, wrapped_actual);
    }

    // Running generative tests with different types even though running
    //  with only one type passes the test.
    proptest! {
        #[test]
        fn from_and_to_fun_equal_int(v: i128) {
            from_and_to_fun_equal(v);
        }

        #[test]
        fn from_and_to_fun_equal_str(v: String) {
            from_and_to_fun_equal(v);
        }
    }
}
