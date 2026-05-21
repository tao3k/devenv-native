//! `pybindings::dep_indexer_py::config` owns Wendao pybindings dep indexer py config behavior.

use pyo3::{pyclass, pymethods};

use crate::dependency_indexer::{ConfigExternalDependency, DependencyBuildConfig};

/// Python wrapper for `ConfigExternalDependency`.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyExternalDependency {
    pub(crate) inner: ConfigExternalDependency,
}

#[pymethods]
impl PyExternalDependency {
    #[new]
    fn new(pkg_type: &str, registry: Option<&str>, manifests: Vec<String>) -> Self {
        Self {
            inner: ConfigExternalDependency {
                pkg_type: pkg_type.to_string(),
                registry: registry.map(str::to_string),
                manifests,
            },
        }
    }

    #[getter]
    fn pkg_type(&self) -> String {
        self.inner.pkg_type.clone()
    }

    #[getter]
    fn registry(&self) -> Option<String> {
        self.inner.registry.clone()
    }

    #[getter]
    fn manifests(&self) -> Vec<String> {
        self.inner.manifests.clone()
    }

    fn to_dict(&self) -> String {
        let value = serde_json::json!({
            "pkg_type": self.inner.pkg_type,
            "registry": self.inner.registry,
            "manifests": self.inner.manifests,
        });
        serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Python wrapper for `DependencyBuildConfig`.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyDependencyConfig {
    inner: DependencyBuildConfig,
}

#[pymethods]
impl PyDependencyConfig {
    #[new]
    #[pyo3(signature = (path))]
    fn new(path: &str) -> Self {
        Self {
            inner: DependencyBuildConfig::load(path),
        }
    }

    #[getter]
    fn manifests(&self) -> Vec<PyExternalDependency> {
        self.inner
            .manifests
            .iter()
            .map(|e| PyExternalDependency { inner: e.clone() })
            .collect()
    }

    /// Load config from a TOML file path.
    #[staticmethod]
    #[pyo3(signature = (path))]
    fn load(path: &str) -> Self {
        Self::new(path)
    }

    fn to_dict(&self) -> String {
        let manifests: Vec<serde_json::Value> = self
            .inner
            .manifests
            .iter()
            .map(|e| {
                serde_json::json!({
                    "pkg_type": e.pkg_type,
                    "manifests": e.manifests,
                })
            })
            .collect();
        let value = serde_json::json!({
            "manifests": manifests,
        });
        serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string())
    }
}
