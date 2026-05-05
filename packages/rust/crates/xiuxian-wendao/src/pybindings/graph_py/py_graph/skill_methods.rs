use pyo3::PyResult;
use serde_json::{Value, json};

use crate::graph::SkillDoc;

use super::PyKnowledgeGraph;
use crate::pybindings::graph_py::PySkillDoc;

fn register_skill_docs(graph: &PyKnowledgeGraph, skill_docs: &[SkillDoc]) -> PyResult<String> {
    let result = graph
        .inner
        .register_skill_entities(skill_docs)
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))?;
    let value = json!({
        "entities_added": result.entities_added,
        "relations_added": result.relations_added,
        "status": "success",
    });
    serde_json::to_string(&value)
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))
}

pub(super) fn register_skill_entities(
    graph: &PyKnowledgeGraph,
    docs: Vec<PySkillDoc>,
) -> PyResult<String> {
    let skill_docs: Vec<SkillDoc> = docs.into_iter().map(|doc| doc.inner).collect();
    register_skill_docs(graph, &skill_docs)
}

pub(super) fn register_skill_entities_json(
    graph: &PyKnowledgeGraph,
    json_str: &str,
) -> PyResult<String> {
    let parsed: Vec<Value> = serde_json::from_str(json_str)
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))?;

    let mut skill_docs = Vec::with_capacity(parsed.len());
    for value in &parsed {
        let doc = SkillDoc {
            id: value
                .get("id")
                .and_then(|row| row.as_str())
                .unwrap_or("")
                .to_string(),
            doc_type: value
                .get("type")
                .or_else(|| value.get("doc_type"))
                .and_then(|row| row.as_str())
                .unwrap_or("")
                .to_string(),
            skill_name: value
                .get("skill_name")
                .and_then(|row| row.as_str())
                .unwrap_or("")
                .to_string(),
            tool_name: value
                .get("tool_name")
                .and_then(|row| row.as_str())
                .unwrap_or("")
                .to_string(),
            content: value
                .get("content")
                .and_then(|row| row.as_str())
                .unwrap_or("")
                .to_string(),
            routing_keywords: value
                .get("routing_keywords")
                .and_then(|row| row.as_array())
                .map(|rows| {
                    rows.iter()
                        .filter_map(|item| item.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
        };
        skill_docs.push(doc);
    }

    register_skill_docs(graph, &skill_docs)
}

pub(super) fn query_tool_relevance(
    graph: &PyKnowledgeGraph,
    query_terms: Vec<String>,
    max_hops: usize,
    limit: usize,
) -> PyResult<String> {
    let query_terms: Vec<String> = query_terms
        .into_iter()
        .map(|term| term.trim().to_string())
        .filter(|term| !term.is_empty())
        .collect();
    let results = graph
        .inner
        .query_tool_relevance(&query_terms, max_hops, limit);
    let json_arr: Vec<Value> = results
        .iter()
        .map(|(name, score)| json!([name, score]))
        .collect();
    serde_json::to_string(&json_arr)
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))
}
