pub(super) trait PanicExt<T> {
    fn or_panic(self, context: &str) -> T;
}

impl<T, E> PanicExt<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn or_panic(self, context: &str) -> T {
        self.unwrap_or_else(|error| panic!("{context}: {error}"))
    }
}

impl<T> PanicExt<T> for Option<T> {
    fn or_panic(self, context: &str) -> T {
        self.unwrap_or_else(|| panic!("{context}"))
    }
}
