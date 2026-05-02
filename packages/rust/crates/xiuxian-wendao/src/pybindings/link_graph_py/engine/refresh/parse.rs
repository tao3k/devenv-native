use crate::pybindings::link_graph_py::engine::PyLinkGraphEngine;
use pyo3::PyResult;
use std::path::PathBuf;
use std::time::Instant;

use crate::link_graph::{LinkGraphCacheBuildMeta, LinkGraphIndex};

impl PyLinkGraphEngine {
    pub(super) fn parse_changed_paths(changed_paths_json: Option<&str>) -> PyResult<Vec<PathBuf>> {
        let Some(raw) = changed_paths_json
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Ok(Vec::new());
        };
        let payload = serde_json::from_str::<Vec<String>>(raw)
            .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.to_string()))?;
        Ok(payload.into_iter().map(PathBuf::from).collect())
    }

    fn apply_cache_meta(&mut self, meta: LinkGraphCacheBuildMeta) {
        self.cache_backend = meta.backend;
        self.cache_status = meta.status;
        self.cache_miss_reason = meta.miss_reason;
        self.cache_schema_version = meta.schema_version;
        self.cache_schema_fingerprint = meta.schema_fingerprint;
    }

    pub(super) fn elapsed_ms(started_at: Instant) -> f64 {
        started_at.elapsed().as_secs_f64() * 1000.0
    }

    pub(in crate::pybindings::link_graph_py::engine) fn refresh_impl(&mut self) -> PyResult<()> {
        let (inner, meta) = LinkGraphIndex::build_with_cache_with_meta(
            &self.root,
            &self.include_dirs,
            &self.excluded_dirs,
        )
        .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.clone()))?;
        self.inner = inner;
        self.apply_cache_meta(meta);
        Ok(())
    }

    pub(in crate::pybindings::link_graph_py::engine) fn refresh_with_delta_impl(
        &mut self,
        changed_paths_json: Option<&str>,
        force_full: bool,
    ) -> PyResult<()> {
        if force_full {
            return self.refresh_impl();
        }
        let changed_paths = Self::parse_changed_paths(changed_paths_json)?;
        if changed_paths.is_empty() {
            return Ok(());
        }
        self.inner
            .refresh_incremental(&changed_paths)
            .map_err(|error| pyo3::exceptions::PyValueError::new_err(error.clone()))
    }
}
