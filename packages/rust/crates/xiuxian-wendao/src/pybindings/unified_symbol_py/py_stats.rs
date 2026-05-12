//! `pybindings::unified_symbol_py::py_stats` owns Wendao pybindings unified symbol py py stats behavior.

use crate::unified_symbol::UnifiedIndexStats;
use pyo3::{pyclass, pymethods};
use xiuxian_macros::py_from;

/// Python wrapper for `UnifiedIndexStats`.
#[pyclass]
#[derive(Debug, Default, Clone)]
pub struct PyUnifiedIndexStats {
    pub(crate) inner: UnifiedIndexStats,
}

// Generate From<UnifiedIndexStats> for PyUnifiedIndexStats.
py_from!(PyUnifiedIndexStats, UnifiedIndexStats);

#[pymethods]
impl PyUnifiedIndexStats {
    #[new]
    #[pyo3(signature = (total_symbols, project_symbols, external_symbols, external_crates, project_files_with_externals))]
    fn new(
        total_symbols: usize,
        project_symbols: usize,
        external_symbols: usize,
        external_crates: usize,
        project_files_with_externals: usize,
    ) -> Self {
        Self {
            inner: UnifiedIndexStats {
                total_symbols,
                project_symbols,
                external_symbols,
                external_crates,
                project_files_with_externals,
            },
        }
    }

    #[getter]
    fn total_symbols(&self) -> usize {
        self.inner.total_symbols
    }

    #[getter]
    fn project_symbols(&self) -> usize {
        self.inner.project_symbols
    }

    #[getter]
    fn external_symbols(&self) -> usize {
        self.inner.external_symbols
    }

    #[getter]
    fn external_crates(&self) -> usize {
        self.inner.external_crates
    }

    #[getter]
    fn project_files_with_externals(&self) -> usize {
        self.inner.project_files_with_externals
    }

    fn to_dict(&self) -> String {
        let value = serde_json::json!({
            "total_symbols": self.inner.total_symbols,
            "project_symbols": self.inner.project_symbols,
            "external_symbols": self.inner.external_symbols,
            "external_crates": self.inner.external_crates,
            "project_files_with_externals": self.inner.project_files_with_externals,
        });
        serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
    }
}
