/// Shared strict-clippy test assertions that avoid `expect`/`expect_err`.
pub(crate) trait MustExt {
    /// Successful value returned by `must`.
    type Output;
    /// Failure value returned by `must_err`.
    type Failure;

    /// Returns the success payload or panics with context.
    fn must(self, context: &str) -> Self::Output;
    /// Returns the failure payload or panics with context.
    fn must_err(self, context: &str) -> Self::Failure;
}

impl<T, E> MustExt for Result<T, E>
where
    E: std::fmt::Debug,
{
    type Output = T;
    type Failure = E;

    fn must(self, context: &str) -> Self::Output {
        match self {
            Ok(value) => value,
            Err(error) => panic!("{context}: {error:?}"),
        }
    }

    fn must_err(self, context: &str) -> Self::Failure {
        match self {
            Ok(_) => panic!("{context}: got Ok(..)"),
            Err(error) => error,
        }
    }
}

impl<T> MustExt for Option<T> {
    type Output = T;
    type Failure = ();

    fn must(self, context: &str) -> Self::Output {
        match self {
            Some(value) => value,
            None => panic!("{context}: got None"),
        }
    }

    fn must_err(self, context: &str) -> Self::Failure {
        assert!(self.is_none(), "{context}: got Some(..)");
    }
}
