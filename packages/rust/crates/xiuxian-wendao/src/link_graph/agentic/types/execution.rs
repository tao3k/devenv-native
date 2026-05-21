//! `link_graph::agentic::types::execution` owns Wendao agentic types execution behavior.

use super::config::LinkGraphAgenticExecutionConfig;
use super::plan::LinkGraphAgenticExpansionPlan;
use serde::{Deserialize, Serialize};

/// Runtime telemetry for one execution worker partition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct LinkGraphAgenticWorkerPhase {
    /// Stable phase id (`worker.prepare|worker.dedupe|worker.persist|worker.total`).
    pub phase: String,
    /// Phase elapsed wall-clock duration in milliseconds.
    pub duration_ms: f64,
    /// Count aligned to the phase domain (for example processed pairs/attempts).
    pub item_count: usize,
}

/// Runtime telemetry for one execution worker partition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphAgenticWorkerExecution {
    /// Worker index from planner output.
    pub worker_id: usize,
    /// Pair budget assigned to this worker.
    pub pair_budget: usize,
    /// Number of pairs processed by this worker.
    pub processed_pairs: usize,
    /// Number of proposal rows prepared by this worker.
    pub prepared_proposals: usize,
    /// Number of proposal rows persisted to Valkey.
    pub persisted_proposals: usize,
    /// Number of proposal rows skipped due idempotency dedupe.
    pub skipped_duplicates: usize,
    /// Number of proposal rows that failed to persist.
    pub failed_proposals: usize,
    /// Number of persistence attempts (including retries).
    pub persist_attempts: usize,
    /// Worker phase timeline breakdown.
    pub phases: Vec<LinkGraphAgenticWorkerPhase>,
    /// Whether this worker ended due runtime budget exhaustion.
    pub timed_out: bool,
    /// Worker elapsed wall-clock duration in milliseconds.
    pub elapsed_ms: f64,
    /// Estimated prompt token cost (placeholder until model dispatch phase).
    pub estimated_prompt_tokens: u64,
    /// Estimated completion token cost (placeholder until model dispatch phase).
    pub estimated_completion_tokens: u64,
}

/// End-to-end bounded expansion execution result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphAgenticExecutionResult {
    /// Optional query used to narrow candidate notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Effective execution config after normalization.
    pub config: LinkGraphAgenticExecutionConfig,
    /// Planner output consumed by worker execution.
    pub plan: LinkGraphAgenticExpansionPlan,
    /// Worker-level execution telemetry rows.
    pub worker_runs: Vec<LinkGraphAgenticWorkerExecution>,
    /// Number of proposal rows prepared across all workers.
    pub prepared_proposals: usize,
    /// Number of proposal rows persisted across all workers.
    pub persisted_proposals: usize,
    /// Number of proposal rows skipped due idempotency dedupe.
    pub skipped_duplicates: usize,
    /// Number of proposal rows that failed to persist.
    pub failed_proposals: usize,
    /// Number of persistence attempts across all workers.
    pub persist_attempts: usize,
    /// Whether run ended due planning or execution budget exhaustion.
    pub timed_out: bool,
    /// End-to-end execution wall-clock duration in milliseconds.
    pub elapsed_ms: f64,
    /// Bounded error samples captured during proposal persistence.
    pub errors: Vec<String>,
}
