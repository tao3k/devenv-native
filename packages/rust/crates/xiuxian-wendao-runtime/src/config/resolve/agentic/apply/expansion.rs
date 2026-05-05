use super::scalar::{resolve_f64, resolve_usize};
use crate::config::LinkGraphAgenticRuntimeConfig;
use crate::config::constants::{
    LINK_GRAPH_AGENTIC_EXPANSION_MAX_CANDIDATES_ENV,
    LINK_GRAPH_AGENTIC_EXPANSION_MAX_PAIRS_PER_WORKER_ENV,
    LINK_GRAPH_AGENTIC_EXPANSION_MAX_WORKERS_ENV, LINK_GRAPH_AGENTIC_EXPANSION_TIME_BUDGET_MS_ENV,
};
use serde_yaml::Value;

pub(super) fn apply_expansion_settings(
    settings: &Value,
    resolved: &mut LinkGraphAgenticRuntimeConfig,
) {
    if let Some(value) = resolve_usize(
        settings,
        "link_graph.agentic.expansion.max_workers",
        LINK_GRAPH_AGENTIC_EXPANSION_MAX_WORKERS_ENV,
    ) {
        resolved.expansion_max_workers = value;
    }

    if let Some(value) = resolve_usize(
        settings,
        "link_graph.agentic.expansion.max_candidates",
        LINK_GRAPH_AGENTIC_EXPANSION_MAX_CANDIDATES_ENV,
    ) {
        resolved.expansion_max_candidates = value;
    }

    if let Some(value) = resolve_usize(
        settings,
        "link_graph.agentic.expansion.max_pairs_per_worker",
        LINK_GRAPH_AGENTIC_EXPANSION_MAX_PAIRS_PER_WORKER_ENV,
    ) {
        resolved.expansion_max_pairs_per_worker = value;
    }

    if let Some(value) = resolve_f64(
        settings,
        "link_graph.agentic.expansion.time_budget_ms",
        LINK_GRAPH_AGENTIC_EXPANSION_TIME_BUDGET_MS_ENV,
    ) {
        resolved.expansion_time_budget_ms = value;
    }
}
