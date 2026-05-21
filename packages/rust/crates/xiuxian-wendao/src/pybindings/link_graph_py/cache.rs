//! `pybindings::link_graph_py::cache` owns Wendao pybindings link graph py cache behavior.

use pyo3::{PyResult, pyfunction};

use crate::link_graph::{valkey_stats_cache_del, valkey_stats_cache_get, valkey_stats_cache_set};

/// Read `LinkGraph` stats cache payload from `Valkey`.
///
/// Returns JSON object string when cache is valid and fresh, otherwise `None`.
///
/// # Errors
///
/// Returns an error when underlying cache runtime validation or `Valkey` operations fail.
#[pyfunction]
pub fn link_graph_stats_cache_get(source_key: &str, ttl_sec: f64) -> PyResult<Option<String>> {
    valkey_stats_cache_get(source_key, ttl_sec).map_err(pyo3::exceptions::PyValueError::new_err)
}

/// Write `LinkGraph` stats cache payload to `Valkey` with TTL.
///
/// `stats_json` must be a JSON object with:
/// `total_notes`, `orphans`, `links_in_graph`, `nodes_in_graph`.
///
/// # Errors
///
/// Returns an error when payload validation fails or `Valkey` operations fail.
#[pyfunction]
pub fn link_graph_stats_cache_set(
    source_key: &str,
    stats_json: &str,
    ttl_sec: f64,
) -> PyResult<()> {
    valkey_stats_cache_set(source_key, stats_json, ttl_sec)
        .map_err(pyo3::exceptions::PyValueError::new_err)
}

/// Delete `LinkGraph` stats cache payload from `Valkey`.
///
/// # Errors
///
/// Returns an error when underlying cache runtime validation or `Valkey` operations fail.
#[pyfunction]
pub fn link_graph_stats_cache_del(source_key: &str) -> PyResult<()> {
    valkey_stats_cache_del(source_key).map_err(pyo3::exceptions::PyValueError::new_err)
}
