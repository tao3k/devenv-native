use pyo3::{pyclass, pymethods};

use crate::unified_symbol::UnifiedSymbolIndex;

use super::{PyUnifiedIndexStats, PyUnifiedSymbol};

/// Python wrapper for `UnifiedSymbolIndex`.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyUnifiedSymbolIndex {
    pub(crate) inner: UnifiedSymbolIndex,
}

#[pymethods]
impl PyUnifiedSymbolIndex {
    #[new]
    #[pyo3(signature = ())]
    fn new() -> Self {
        Self {
            inner: UnifiedSymbolIndex::new(),
        }
    }

    /// Add a project symbol.
    #[pyo3(signature = (name, kind, location, crate_name))]
    fn add_project_symbol(&mut self, name: &str, kind: &str, location: &str, crate_name: &str) {
        self.inner
            .add_project_symbol(name, kind, location, crate_name);
    }

    /// Add an external dependency symbol.
    #[pyo3(signature = (name, kind, location, crate_name))]
    fn add_external_symbol(&mut self, name: &str, kind: &str, location: &str, crate_name: &str) {
        self.inner
            .add_external_symbol(name, kind, location, crate_name);
    }

    /// Record usage of an external symbol in a project file.
    #[pyo3(signature = (crate_name, symbol_name, project_file))]
    fn record_external_usage(&mut self, crate_name: &str, symbol_name: &str, project_file: &str) {
        self.inner
            .record_external_usage(crate_name, symbol_name, project_file);
    }

    /// Search across both project and external symbols.
    #[pyo3(signature = (pattern, limit))]
    fn search_unified(&self, pattern: &str, limit: usize) -> Vec<PyUnifiedSymbol> {
        self.inner
            .search_unified(pattern, limit)
            .into_iter()
            .map(|s| PyUnifiedSymbol { inner: s })
            .collect()
    }

    /// Search only project symbols.
    #[pyo3(signature = (pattern, limit))]
    fn search_project(&self, pattern: &str, limit: usize) -> Vec<PyUnifiedSymbol> {
        self.inner
            .search_project(pattern, limit)
            .into_iter()
            .map(|s| PyUnifiedSymbol { inner: s })
            .collect()
    }

    /// Search only external symbols.
    #[pyo3(signature = (pattern, limit))]
    fn search_external(&self, pattern: &str, limit: usize) -> Vec<PyUnifiedSymbol> {
        self.inner
            .search_external(pattern, limit)
            .into_iter()
            .map(|s| PyUnifiedSymbol { inner: s })
            .collect()
    }

    /// Search within a specific crate.
    #[pyo3(signature = (crate_name, pattern, limit))]
    fn search_crate(&self, crate_name: &str, pattern: &str, limit: usize) -> Vec<PyUnifiedSymbol> {
        self.inner
            .search_crate(crate_name, pattern, limit)
            .into_iter()
            .map(|s| PyUnifiedSymbol { inner: s })
            .collect()
    }

    /// Find where an external crate's symbols are used in the project.
    #[pyo3(signature = (crate_name))]
    fn find_external_usage(&self, crate_name: &str) -> Vec<String> {
        self.inner.find_external_usage(crate_name)
    }

    /// Get all external crates used in the project.
    fn get_external_crates(&self) -> Vec<String> {
        self.inner.get_external_crates()
    }

    /// Get all project crates.
    fn get_project_crates(&self) -> Vec<String> {
        self.inner.get_project_crates()
    }

    /// Get statistics.
    fn stats(&self) -> PyUnifiedIndexStats {
        self.inner.stats().into()
    }

    /// Get stats as JSON.
    fn stats_json(&self) -> String {
        let stats = self.inner.stats();
        serde_json::to_string(&stats).unwrap_or_else(|_| "{}".to_string())
    }

    /// Clear all symbols.
    fn clear(&mut self) {
        self.inner.clear();
    }

    /// Search unified and return JSON.
    #[pyo3(signature = (pattern, limit))]
    fn search_unified_json(&self, pattern: &str, limit: usize) -> String {
        let results = self.inner.search_unified(pattern, limit);
        let json_results: Vec<serde_json::Value> = results
            .iter()
            .map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "kind": s.kind,
                    "location": s.location,
                    "source": if s.is_external() { "external" } else { "project" },
                    "crate_name": s.crate_name,
                })
            })
            .collect();
        serde_json::to_string(&json_results).unwrap_or_else(|_| "[]".to_string())
    }
}
