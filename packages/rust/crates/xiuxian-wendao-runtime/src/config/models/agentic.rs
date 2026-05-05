//! Agentic runtime configuration records for Wendao graph expansion behavior.

use crate::config::constants::{
    DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_AGENT_ID,
    DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_EVIDENCE_PREFIX,
    DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_IDEMPOTENCY_SCAN_LIMIT,
    DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_PERSIST_RETRY_ATTEMPTS,
    DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_PERSIST_SUGGESTIONS_DEFAULT,
    DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_RELATION,
    DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_WORKER_TIME_BUDGET_MS,
    DEFAULT_LINK_GRAPH_AGENTIC_EXPANSION_MAX_CANDIDATES,
    DEFAULT_LINK_GRAPH_AGENTIC_EXPANSION_MAX_PAIRS_PER_WORKER,
    DEFAULT_LINK_GRAPH_AGENTIC_EXPANSION_MAX_WORKERS,
    DEFAULT_LINK_GRAPH_AGENTIC_EXPANSION_TIME_BUDGET_MS,
    DEFAULT_LINK_GRAPH_AGENTIC_SEARCH_PROVISIONAL_LIMIT,
    DEFAULT_LINK_GRAPH_AGENTIC_SUGGESTED_LINK_MAX_ENTRIES,
};

/// Resolved runtime controls for link-graph agentic workflows.
#[derive(Debug, Clone)]
pub struct LinkGraphAgenticRuntimeConfig {
    /// Maximum number of suggested-link entries retained in storage.
    pub suggested_link_max_entries: usize,
    /// Optional TTL, in seconds, applied to suggested-link records.
    pub suggested_link_ttl_seconds: Option<u64>,
    /// Default inclusion behavior for provisional agentic search rows.
    pub search_include_provisional_default: bool,
    /// Default limit applied to provisional search rows.
    pub search_provisional_limit: usize,
    /// Maximum number of expansion workers.
    pub expansion_max_workers: usize,
    /// Maximum number of expansion candidates.
    pub expansion_max_candidates: usize,
    /// Maximum number of candidate pairs assigned per worker.
    pub expansion_max_pairs_per_worker: usize,
    /// Expansion time budget, in milliseconds.
    pub expansion_time_budget_ms: f64,
    /// Execution worker time budget, in milliseconds.
    pub execution_worker_time_budget_ms: f64,
    /// Default persistence behavior for suggested links during execution.
    pub execution_persist_suggestions_default: bool,
    /// Retry attempts for persistence during execution.
    pub execution_persist_retry_attempts: usize,
    /// Scan limit applied to idempotency checks.
    pub execution_idempotency_scan_limit: usize,
    /// Relation used when persisting executed suggestions.
    pub execution_relation: String,
    /// Default agent id used for execution records.
    pub execution_agent_id: String,
    /// Prefix applied to generated evidence text.
    pub execution_evidence_prefix: String,
}

impl Default for LinkGraphAgenticRuntimeConfig {
    fn default() -> Self {
        Self {
            suggested_link_max_entries: DEFAULT_LINK_GRAPH_AGENTIC_SUGGESTED_LINK_MAX_ENTRIES,
            suggested_link_ttl_seconds: None,
            search_include_provisional_default: false,
            search_provisional_limit: DEFAULT_LINK_GRAPH_AGENTIC_SEARCH_PROVISIONAL_LIMIT,
            expansion_max_workers: DEFAULT_LINK_GRAPH_AGENTIC_EXPANSION_MAX_WORKERS,
            expansion_max_candidates: DEFAULT_LINK_GRAPH_AGENTIC_EXPANSION_MAX_CANDIDATES,
            expansion_max_pairs_per_worker:
                DEFAULT_LINK_GRAPH_AGENTIC_EXPANSION_MAX_PAIRS_PER_WORKER,
            expansion_time_budget_ms: DEFAULT_LINK_GRAPH_AGENTIC_EXPANSION_TIME_BUDGET_MS,
            execution_worker_time_budget_ms:
                DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_WORKER_TIME_BUDGET_MS,
            execution_persist_suggestions_default:
                DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_PERSIST_SUGGESTIONS_DEFAULT,
            execution_persist_retry_attempts:
                DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_PERSIST_RETRY_ATTEMPTS,
            execution_idempotency_scan_limit:
                DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_IDEMPOTENCY_SCAN_LIMIT,
            execution_relation: DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_RELATION.to_string(),
            execution_agent_id: DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_AGENT_ID.to_string(),
            execution_evidence_prefix: DEFAULT_LINK_GRAPH_AGENTIC_EXECUTION_EVIDENCE_PREFIX
                .to_string(),
        }
    }
}
