//! Runtime configuration model and resolver boundary for Wendao host behavior.

mod constants;
#[cfg(feature = "duckdb-config")]
mod duckdb;
mod memory;
mod models;
mod resolve;
mod retrieval;
#[cfg(test)]
#[path = "../../tests/unit/config/test_support.rs"]
pub(crate) mod test_support;

pub use constants::{
    DEFAULT_LINK_GRAPH_CANDIDATE_MULTIPLIER, DEFAULT_LINK_GRAPH_COACTIVATION_ALPHA_SCALE,
    DEFAULT_LINK_GRAPH_COACTIVATION_ENABLED, DEFAULT_LINK_GRAPH_COACTIVATION_HOP_DECAY_SCALE,
    DEFAULT_LINK_GRAPH_COACTIVATION_MAX_HOPS,
    DEFAULT_LINK_GRAPH_COACTIVATION_MAX_NEIGHBORS_PER_DIRECTION,
    DEFAULT_LINK_GRAPH_COACTIVATION_TOUCH_QUEUE_DEPTH, DEFAULT_LINK_GRAPH_HYBRID_MIN_HITS,
    DEFAULT_LINK_GRAPH_HYBRID_MIN_TOP_SCORE, DEFAULT_LINK_GRAPH_MAX_SOURCES,
    DEFAULT_LINK_GRAPH_RELATED_MAX_CANDIDATES, DEFAULT_LINK_GRAPH_RELATED_MAX_PARTITIONS,
    DEFAULT_LINK_GRAPH_RELATED_TIME_BUDGET_MS, DEFAULT_LINK_GRAPH_ROWS_PER_SOURCE,
    DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX, LINK_GRAPH_CANDIDATE_MULTIPLIER_ENV,
    LINK_GRAPH_HYBRID_MIN_HITS_ENV, LINK_GRAPH_HYBRID_MIN_TOP_SCORE_ENV,
    LINK_GRAPH_MAX_SOURCES_ENV, LINK_GRAPH_ROWS_PER_SOURCE_ENV,
};
#[cfg(feature = "duckdb-config")]
pub use duckdb::{
    DEFAULT_SEARCH_DUCKDB_DATABASE_PATH, DEFAULT_SEARCH_DUCKDB_MATERIALIZE_THRESHOLD_ROWS,
    DEFAULT_SEARCH_DUCKDB_PARQUET_METADATA_CACHE, DEFAULT_SEARCH_DUCKDB_PREFER_VIRTUAL_ARROW,
    DEFAULT_SEARCH_DUCKDB_PRESERVE_INSERTION_ORDER, DEFAULT_SEARCH_DUCKDB_THREADS,
    DuckDbDatabasePath, SearchDuckDbExecutionConfig, SearchDuckDbRuntimeConfig,
    default_search_duckdb_temp_directory, resolve_search_duckdb_runtime_with_settings,
};
pub use memory::{
    DEFAULT_MEMORY_JULIA_COMPUTE_BASE_URL, DEFAULT_MEMORY_JULIA_COMPUTE_CALIBRATION_ROUTE,
    DEFAULT_MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_ROUTE,
    DEFAULT_MEMORY_JULIA_COMPUTE_GATE_SCORE_ROUTE, DEFAULT_MEMORY_JULIA_COMPUTE_PLAN_TUNING_ROUTE,
    DEFAULT_MEMORY_JULIA_COMPUTE_PLUGIN_ID, DEFAULT_MEMORY_JULIA_COMPUTE_SCHEMA_VERSION,
    DEFAULT_MEMORY_JULIA_COMPUTE_TIMEOUT_SECS, MemoryJuliaComputeFallbackMode,
    MemoryJuliaComputePluginId, MemoryJuliaComputeRoutesRuntimeConfig,
    MemoryJuliaComputeRuntimeConfig, MemoryJuliaComputeServiceMode, MemoryJuliaComputeTimeoutSecs,
    resolve_memory_julia_compute_runtime_with_settings,
};
pub use models::{
    LinkGraphAgenticRuntimeConfig, LinkGraphCacheRuntimeConfig, LinkGraphCoactivationRuntimeConfig,
    LinkGraphIndexRuntimeConfig, LinkGraphRelatedRuntimeConfig,
};
pub use resolve::{
    resolve_link_graph_agentic_runtime_with_settings,
    resolve_link_graph_cache_runtime_with_settings,
    resolve_link_graph_coactivation_runtime_with_settings,
    resolve_link_graph_index_runtime_with_settings,
    resolve_link_graph_related_runtime_with_settings,
};
pub use retrieval::{
    LinkGraphRetrievalBaseRuntimeConfig, LinkGraphSemanticIgnitionBackend,
    LinkGraphSemanticIgnitionRuntimeConfig, apply_semantic_ignition_runtime_config,
    resolve_link_graph_retrieval_base_runtime_with_settings,
};
