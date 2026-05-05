use pyo3::PyResult;

use crate::kg_cache;

use super::PyKnowledgeGraph;

pub(super) fn save_to_file(graph: &PyKnowledgeGraph, path: &str) -> PyResult<()> {
    graph
        .inner
        .save_to_file(path)
        .map_err(|error| pyo3::exceptions::PyIOError::new_err(error.to_string()))
}

pub(super) fn load_from_file(graph: &mut PyKnowledgeGraph, path: &str) -> PyResult<()> {
    graph
        .inner
        .load_from_file(path)
        .map_err(|error| pyo3::exceptions::PyIOError::new_err(error.to_string()))
}

pub(super) fn save_to_valkey(
    graph: &PyKnowledgeGraph,
    scope_key: &str,
    dimension: usize,
) -> PyResult<()> {
    graph
        .inner
        .save_to_valkey(scope_key, dimension)
        .map_err(|error| pyo3::exceptions::PyIOError::new_err(error.to_string()))?;
    kg_cache::invalidate(scope_key);
    Ok(())
}

pub(super) fn load_from_valkey(graph: &mut PyKnowledgeGraph, scope_key: &str) -> PyResult<()> {
    graph
        .inner
        .load_from_valkey(scope_key)
        .map_err(|error| pyo3::exceptions::PyIOError::new_err(error.to_string()))
}

pub(super) fn export_as_json(graph: &PyKnowledgeGraph) -> PyResult<String> {
    graph
        .inner
        .export_as_json()
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))
}
