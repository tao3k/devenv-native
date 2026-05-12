//! `pybindings::graph_py::py_graph::knowledge_graph` owns Wendao graph py py graph knowledge graph behavior.

use pyo3::{PyResult, pyclass, pymethods};

use crate::graph::KnowledgeGraph;

use super::{core_methods, persistence_methods, skill_methods};
use crate::pybindings::graph_py::{PyEntity, PyRelation, PySkillDoc};

/// Python wrapper for `KnowledgeGraph`.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyKnowledgeGraph {
    pub(crate) inner: KnowledgeGraph,
}

#[pymethods]
impl PyKnowledgeGraph {
    #[new]
    fn new() -> Self {
        Self {
            inner: KnowledgeGraph::new(),
        }
    }

    fn add_entity(&self, entity: PyEntity) -> PyResult<()> {
        core_methods::add_entity(self, entity)
    }

    fn add_relation(&self, relation: PyRelation) -> PyResult<()> {
        core_methods::add_relation(self, relation)
    }

    fn search_entities(&self, query: &str, limit: i32) -> Vec<PyEntity> {
        core_methods::search_entities(self, query, limit)
    }

    fn get_entity(&self, entity_id: &str) -> Option<PyEntity> {
        core_methods::get_entity(self, entity_id)
    }

    fn get_entity_by_name(&self, name: &str) -> Option<PyEntity> {
        core_methods::get_entity_by_name(self, name)
    }

    fn get_relations(
        &self,
        entity_name: Option<&str>,
        relation_type: Option<&str>,
    ) -> Vec<PyRelation> {
        core_methods::get_relations(self, entity_name, relation_type)
    }

    fn multi_hop_search(&self, start_name: &str, max_hops: usize) -> Vec<PyEntity> {
        core_methods::multi_hop_search(self, start_name, max_hops)
    }

    fn get_stats(&self) -> String {
        core_methods::get_stats(self)
    }

    fn clear(&mut self) {
        core_methods::clear(self);
    }

    fn get_all_entities_json(&self) -> PyResult<String> {
        core_methods::get_all_entities_json(self)
    }

    fn get_all_relations_json(&self) -> PyResult<String> {
        core_methods::get_all_relations_json(self)
    }

    fn save_to_file(&self, path: &str) -> PyResult<()> {
        persistence_methods::save_to_file(self, path)
    }

    fn load_from_file(&mut self, path: &str) -> PyResult<()> {
        persistence_methods::load_from_file(self, path)
    }

    /// Save the graph snapshot to Valkey using `scope_key`.
    ///
    /// Invalidates the KG cache for this scope so subsequent loads see fresh data.
    #[pyo3(signature = (scope_key, dimension=1024))]
    fn save_to_valkey(&self, scope_key: &str, dimension: usize) -> PyResult<()> {
        persistence_methods::save_to_valkey(self, scope_key, dimension)
    }

    /// Load the graph snapshot from Valkey by `scope_key`.
    fn load_from_valkey(&mut self, scope_key: &str) -> PyResult<()> {
        persistence_methods::load_from_valkey(self, scope_key)
    }

    fn export_as_json(&self) -> PyResult<String> {
        persistence_methods::export_as_json(self)
    }

    /// Batch-register skill docs as entities and relations in the graph.
    fn register_skill_entities(&self, docs: Vec<PySkillDoc>) -> PyResult<String> {
        skill_methods::register_skill_entities(self, docs)
    }

    /// Register skill entities from a JSON string (convenience method).
    fn register_skill_entities_json(&self, json_str: &str) -> PyResult<String> {
        skill_methods::register_skill_entities_json(self, json_str)
    }

    /// Query-time tool relevance scoring via `KnowledgeGraph` traversal.
    #[pyo3(signature = (query_terms, max_hops = 2, limit = 10))]
    fn query_tool_relevance(
        &self,
        query_terms: Vec<String>,
        max_hops: usize,
        limit: usize,
    ) -> PyResult<String> {
        skill_methods::query_tool_relevance(self, query_terms, max_hops, limit)
    }
}
