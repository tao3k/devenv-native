//! `pybindings::graph_py::py_relation` owns Wendao pybindings graph py py relation behavior.

use pyo3::{pyclass, pymethods};
use serde_json::json;

use crate::entity::Relation;

use super::parsers::parse_persisted_relation_type;

/// Python wrapper for Relation.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyRelation {
    pub(crate) inner: Relation,
}

#[pymethods]
impl PyRelation {
    #[new]
    #[pyo3(signature = (source, target, relation_type, description))]
    fn new(source: &str, target: &str, relation_type: &str, description: &str) -> Self {
        let rtype = parse_persisted_relation_type(relation_type);
        Self {
            inner: Relation::new(
                source.to_string(),
                target.to_string(),
                rtype,
                description.to_string(),
            ),
        }
    }

    #[getter]
    fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[getter]
    fn source(&self) -> String {
        self.inner.source.clone()
    }

    #[getter]
    fn target(&self) -> String {
        self.inner.target.clone()
    }

    #[getter]
    fn relation_type(&self) -> String {
        self.inner.relation_type.to_string()
    }

    #[getter]
    fn description(&self) -> String {
        self.inner.description.clone()
    }

    #[getter]
    fn confidence(&self) -> f32 {
        self.inner.confidence
    }

    fn to_dict(&self) -> String {
        let value = json!({
            "id": self.inner.id,
            "source": self.inner.source,
            "target": self.inner.target,
            "relation_type": self.inner.relation_type.to_string(),
            "description": self.inner.description,
            "source_doc": self.inner.source_doc,
            "confidence": self.inner.confidence,
        });
        serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
    }
}
