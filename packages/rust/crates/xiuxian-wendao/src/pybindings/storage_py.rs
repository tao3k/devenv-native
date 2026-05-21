//! `PyO3` bindings for `KnowledgeStorage` (Valkey operations).

use pyo3::{PyErr, PyResult, pyclass, pymethods};
use tokio::runtime::Runtime;

use super::knowledge_py::PyKnowledgeEntry;
use crate::storage::KnowledgeStorage;

/// Knowledge storage Python wrapper.
#[pyclass]
#[derive(Debug)]
pub struct PyKnowledgeStorage {
    inner: KnowledgeStorage,
    runtime: Runtime,
}

#[pymethods]
impl PyKnowledgeStorage {
    /// Create a new storage instance.
    ///
    /// # Errors
    ///
    /// Returns a Python runtime error when the Tokio runtime cannot be created.
    #[new]
    #[pyo3(signature = (path, table_name))]
    pub fn new(path: &str, table_name: &str) -> PyResult<Self> {
        let storage = KnowledgeStorage::new(path, table_name);
        let runtime = Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(Self {
            inner: storage,
            runtime,
        })
    }

    /// Add an entry to storage.
    ///
    /// # Errors
    ///
    /// Returns a Python runtime error when the underlying storage upsert fails.
    #[allow(clippy::needless_pass_by_value)] // PyO3 extracts owned wrapper values at the Python binding boundary.
    pub fn add_entry(&self, entry: PyKnowledgeEntry) -> PyResult<()> {
        self.inner
            .upsert(&entry.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get an entry by ID.
    ///
    /// # Errors
    ///
    /// Returns a Python runtime error when the storage lookup fails.
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn get_entry(&self, entry_id: &str) -> PyResult<Option<PyKnowledgeEntry>> {
        self.runtime.block_on(async {
            let entry = self
                .inner
                .get_entry(entry_id)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
            Ok(entry.map(|e| PyKnowledgeEntry { inner: e }))
        })
    }

    /// Search entries by text.
    ///
    /// # Errors
    ///
    /// Returns a Python runtime error when the underlying text search fails.
    pub fn text_search(&self, query: &str, limit: i32) -> PyResult<Vec<PyKnowledgeEntry>> {
        let entries = self
            .inner
            .search_text(query, limit)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(entries
            .into_iter()
            .map(|e| PyKnowledgeEntry { inner: e })
            .collect())
    }

    /// Search entries by vector similarity.
    ///
    /// # Errors
    ///
    /// Returns a Python runtime error when the underlying vector search fails.
    #[allow(clippy::needless_pass_by_value)] // PyO3 converts Python sequences into owned Rust vectors for method calls.
    pub fn vector_search(
        &self,
        query_vector: Vec<f32>,
        limit: i32,
    ) -> PyResult<Vec<PyKnowledgeEntry>> {
        let entries = self
            .inner
            .search(&query_vector, limit)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(entries
            .into_iter()
            .map(|e| PyKnowledgeEntry { inner: e })
            .collect())
    }

    /// Hybrid search (text + vector).
    ///
    /// # Errors
    ///
    /// Returns a Python runtime error when the underlying search fails.
    pub fn search(&self, query: &str, limit: i32) -> PyResult<Vec<PyKnowledgeEntry>> {
        let entries = self
            .inner
            .search_text(query, limit)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(entries
            .into_iter()
            .map(|e| PyKnowledgeEntry { inner: e })
            .collect())
    }
}
