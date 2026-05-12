//! `pybindings::graph_py::py_graph::cache` owns Wendao graph py py graph cache behavior.

use pyo3::{PyResult, pyfunction};

use crate::kg_cache;

use super::PyKnowledgeGraph;

/// Invalidate the in-process KG cache for the given scope key.
///
/// Call after evicting the knowledge vector store so the long-lived process
/// does not retain the graph in memory. Safe to call when cache is empty.
#[pyfunction]
pub fn invalidate_kg_cache(scope_key: &str) {
    kg_cache::invalidate(scope_key);
}

/// Load `KnowledgeGraph` from Valkey with caching.
///
/// Uses an in-process cache keyed by path. Avoids repeated disk reads
/// when the same scope key is accessed across multiple recalls.
/// Returns None only when backend returns empty and caller chooses to ignore it.
///
/// # Errors
///
/// Returns `PyErr` when Valkey loading fails.
#[pyfunction]
pub fn load_kg_from_valkey_cached(scope_key: &str) -> PyResult<Option<PyKnowledgeGraph>> {
    match kg_cache::load_from_valkey_cached(scope_key) {
        Ok(Some(graph)) => Ok(Some(PyKnowledgeGraph { inner: graph })),
        Ok(None) => Ok(None),
        Err(error) => Err(pyo3::exceptions::PyIOError::new_err(error.to_string())),
    }
}
