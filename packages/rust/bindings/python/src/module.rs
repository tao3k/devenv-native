use pyo3::exceptions::PyValueError;
use pyo3::types::{PyModule, PyModuleMethods};
use pyo3::{Bound, PyErr, PyResult, Python, pyfunction, pymodule, wrap_pyfunction};
use xiuxian_wendao::pybindings::{
    PyEntity, PyKnowledgeCategory, PyKnowledgeEntry, PyKnowledgeGraph, PyKnowledgeStorage,
    PyLinkGraphEngine, PyQueryIntent, PyRelation, PySkillDoc, PySyncEngine, PySyncResult,
    create_knowledge_entry, dep_indexer_py::register_dependency_indexer_module,
    extract_query_intent, fusion_py::apply_link_graph_proximity_boost_py, invalidate_kg_cache,
    link_graph_stats_cache_del, link_graph_stats_cache_get, link_graph_stats_cache_set,
    load_kg_from_valkey_cached, unified_symbol_py::register_unified_symbol_module,
};
use xiuxian_wendao::schemas;

#[pyfunction(name = "get_schema")]
pub(crate) fn py_get_schema(name: &str) -> PyResult<String> {
    schemas::get_schema(name)
        .map(std::string::ToString::to_string)
        .ok_or_else(|| PyErr::new::<PyValueError, _>(format!("Unknown schema name: {name}")))
}

pub(crate) fn register_module(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyKnowledgeCategory>()?;
    m.add_class::<PyKnowledgeEntry>()?;
    m.add_function(wrap_pyfunction!(create_knowledge_entry, py)?)?;

    m.add_class::<PyKnowledgeStorage>()?;

    m.add_class::<PySyncEngine>()?;
    m.add_class::<PySyncResult>()?;
    m.add_function(wrap_pyfunction!(
        xiuxian_wendao::pybindings::compute_hash,
        py
    )?)?;

    m.add_class::<PyEntity>()?;
    m.add_class::<PyRelation>()?;
    m.add_class::<PyKnowledgeGraph>()?;
    m.add_class::<PySkillDoc>()?;
    m.add_class::<PyQueryIntent>()?;
    m.add_function(wrap_pyfunction!(extract_query_intent, py)?)?;
    m.add_function(wrap_pyfunction!(invalidate_kg_cache, py)?)?;
    m.add_function(wrap_pyfunction!(load_kg_from_valkey_cached, py)?)?;

    m.add_class::<PyLinkGraphEngine>()?;
    m.add_function(wrap_pyfunction!(link_graph_stats_cache_get, py)?)?;
    m.add_function(wrap_pyfunction!(link_graph_stats_cache_set, py)?)?;
    m.add_function(wrap_pyfunction!(link_graph_stats_cache_del, py)?)?;
    m.add_function(wrap_pyfunction!(apply_link_graph_proximity_boost_py, py)?)?;

    register_dependency_indexer_module(m)?;
    register_unified_symbol_module(m)?;
    m.add_function(wrap_pyfunction!(py_get_schema, py)?)?;
    Ok(())
}

/// Python module initialization.
#[pymodule]
pub(crate) fn xiuxian_core_rs(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    register_module(py, m)
}
