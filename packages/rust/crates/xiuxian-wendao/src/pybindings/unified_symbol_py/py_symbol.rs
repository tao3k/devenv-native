//! `pybindings::unified_symbol_py::py_symbol` owns Wendao pybindings unified symbol py py symbol behavior.

use crate::unified_symbol::{SymbolSource, UnifiedSymbol};
use pyo3::{pyclass, pymethods};

/// Python wrapper for `UnifiedSymbol`.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyUnifiedSymbol {
    pub(crate) inner: UnifiedSymbol,
}

#[pymethods]
impl PyUnifiedSymbol {
    #[new]
    #[pyo3(signature = (name, kind, location, source, crate_name))]
    fn new(name: &str, kind: &str, location: &str, source: &str, crate_name: &str) -> Self {
        let _source = if source == "project" {
            SymbolSource::Project
        } else {
            SymbolSource::External(source.to_string())
        };
        Self {
            inner: UnifiedSymbol::new_external(name, kind, location, crate_name),
        }
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    fn kind(&self) -> String {
        self.inner.kind.clone()
    }

    #[getter]
    fn location(&self) -> String {
        self.inner.location.clone()
    }

    #[getter]
    fn crate_name(&self) -> String {
        self.inner.crate_name.clone()
    }

    #[getter]
    fn is_external(&self) -> bool {
        self.inner.is_external()
    }

    #[getter]
    fn is_project(&self) -> bool {
        self.inner.is_project()
    }

    fn to_dict(&self) -> String {
        let value = serde_json::json!({
            "name": self.inner.name,
            "kind": self.inner.kind,
            "location": self.inner.location,
            "source": if self.inner.is_external() { "external" } else { "project" },
            "crate_name": self.inner.crate_name,
        });
        serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
    }
}
