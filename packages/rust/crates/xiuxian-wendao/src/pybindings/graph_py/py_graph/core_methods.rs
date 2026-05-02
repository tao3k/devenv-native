use pyo3::PyResult;
use serde_json::{Value, json};

use super::PyKnowledgeGraph;
use crate::pybindings::graph_py::parsers::parse_persisted_relation_type;
use crate::pybindings::graph_py::{PyEntity, PyRelation};

pub(super) fn add_entity(graph: &PyKnowledgeGraph, entity: PyEntity) -> PyResult<()> {
    graph
        .inner
        .add_entity(entity.inner)
        .map(|_| ())
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))
}

pub(super) fn add_relation(graph: &PyKnowledgeGraph, relation: PyRelation) -> PyResult<()> {
    graph
        .inner
        .add_relation(relation.inner)
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))
}

pub(super) fn search_entities(graph: &PyKnowledgeGraph, query: &str, limit: i32) -> Vec<PyEntity> {
    graph
        .inner
        .search_entities(query, limit)
        .into_iter()
        .map(|entity| PyEntity { inner: entity })
        .collect()
}

pub(super) fn get_entity(graph: &PyKnowledgeGraph, entity_id: &str) -> Option<PyEntity> {
    graph
        .inner
        .get_entity(entity_id)
        .map(|entity| PyEntity { inner: entity })
}

pub(super) fn get_entity_by_name(graph: &PyKnowledgeGraph, name: &str) -> Option<PyEntity> {
    graph
        .inner
        .get_entity_by_name(name)
        .map(|entity| PyEntity { inner: entity })
}

pub(super) fn get_relations(
    graph: &PyKnowledgeGraph,
    entity_name: Option<&str>,
    relation_type: Option<&str>,
) -> Vec<PyRelation> {
    let relation_type = relation_type.map(parse_persisted_relation_type);
    graph
        .inner
        .get_relations(entity_name, relation_type)
        .into_iter()
        .map(|relation| PyRelation { inner: relation })
        .collect()
}

pub(super) fn multi_hop_search(
    graph: &PyKnowledgeGraph,
    start_name: &str,
    max_hops: usize,
) -> Vec<PyEntity> {
    graph
        .inner
        .multi_hop_search(start_name, max_hops)
        .into_iter()
        .map(|entity| PyEntity { inner: entity })
        .collect()
}

pub(super) fn get_stats(graph: &PyKnowledgeGraph) -> String {
    let stats = graph.inner.get_stats();
    let value = json!({
        "total_entities": stats.total_entities,
        "total_relations": stats.total_relations,
        "entities_by_type": stats.entities_by_type,
        "relations_by_type": stats.relations_by_type,
    });
    serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
}

pub(super) fn clear(graph: &mut PyKnowledgeGraph) {
    graph.inner.clear();
}

pub(super) fn get_all_entities_json(graph: &PyKnowledgeGraph) -> PyResult<String> {
    let entities = graph.inner.get_all_entities();
    let entities_json: Vec<Value> = entities
        .into_iter()
        .map(|entity| {
            json!({
                "id": entity.id,
                "name": entity.name,
                "entity_type": entity.entity_type.to_string(),
                "description": entity.description,
                "source": entity.source,
                "aliases": entity.aliases,
                "confidence": entity.confidence,
            })
        })
        .collect();
    serde_json::to_string(&entities_json)
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))
}

pub(super) fn get_all_relations_json(graph: &PyKnowledgeGraph) -> PyResult<String> {
    let relations = graph.inner.get_all_relations();
    let relations_json: Vec<Value> = relations
        .into_iter()
        .map(|relation| {
            json!({
                "id": relation.id,
                "source": relation.source,
                "target": relation.target,
                "relation_type": relation.relation_type.to_string(),
                "description": relation.description,
                "source_doc": relation.source_doc,
                "confidence": relation.confidence,
            })
        })
        .collect();
    serde_json::to_string(&relations_json)
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))
}
