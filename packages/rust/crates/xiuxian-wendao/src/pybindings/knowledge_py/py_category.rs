//! `pybindings::knowledge_py::py_category` owns Wendao pybindings knowledge py py category behavior.

use pyo3::{PyResult, pyclass, pymethods};

use crate::types::KnowledgeCategory;

/// Knowledge category Python wrapper.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyKnowledgeCategory {
    pub(crate) inner: KnowledgeCategory,
}

#[pymethods]
impl PyKnowledgeCategory {
    #[classattr]
    const PATTERN: PyKnowledgeCategory = PyKnowledgeCategory {
        inner: KnowledgeCategory::Pattern,
    };

    #[classattr]
    const SOLUTION: PyKnowledgeCategory = PyKnowledgeCategory {
        inner: KnowledgeCategory::Solution,
    };

    #[classattr]
    const ERROR: PyKnowledgeCategory = PyKnowledgeCategory {
        inner: KnowledgeCategory::Error,
    };

    #[classattr]
    const TECHNIQUE: PyKnowledgeCategory = PyKnowledgeCategory {
        inner: KnowledgeCategory::Technique,
    };

    #[classattr]
    const NOTE: PyKnowledgeCategory = PyKnowledgeCategory {
        inner: KnowledgeCategory::Note,
    };

    #[classattr]
    const REFERENCE: PyKnowledgeCategory = PyKnowledgeCategory {
        inner: KnowledgeCategory::Reference,
    };

    #[classattr]
    const ARCHITECTURE: PyKnowledgeCategory = PyKnowledgeCategory {
        inner: KnowledgeCategory::Architecture,
    };

    #[classattr]
    const WORKFLOW: PyKnowledgeCategory = PyKnowledgeCategory {
        inner: KnowledgeCategory::Workflow,
    };

    #[new]
    fn new(category: &str) -> PyResult<Self> {
        match category {
            "patterns" | "pattern" => Ok(PyKnowledgeCategory {
                inner: KnowledgeCategory::Pattern,
            }),
            "solutions" | "solution" => Ok(PyKnowledgeCategory {
                inner: KnowledgeCategory::Solution,
            }),
            "errors" | "error" => Ok(PyKnowledgeCategory {
                inner: KnowledgeCategory::Error,
            }),
            "techniques" | "technique" => Ok(PyKnowledgeCategory {
                inner: KnowledgeCategory::Technique,
            }),
            "notes" | "note" => Ok(PyKnowledgeCategory {
                inner: KnowledgeCategory::Note,
            }),
            "references" | "reference" => Ok(PyKnowledgeCategory {
                inner: KnowledgeCategory::Reference,
            }),
            "architecture" => Ok(PyKnowledgeCategory {
                inner: KnowledgeCategory::Architecture,
            }),
            "workflows" | "workflow" => Ok(PyKnowledgeCategory {
                inner: KnowledgeCategory::Workflow,
            }),
            _ => Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Unknown category: {category}"
            ))),
        }
    }

    #[getter]
    fn value(&self) -> String {
        self.value_string()
    }

    fn __str__(&self) -> String {
        self.value()
    }
}

impl PyKnowledgeCategory {
    pub(crate) fn value_string(&self) -> String {
        self.inner.as_plural_str().to_string()
    }
}
