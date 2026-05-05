use pyo3::{pyclass, pymethods};
use serde_json::json;

use crate::entity::Entity;

use super::parsers::parse_entity_type;

/// Python wrapper for Entity type.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyEntityType;

/// Python wrapper for Entity.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyEntity {
    pub(crate) inner: Entity,
}

#[pymethods]
impl PyEntity {
    #[new]
    #[pyo3(signature = (name, entity_type, description))]
    fn new(name: &str, entity_type: &str, description: &str) -> Self {
        let etype = parse_entity_type(entity_type);
        let id = format!(
            "{}:{}",
            etype.to_string().to_lowercase(),
            name.to_lowercase().replace(' ', "_")
        );
        Self {
            inner: Entity::new(id, name.to_string(), etype, description.to_string()),
        }
    }

    #[getter]
    fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
    }

    #[getter]
    fn entity_type(&self) -> String {
        self.inner.entity_type.to_string()
    }

    #[getter]
    fn description(&self) -> String {
        self.inner.description.clone()
    }

    #[getter]
    fn source(&self) -> Option<String> {
        self.inner.source.clone()
    }

    #[getter]
    fn aliases(&self) -> Vec<String> {
        self.inner.aliases.clone()
    }

    #[getter]
    fn confidence(&self) -> f32 {
        self.inner.confidence
    }

    fn to_dict(&self) -> String {
        let value = json!({
            "id": self.inner.id,
            "name": self.inner.name,
            "entity_type": self.inner.entity_type.to_string(),
            "description": self.inner.description,
            "source": self.inner.source,
            "aliases": self.inner.aliases,
            "confidence": self.inner.confidence,
        });
        serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
    }
}
