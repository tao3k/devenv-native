//! Centralized Python binding surface for `xiuxian-wendao`.
//!
//! This module owns the PyO3 boundary and the domain-specific binding modules.

/// Python bindings for dependency indexing.
#[path = "pybindings/dep_indexer_py/mod.rs"]
pub mod dep_indexer_py;
/// Python bindings for fusion recall scoring.
#[path = "pybindings/fusion_py.rs"]
pub mod fusion_py;
/// Python bindings for knowledge graph primitives.
#[path = "pybindings/graph_py/mod.rs"]
pub mod graph_py;
/// Python bindings for knowledge categories and entries.
#[path = "pybindings/knowledge_py/mod.rs"]
pub mod knowledge_py;
/// Python bindings for the LinkGraph engine surface.
#[path = "pybindings/link_graph_py/mod.rs"]
pub mod link_graph_py;
#[path = "pybindings/python_module.rs"]
mod python_module;
/// Python bindings for schema lookup helpers.
#[path = "pybindings/schema_py.rs"]
pub mod schema_py;
/// Python bindings for `KnowledgeStorage`.
#[path = "pybindings/storage_py.rs"]
pub mod storage_py;
/// Python bindings for incremental sync helpers.
#[path = "pybindings/sync_py.rs"]
pub mod sync_py;
/// Python bindings for unified symbol indexing.
#[path = "pybindings/unified_symbol_py/mod.rs"]
pub mod unified_symbol_py;

pub use dep_indexer_py::{
    PyDependencyConfig, PyDependencyIndexResult, PyDependencyIndexer, PyDependencyStats,
    PyExternalDependency, PyExternalSymbol, PySymbolIndex,
};
pub use graph_py::{
    PyEntity, PyEntityType, PyKnowledgeGraph, PyQueryIntent, PyRelation, PySkillDoc,
    extract_query_intent, invalidate_kg_cache, load_kg_from_valkey_cached,
};
pub use knowledge_py::{PyKnowledgeCategory, PyKnowledgeEntry, create_knowledge_entry};
pub use link_graph_py::{
    PyLinkGraphEngine, link_graph_stats_cache_del, link_graph_stats_cache_get,
    link_graph_stats_cache_set,
};
pub use storage_py::PyKnowledgeStorage;
pub use sync_py::{PySyncEngine, PySyncResult, compute_hash};
pub use unified_symbol_py::{PyUnifiedIndexStats, PyUnifiedSymbol, PyUnifiedSymbolIndex};
