//! Result surface for integration-test support helpers.

/// Result type used by public test-support helpers.
pub type TestSupportResult<T> = anyhow::Result<T>;
