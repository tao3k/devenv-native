pub(crate) mod document_extract_artifacts;
mod link_graph;
mod perf;

#[cfg(feature = "duckdb")]
pub(crate) use link_graph::build_link_graph_fixture;
pub(crate) use link_graph::{
    RELATED_LIMIT, RELATED_MAX_DISTANCE, build_index, default_ppr_options, env_f64, env_u64,
    env_usize, seed_set,
};
#[cfg(feature = "duckdb")]
pub(crate) use perf::PerfReport;
#[cfg(feature = "performance-stress")]
pub(crate) use perf::run_async_budget;
pub(crate) use perf::{PerfBudget, PerfRunConfig, assert_perf_budget, run_sync_budget};
