//! `parsers::languages::rust::cargo::dependencies::types` owns Wendao cargo dependencies types behavior.

/// A parsed dependency.
#[derive(Debug, Clone)]
pub struct CargoDependency {
    /// Dependency crate name.
    pub name: String,
    /// Resolved dependency version requirement.
    pub version: String,
}

impl CargoDependency {
    /// Create a new parsed dependency record.
    #[must_use]
    pub fn new(name: String, version: String) -> Self {
        Self { name, version }
    }
}
