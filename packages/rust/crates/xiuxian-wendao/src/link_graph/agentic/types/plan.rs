//! `link_graph::agentic::types::plan` owns Wendao agentic types plan behavior.

use super::config::LinkGraphAgenticExpansionConfig;
use serde::{Deserialize, Serialize};

/// One ranked candidate pair planned for agentic enrichment workers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphAgenticCandidatePair {
    /// Canonical left endpoint id/path.
    pub left_id: String,
    /// Canonical right endpoint id/path.
    pub right_id: String,
    /// Planner priority in `[0.0, 1.0]`.
    pub priority: f64,
}

/// One worker partition in the bounded expansion plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphAgenticWorkerPlan {
    /// Zero-based worker index in this planning cycle.
    pub worker_id: usize,
    /// Unique seed note ids touched by this worker's pair set.
    pub seed_ids: Vec<String>,
    /// Candidate pairs assigned to this worker.
    pub pairs: Vec<LinkGraphAgenticCandidatePair>,
    /// Number of candidate pairs in this worker partition.
    pub pair_count: usize,
}

/// Bounded sub-agent expansion planning result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphAgenticExpansionPlan {
    /// Optional query used to narrow candidate notes before pairing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Total indexed notes available in current graph snapshot.
    pub total_notes: usize,
    /// Candidate notes that entered pair generation.
    pub candidate_notes: usize,
    /// Total possible undirected pairs from selected candidates.
    pub total_possible_pairs: usize,
    /// Pairs evaluated before truncation/timeout.
    pub evaluated_pairs: usize,
    /// Pairs selected after ranking and budget limits.
    pub selected_pairs: usize,
    /// Whether planner stopped early due wall-clock time budget.
    pub timed_out: bool,
    /// Whether selected pairs were capped by `max_workers * max_pairs_per_worker`.
    pub capped_by_pair_limit: bool,
    /// Effective planner config used for this run.
    pub config: LinkGraphAgenticExpansionConfig,
    /// End-to-end planner duration in milliseconds.
    pub elapsed_ms: f64,
    /// Worker partitions generated for this cycle.
    pub workers: Vec<LinkGraphAgenticWorkerPlan>,
}
