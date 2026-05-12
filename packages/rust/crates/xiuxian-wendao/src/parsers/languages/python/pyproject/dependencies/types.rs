//! `parsers::languages::python::pyproject::dependencies::types` owns Wendao pyproject dependencies types behavior.

/// A parsed Python dependency from `pyproject.toml`.
#[derive(Debug, Clone)]
pub struct PyprojectDependency {
    /// Python package name.
    pub name: String,
    /// Optional parsed version constraint or value.
    pub version: Option<String>,
}

impl PyprojectDependency {
    /// Create a new parsed pyproject dependency record.
    #[must_use]
    pub fn new(name: String, version: Option<String>) -> Self {
        Self { name, version }
    }
}
