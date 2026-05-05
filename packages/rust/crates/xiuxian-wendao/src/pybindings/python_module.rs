//! Python module registration helpers for `_xiuxian_wendao`.

use pyo3::types::{PyModule, PyModuleMethods};
use pyo3::{Bound, PyResult, Python, pymodule, wrap_pyfunction};

pub(crate) fn register(py: Python<'_>, m: &Bound<PyModule>) -> PyResult<()> {
    // Knowledge types
    m.add_class::<super::knowledge_py::PyKnowledgeCategory>()?;
    m.add_class::<super::knowledge_py::PyKnowledgeEntry>()?;
    m.add_function(wrap_pyfunction!(
        super::knowledge_py::create_knowledge_entry,
        py
    )?)?;

    // Storage
    m.add_class::<super::storage_py::PyKnowledgeStorage>()?;

    // Sync
    m.add_class::<super::sync_py::PySyncEngine>()?;
    m.add_class::<super::sync_py::PySyncResult>()?;
    m.add_function(wrap_pyfunction!(super::sync_py::compute_hash, py)?)?;

    // Knowledge graph
    m.add_class::<super::graph_py::PyEntity>()?;
    m.add_class::<super::graph_py::PyRelation>()?;
    m.add_class::<super::graph_py::PyKnowledgeGraph>()?;
    m.add_class::<super::graph_py::PySkillDoc>()?;
    m.add_class::<super::graph_py::PyQueryIntent>()?;
    m.add_function(wrap_pyfunction!(super::graph_py::extract_query_intent, py)?)?;
    m.add_function(wrap_pyfunction!(super::graph_py::invalidate_kg_cache, py)?)?;
    m.add_function(wrap_pyfunction!(
        super::graph_py::load_kg_from_valkey_cached,
        py
    )?)?;
    m.add_class::<super::link_graph_py::PyLinkGraphEngine>()?;
    m.add_function(wrap_pyfunction!(
        super::link_graph_py::link_graph_stats_cache_get,
        py
    )?)?;
    m.add_function(wrap_pyfunction!(
        super::link_graph_py::link_graph_stats_cache_set,
        py
    )?)?;
    m.add_function(wrap_pyfunction!(
        super::link_graph_py::link_graph_stats_cache_del,
        py
    )?)?;

    // Fusion recall boost (LinkGraph proximity)
    m.add_function(wrap_pyfunction!(
        super::fusion_py::apply_link_graph_proximity_boost_py,
        py
    )?)?;

    // Unified symbol index
    super::unified_symbol_py::register_unified_symbol_module(m)?;

    // Schemas
    super::schema_py::register_schema_module(m)?;

    Ok(())
}

/// Python module definition - delegates to domain-specific binding modules.
#[pymodule]
fn _xiuxian_wendao(py: Python<'_>, m: &Bound<PyModule>) -> PyResult<()> {
    register(py, m)
}
