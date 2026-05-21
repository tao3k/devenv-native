//! `parsers::languages::python::pyproject::dependencies` owns Wendao python pyproject dependencies behavior.

mod api;
mod regex;
mod types;

pub use self::api::parse_pyproject_dependencies;
pub use self::types::PyprojectDependency;

#[cfg(test)]
#[path = "../../../../../../tests/unit/parsers/languages/python/pyproject/dependencies.rs"]
mod tests;
