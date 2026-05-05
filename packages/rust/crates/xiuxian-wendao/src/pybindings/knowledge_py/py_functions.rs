use pyo3::pyfunction;

use crate::types::KnowledgeEntry;

use super::{PyKnowledgeCategory, PyKnowledgeEntry};

/// Create a knowledge entry from Python.
#[pyfunction]
#[pyo3(signature = (title, content, category, tags, source))]
pub fn create_knowledge_entry(
    title: &str,
    content: &str,
    category: &PyKnowledgeCategory,
    tags: Vec<String>,
    source: Option<&str>,
) -> PyKnowledgeEntry {
    let entry = KnowledgeEntry::new(
        uuid::Uuid::new_v4().to_string(),
        title.to_string(),
        content.to_string(),
        category.inner,
    )
    .with_tags(tags)
    .with_source(source.map(std::string::ToString::to_string));

    PyKnowledgeEntry { inner: entry }
}
