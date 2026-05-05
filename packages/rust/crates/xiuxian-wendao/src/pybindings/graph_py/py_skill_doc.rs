use pyo3::{pyclass, pymethods};

use crate::graph::SkillDoc;

/// Python wrapper for `SkillDoc` (used by `register_skill_entities`).
#[pyclass]
#[derive(Debug, Clone)]
pub struct PySkillDoc {
    pub(crate) inner: SkillDoc,
}

#[pymethods]
impl PySkillDoc {
    #[new]
    #[pyo3(signature = (id, doc_type, skill_name, tool_name, content, routing_keywords))]
    fn new(
        id: &str,
        doc_type: &str,
        skill_name: &str,
        tool_name: &str,
        content: &str,
        routing_keywords: Vec<String>,
    ) -> Self {
        Self {
            inner: SkillDoc {
                id: id.to_string(),
                doc_type: doc_type.to_string(),
                skill_name: skill_name.to_string(),
                tool_name: tool_name.to_string(),
                content: content.to_string(),
                routing_keywords,
            },
        }
    }
}
