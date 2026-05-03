use crate::bin_support::wendao::types::{DecisionTargetStateArg, SuggestedLinkStateArg};
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub(crate) enum AgenticCommand {
    /// Log one suggested-link proposal (state defaults to provisional).
    Log {
        source_id: String,
        target_id: String,
        relation: String,
        #[arg(long, default_value_t = 0.5)]
        confidence: f64,
        #[arg(long)]
        evidence: String,
        #[arg(long, default_value = "qianhuan-architect")]
        agent_id: String,
        #[arg(long)]
        created_at_unix: Option<f64>,
    },
    /// Read recent suggested-link proposals.
    Recent {
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
        /// Return latest unique state per suggestion id.
        #[arg(long, default_value_t = false)]
        latest: bool,
        /// Filter by current lifecycle state.
        #[arg(long, value_enum)]
        state: Option<SuggestedLinkStateArg>,
    },
    /// Apply one transition from provisional to promoted/rejected.
    Decide {
        suggestion_id: String,
        #[arg(long, value_enum)]
        target_state: DecisionTargetStateArg,
        #[arg(long)]
        decided_by: String,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        decided_at_unix: Option<f64>,
    },
    /// Read recent decision audit rows.
    Decisions {
        #[arg(short, long, default_value_t = 20)]
        limit: usize,
    },
    /// Plan bounded sub-agent expansion workers for candidate link proposals.
    Plan {
        /// Optional query used to narrow candidate notes before pairing.
        #[arg(long)]
        query: Option<String>,
        /// Optional max worker override (defaults to runtime config).
        #[arg(long)]
        max_workers: Option<usize>,
        /// Optional candidate-note cap override (defaults to runtime config).
        #[arg(long)]
        max_candidates: Option<usize>,
        /// Optional per-worker pair cap override (defaults to runtime config).
        #[arg(long)]
        max_pairs_per_worker: Option<usize>,
        /// Optional planner wall-clock budget override in milliseconds.
        #[arg(long)]
        time_budget_ms: Option<f64>,
    },
    /// Execute bounded sub-agent expansion workers and optionally persist suggestions.
    Run {
        /// Optional query used to narrow candidate notes before pairing.
        #[arg(long)]
        query: Option<String>,
        /// Optional max worker override (defaults to runtime config).
        #[arg(long)]
        max_workers: Option<usize>,
        /// Optional candidate-note cap override (defaults to runtime config).
        #[arg(long)]
        max_candidates: Option<usize>,
        /// Optional per-worker pair cap override (defaults to runtime config).
        #[arg(long)]
        max_pairs_per_worker: Option<usize>,
        /// Optional planner wall-clock budget override in milliseconds.
        #[arg(long)]
        time_budget_ms: Option<f64>,
        /// Optional per-worker runtime budget override in milliseconds.
        #[arg(long)]
        worker_time_budget_ms: Option<f64>,
        /// Optional persistence override:
        /// `--persist` (true), `--persist=false`.
        #[arg(
            long = "persist",
            value_name = "BOOL",
            num_args = 0..=1,
            default_missing_value = "true"
        )]
        persist: Option<bool>,
        /// Optional persistence retry attempts override (`>= 1`).
        #[arg(long)]
        persist_retry_attempts: Option<usize>,
        /// Optional idempotency scan limit override (`>= 1`).
        #[arg(long)]
        idempotency_scan_limit: Option<usize>,
        /// Optional relation label override for generated suggestions.
        #[arg(long)]
        relation: Option<String>,
        /// Optional agent id override for generated suggestions.
        #[arg(long)]
        agent_id: Option<String>,
        /// Optional evidence prefix override for generated suggestions.
        #[arg(long)]
        evidence_prefix: Option<String>,
        /// Optional deterministic timestamp override for tests/replay.
        #[arg(long)]
        created_at_unix: Option<f64>,
        /// Include aggregated monitor phases + bottleneck summary in response payload.
        #[arg(long, default_value_t = false)]
        verbose: bool,
    },
}
