use super::scalar::{resolve_bool, resolve_f64, resolve_non_empty_string, resolve_usize};
use crate::config::LinkGraphAgenticRuntimeConfig;
use crate::config::constants::{
    LINK_GRAPH_AGENTIC_EXECUTION_AGENT_ID_ENV, LINK_GRAPH_AGENTIC_EXECUTION_EVIDENCE_PREFIX_ENV,
    LINK_GRAPH_AGENTIC_EXECUTION_IDEMPOTENCY_SCAN_LIMIT_ENV,
    LINK_GRAPH_AGENTIC_EXECUTION_PERSIST_RETRY_ATTEMPTS_ENV,
    LINK_GRAPH_AGENTIC_EXECUTION_PERSIST_SUGGESTIONS_DEFAULT_ENV,
    LINK_GRAPH_AGENTIC_EXECUTION_RELATION_ENV,
    LINK_GRAPH_AGENTIC_EXECUTION_WORKER_TIME_BUDGET_MS_ENV,
};
use serde_yaml::Value;

pub(super) fn apply_execution_settings(
    settings: &Value,
    resolved: &mut LinkGraphAgenticRuntimeConfig,
) {
    if let Some(value) = resolve_f64(
        settings,
        "link_graph.agentic.execution.worker_time_budget_ms",
        LINK_GRAPH_AGENTIC_EXECUTION_WORKER_TIME_BUDGET_MS_ENV,
    ) {
        resolved.execution_worker_time_budget_ms = value;
    }

    if let Some(value) = resolve_bool(
        settings,
        "link_graph.agentic.execution.persist_suggestions_default",
        LINK_GRAPH_AGENTIC_EXECUTION_PERSIST_SUGGESTIONS_DEFAULT_ENV,
    ) {
        resolved.execution_persist_suggestions_default = value;
    }

    if let Some(value) = resolve_usize(
        settings,
        "link_graph.agentic.execution.persist_retry_attempts",
        LINK_GRAPH_AGENTIC_EXECUTION_PERSIST_RETRY_ATTEMPTS_ENV,
    ) {
        resolved.execution_persist_retry_attempts = value;
    }

    if let Some(value) = resolve_usize(
        settings,
        "link_graph.agentic.execution.idempotency_scan_limit",
        LINK_GRAPH_AGENTIC_EXECUTION_IDEMPOTENCY_SCAN_LIMIT_ENV,
    ) {
        resolved.execution_idempotency_scan_limit = value;
    }

    if let Some(value) = resolve_non_empty_string(
        settings,
        "link_graph.agentic.execution.relation",
        LINK_GRAPH_AGENTIC_EXECUTION_RELATION_ENV,
    ) {
        resolved.execution_relation = value;
    }

    if let Some(value) = resolve_non_empty_string(
        settings,
        "link_graph.agentic.execution.agent_id",
        LINK_GRAPH_AGENTIC_EXECUTION_AGENT_ID_ENV,
    ) {
        resolved.execution_agent_id = value;
    }

    if let Some(value) = resolve_non_empty_string(
        settings,
        "link_graph.agentic.execution.evidence_prefix",
        LINK_GRAPH_AGENTIC_EXECUTION_EVIDENCE_PREFIX_ENV,
    ) {
        resolved.execution_evidence_prefix = value;
    }
}
