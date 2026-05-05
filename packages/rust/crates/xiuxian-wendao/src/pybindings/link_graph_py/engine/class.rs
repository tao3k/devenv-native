use crate::link_graph::{LinkGraphDirection, LinkGraphIndex};
use pyo3::{PyResult, pyclass, pymethods};
use std::path::PathBuf;

/// Python wrapper around Rust markdown link-graph index.
#[pyclass]
#[derive(Debug, Clone)]
pub struct PyLinkGraphEngine {
    pub(super) root: PathBuf,
    pub(super) include_dirs: Vec<String>,
    pub(super) excluded_dirs: Vec<String>,
    pub(super) inner: LinkGraphIndex,
    pub(super) cache_backend: String,
    pub(super) cache_status: String,
    pub(super) cache_miss_reason: Option<String>,
    pub(super) cache_schema_version: String,
    pub(super) cache_schema_fingerprint: String,
}

#[pymethods]
impl PyLinkGraphEngine {
    #[new]
    #[pyo3(signature = (notebook_dir, include_dirs=None, excluded_dirs=None))]
    fn new(
        notebook_dir: &str,
        include_dirs: Option<Vec<String>>,
        excluded_dirs: Option<Vec<String>>,
    ) -> PyResult<Self> {
        let root = PathBuf::from(notebook_dir);
        let include_dirs = include_dirs.unwrap_or_default();
        let excluded_dirs = excluded_dirs.unwrap_or_default();
        let (inner, meta) =
            LinkGraphIndex::build_with_cache_with_meta(&root, &include_dirs, &excluded_dirs)
                .map_err(pyo3::exceptions::PyValueError::new_err)?;
        Ok(Self {
            root,
            include_dirs,
            excluded_dirs,
            inner,
            cache_backend: meta.backend,
            cache_status: meta.status,
            cache_miss_reason: meta.miss_reason,
            cache_schema_version: meta.schema_version,
            cache_schema_fingerprint: meta.schema_fingerprint,
        })
    }

    /// Rebuild index from the same root path.
    fn refresh(&mut self) -> PyResult<()> {
        self.refresh_impl()
    }

    /// Incremental refresh with changed path list.
    ///
    /// `changed_paths_json` should be a JSON array of path strings.
    #[pyo3(signature = (changed_paths_json=None, force_full=false))]
    fn refresh_with_delta(
        &mut self,
        changed_paths_json: Option<&str>,
        force_full: bool,
    ) -> PyResult<()> {
        self.refresh_with_delta_impl(changed_paths_json, force_full)
    }

    /// Unified refresh planner + executor.
    ///
    /// Returns JSON object with mode/fallback and phase events:
    /// `{"mode":"delta|full|noop","changed_count":1,"force_full":false,"fallback":false,"events":[...]}`
    #[pyo3(signature = (changed_paths_json=None, force_full=false, full_rebuild_threshold=None))]
    fn refresh_plan_apply(
        &mut self,
        changed_paths_json: Option<&str>,
        force_full: bool,
        full_rebuild_threshold: Option<usize>,
    ) -> PyResult<String> {
        self.refresh_plan_apply_impl(changed_paths_json, force_full, full_rebuild_threshold)
    }

    /// Search and return parsed query plan + effective options:
    /// {"query":"...","options":{...},"results":[...]}
    #[pyo3(signature = (query, limit=20, options_json=None))]
    fn search_planned(
        &self,
        query: &str,
        limit: usize,
        options_json: Option<&str>,
    ) -> PyResult<String> {
        self.run_search_planned_impl(query, limit, options_json)
    }

    /// Fetch neighbors around a note.
    #[pyo3(signature = (stem, direction="both", hops=1, limit=50))]
    fn neighbors(
        &self,
        stem: &str,
        direction: &str,
        hops: usize,
        limit: usize,
    ) -> PyResult<String> {
        self.neighbors_impl(stem, LinkGraphDirection::from_alias(direction), hops, limit)
    }

    /// Fetch related notes through bidirectional traversal.
    #[pyo3(signature = (stem, max_distance=2, limit=20))]
    fn related(&self, stem: &str, max_distance: usize, limit: usize) -> PyResult<String> {
        self.related_impl(stem, max_distance, limit)
    }

    /// Fetch note metadata.
    fn metadata(&self, stem: &str) -> PyResult<String> {
        self.metadata_impl(stem)
    }

    /// Return table-of-contents rows.
    #[pyo3(signature = (limit=1000))]
    fn toc(&self, limit: usize) -> PyResult<String> {
        self.toc_impl(limit)
    }

    /// Return graph stats.
    fn stats(&self) -> PyResult<String> {
        self.stats_impl()
    }

    /// Return cache schema version/fingerprint used by Valkey snapshot payloads.
    fn cache_schema_info(&self) -> PyResult<String> {
        self.cache_schema_info_impl()
    }

    /// Generate GRAG narrative hard-prompt from hits JSON.
    #[staticmethod]
    fn narrate_hits_json(hits_json: &str) -> PyResult<String> {
        Self::narrate_hits_json_impl(hits_json)
    }
}
