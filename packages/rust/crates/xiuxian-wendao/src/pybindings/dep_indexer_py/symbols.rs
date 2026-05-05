use pyo3::{pyclass, pymethods};
use std::path::PathBuf;

use crate::dependency_indexer::{ExternalSymbol, SymbolIndex};

use super::helpers::{symbol_kind_from_str, symbol_kind_to_str, symbol_to_dict};

/// Python wrapper for `ExternalSymbol`.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyExternalSymbol {
    pub(crate) inner: ExternalSymbol,
}

#[pymethods]
impl PyExternalSymbol {
    #[new]
    fn new(name: &str, kind: &str, file: &str, line: usize, crate_name: &str) -> Self {
        let kind = symbol_kind_from_str(kind);
        Self {
            inner: ExternalSymbol {
                name: name.to_string(),
                kind,
                file: PathBuf::from(file),
                line,
                crate_name: crate_name.to_string(),
            },
        }
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    fn kind(&self) -> String {
        symbol_kind_to_str(&self.inner.kind).to_string()
    }

    #[getter]
    fn file(&self) -> String {
        self.inner.file.to_string_lossy().to_string()
    }

    #[getter]
    fn line(&self) -> usize {
        self.inner.line
    }

    #[getter]
    fn crate_name(&self) -> String {
        self.inner.crate_name.clone()
    }

    fn to_dict(&self) -> String {
        let value = serde_json::json!({
            "name": self.inner.name,
            "kind": self.kind(),
            "file": self.file(),
            "line": self.inner.line,
            "crate_name": self.inner.crate_name,
        });
        serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Python wrapper for `SymbolIndex`.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PySymbolIndex {
    pub(crate) inner: SymbolIndex,
}

#[pymethods]
impl PySymbolIndex {
    #[new]
    #[pyo3(signature = ())]
    fn new() -> Self {
        Self {
            inner: SymbolIndex::new(),
        }
    }

    fn search(&self, pattern: &str, limit: usize) -> Vec<PyExternalSymbol> {
        self.inner
            .search(pattern, limit)
            .into_iter()
            .map(|s| PyExternalSymbol { inner: s })
            .collect()
    }

    fn search_crate(&self, crate_name: &str, pattern: &str, limit: usize) -> Vec<PyExternalSymbol> {
        self.inner
            .search_crate(crate_name, pattern, limit)
            .into_iter()
            .map(|s| PyExternalSymbol { inner: s })
            .collect()
    }

    fn get_crates(&self) -> Vec<String> {
        self.inner
            .get_crates()
            .iter()
            .map(std::string::ToString::to_string)
            .collect()
    }

    fn symbol_count(&self) -> usize {
        self.inner.symbol_count()
    }

    fn crate_count(&self) -> usize {
        self.inner.crate_count()
    }

    fn clear(&mut self) {
        self.inner.clear();
    }

    fn serialize(&self) -> String {
        self.inner.serialize()
    }

    fn deserialize(&mut self, data: &str) -> bool {
        self.inner.deserialize(data)
    }

    /// Get results as JSON string.
    fn search_json(&self, pattern: &str, limit: usize) -> String {
        let results = self.inner.search(pattern, limit);
        let json_results: Vec<serde_json::Value> = results.iter().map(symbol_to_dict).collect();
        serde_json::to_string(&json_results).unwrap_or_else(|_| "[]".to_string())
    }

    /// Search within crate and return JSON.
    fn search_crate_json(&self, crate_name: &str, pattern: &str, limit: usize) -> String {
        let results = self.inner.search_crate(crate_name, pattern, limit);
        let json_results: Vec<serde_json::Value> = results.iter().map(symbol_to_dict).collect();
        serde_json::to_string(&json_results).unwrap_or_else(|_| "[]".to_string())
    }
}
