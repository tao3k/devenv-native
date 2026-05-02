use pyo3::{pyclass, pyfunction, pymethods};
use serde_json::json;

use crate::graph::{QueryIntent, extract_intent};

/// Python wrapper for `QueryIntent`.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyQueryIntent {
    pub(crate) inner: QueryIntent,
}

#[pymethods]
impl PyQueryIntent {
    /// Primary action verb (e.g. "search", "commit", "create"). None if not detected.
    #[getter]
    fn action(&self) -> Option<String> {
        self.inner.action.clone()
    }

    /// Target domain or object (e.g. "git", "knowledge", "code"). None if not detected.
    #[getter]
    fn target(&self) -> Option<String> {
        self.inner.target.clone()
    }

    /// Context qualifiers (remaining significant tokens).
    #[getter]
    fn context(&self) -> Vec<String> {
        self.inner.context.clone()
    }

    /// All significant keywords (stop-words removed).
    #[getter]
    fn keywords(&self) -> Vec<String> {
        self.inner.keywords.clone()
    }

    /// Original query, lower-cased and trimmed.
    #[getter]
    fn normalized_query(&self) -> String {
        self.inner.normalized_query.clone()
    }

    fn to_dict(&self) -> String {
        let value = json!({
            "action": self.inner.action,
            "target": self.inner.target,
            "context": self.inner.context,
            "keywords": self.inner.keywords,
            "normalized_query": self.inner.normalized_query,
        });
        serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Extract structured query intent from a natural-language query string.
///
/// Returns a `PyQueryIntent` with action, target, context, and keywords.
#[pyfunction]
#[must_use]
pub fn extract_query_intent(query: &str) -> PyQueryIntent {
    PyQueryIntent {
        inner: extract_intent(query),
    }
}
