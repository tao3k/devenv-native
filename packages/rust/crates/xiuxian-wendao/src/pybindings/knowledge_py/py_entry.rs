//! `pybindings::knowledge_py::py_entry` owns Wendao pybindings knowledge py py entry behavior.

use pyo3::{pyclass, pymethods};
use serde_json::{json, to_string};

use crate::types::KnowledgeEntry;

use super::PyKnowledgeCategory;

/// Knowledge entry Python wrapper.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyKnowledgeEntry {
    pub(crate) inner: KnowledgeEntry,
}

#[pymethods]
impl PyKnowledgeEntry {
    #[new]
    #[pyo3(signature = (id, title, content, category))]
    fn new(id: &str, title: &str, content: &str, category: &PyKnowledgeCategory) -> Self {
        Self {
            inner: KnowledgeEntry::new(
                id.to_string(),
                title.to_string(),
                content.to_string(),
                category.inner,
            ),
        }
    }

    #[getter]
    fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[getter]
    fn title(&self) -> String {
        self.inner.title.clone()
    }

    #[getter]
    fn content(&self) -> String {
        self.inner.content.clone()
    }

    #[getter]
    fn category(&self) -> PyKnowledgeCategory {
        PyKnowledgeCategory {
            inner: self.inner.category,
        }
    }

    #[getter]
    fn tags(&self) -> Vec<String> {
        self.inner.tags.clone()
    }

    #[getter]
    fn source(&self) -> Option<String> {
        self.inner.source.clone()
    }

    #[getter]
    fn version(&self) -> i32 {
        self.inner.version
    }

    #[setter]
    fn set_tags(&mut self, tags: Vec<String>) {
        self.inner.tags = tags;
    }

    #[setter]
    fn set_source(&mut self, source: Option<String>) {
        self.inner.source = source;
    }

    fn add_tag(&mut self, tag: String) {
        self.inner.add_tag(tag);
    }

    fn to_dict(&self) -> String {
        let value = json!({
            "id": self.inner.id,
            "title": self.inner.title,
            "content": self.inner.content,
            "category": self.category().value_string(),
            "tags": self.inner.tags,
            "source": self.inner.source,
            "version": self.inner.version,
        });
        to_string(&value).unwrap_or_else(|_| "{}".to_string())
    }
}
