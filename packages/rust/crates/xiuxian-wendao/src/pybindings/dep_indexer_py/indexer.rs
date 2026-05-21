//! `pybindings::dep_indexer_py::indexer` owns Wendao pybindings dep indexer py indexer behavior.

use pyo3::{pyclass, pymethods};

use crate::dependency_indexer::{DependencyIndexResult, DependencyIndexer, DependencyStats};

use super::helpers::symbol_to_dict;
use super::symbols::PySymbolIndex;

/// Python wrapper for `DependencyIndexResult`.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyDependencyIndexResult {
    inner: DependencyIndexResult,
}

#[pymethods]
impl PyDependencyIndexResult {
    #[new]
    fn new(
        files_processed: usize,
        total_symbols: usize,
        errors: usize,
        crates_indexed: usize,
        error_details: Vec<String>,
    ) -> Self {
        Self {
            inner: DependencyIndexResult {
                files_processed,
                total_symbols,
                errors,
                crates_indexed,
                error_details,
            },
        }
    }

    #[getter]
    fn files_processed(&self) -> usize {
        self.inner.files_processed
    }

    #[getter]
    fn total_symbols(&self) -> usize {
        self.inner.total_symbols
    }

    #[getter]
    fn errors(&self) -> usize {
        self.inner.errors
    }

    #[getter]
    fn crates_indexed(&self) -> usize {
        self.inner.crates_indexed
    }

    #[getter]
    fn error_details(&self) -> Vec<String> {
        self.inner.error_details.clone()
    }

    fn to_dict(&self) -> String {
        let value = serde_json::json!({
            "files_processed": self.inner.files_processed,
            "total_symbols": self.inner.total_symbols,
            "errors": self.inner.errors,
            "crates_indexed": self.inner.crates_indexed,
            "error_details": self.inner.error_details,
        });
        serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Python wrapper for `DependencyStats`.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyDependencyStats {
    inner: DependencyStats,
}

#[pymethods]
impl PyDependencyStats {
    #[new]
    fn new(total_crates: usize, total_symbols: usize) -> Self {
        Self {
            inner: DependencyStats {
                total_crates,
                total_symbols,
            },
        }
    }

    #[getter]
    fn total_crates(&self) -> usize {
        self.inner.total_crates
    }

    #[getter]
    fn total_symbols(&self) -> usize {
        self.inner.total_symbols
    }

    fn to_dict(&self) -> String {
        let value = serde_json::json!({
            "total_crates": self.inner.total_crates,
            "total_symbols": self.inner.total_symbols,
        });
        serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Python wrapper for `DependencyIndexer`.
#[pyclass]
#[derive(Debug)]
pub struct PyDependencyIndexer {
    inner: DependencyIndexer,
}

#[pymethods]
impl PyDependencyIndexer {
    #[new]
    #[pyo3(signature = (project_root, config_path))]
    fn new(project_root: &str, config_path: Option<&str>) -> Self {
        Self {
            inner: DependencyIndexer::new(project_root, config_path),
        }
    }

    /// Build the dependency index (synchronous).
    #[pyo3(signature = (clean=false, verbose=false))]
    fn build(&mut self, clean: bool, verbose: bool) -> String {
        let _ = clean; // reserved for future cache-clearing behavior
        // CLI argument can force verbose; env var remains a fallback for global logging config.
        let env_verbose = std::env::var("OMNI_LOG_LEVEL").is_ok_and(|v| v == "DEBUG");
        let result = self.inner.build(verbose || env_verbose);
        serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string())
    }

    /// Search for symbols matching a pattern.
    /// Returns list of symbols as JSON string.
    fn search(&self, pattern: &str, limit: usize) -> String {
        let results = self.inner.search(pattern, limit);
        let json_results: Vec<serde_json::Value> = results.iter().map(symbol_to_dict).collect();
        serde_json::to_string(&json_results).unwrap_or_else(|_| "[]".to_string())
    }

    /// Search within a specific crate.
    fn search_crate(&self, crate_name: &str, pattern: &str, limit: usize) -> String {
        let results = self.inner.search_crate(crate_name, pattern, limit);
        let json_results: Vec<serde_json::Value> = results.iter().map(symbol_to_dict).collect();
        serde_json::to_string(&json_results).unwrap_or_else(|_| "[]".to_string())
    }

    /// Get list of indexed crates/packages.
    fn get_indexed(&self) -> Vec<String> {
        self.inner.get_indexed().iter().map(String::clone).collect()
    }

    /// Get statistics as JSON string.
    fn stats(&self) -> String {
        let stats = self.inner.stats();
        serde_json::to_string(&stats).unwrap_or_else(|_| "{}".to_string())
    }

    /// Load index from cache.
    fn load_index(&mut self) -> bool {
        self.inner.load_index().is_ok()
    }

    /// Get the symbol index for direct manipulation (returns cloned data).
    fn get_symbol_index(&self) -> PySymbolIndex {
        PySymbolIndex {
            inner: self.inner.symbol_index.clone(),
        }
    }
}
