//! `link_graph::agentic::types::config` owns Wendao agentic types config behavior.

use serde::{Deserialize, Serialize};

/// Runtime budget config for bounded agentic expansion planning.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphAgenticExpansionConfig {
    /// Max number of worker partitions in one planning cycle.
    pub max_workers: usize,
    /// Max candidate notes considered before pair generation.
    pub max_candidates: usize,
    /// Max candidate pairs assigned to one worker.
    pub max_pairs_per_worker: usize,
    /// End-to-end planner wall-clock budget in milliseconds.
    pub time_budget_ms: f64,
}

impl Default for LinkGraphAgenticExpansionConfig {
    fn default() -> Self {
        Self {
            max_workers: 4,
            max_candidates: 256,
            max_pairs_per_worker: 128,
            time_budget_ms: 250.0,
        }
    }
}

impl LinkGraphAgenticExpansionConfig {
    /// Return a config with guardrails applied (`>= 1` and finite positive budget).
    #[must_use]
    pub fn normalized(self) -> Self {
        Self {
            max_workers: self.max_workers.max(1),
            max_candidates: self.max_candidates.max(1),
            max_pairs_per_worker: self.max_pairs_per_worker.max(1),
            time_budget_ms: if self.time_budget_ms.is_finite() && self.time_budget_ms > 0.0 {
                self.time_budget_ms
            } else {
                250.0
            },
        }
    }
}

/// Runtime config for bounded agentic expansion execution workers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphAgenticExecutionConfig {
    /// Bounded expansion planner config used before execution.
    pub expansion: LinkGraphAgenticExpansionConfig,
    /// Per-worker wall-clock budget in milliseconds.
    pub worker_time_budget_ms: f64,
    /// Whether to persist generated proposals into Valkey suggested-link stream.
    pub persist_suggestions: bool,
    /// Max persistence retry attempts for one proposal (`>= 1`).
    pub persist_retry_attempts: usize,
    /// Max latest-state rows scanned for idempotency dedupe (`>= 1`).
    pub idempotency_scan_limit: usize,
    /// Proposed relation label attached to generated suggestions.
    pub relation: String,
    /// Agent id recorded in generated suggestion rows.
    pub agent_id: String,
    /// Human-readable evidence prefix prepended to each generated suggestion.
    pub evidence_prefix: String,
    /// Optional deterministic timestamp override for generated suggestions.
    #[serde(default)]
    pub created_at_unix: Option<f64>,
}

impl Default for LinkGraphAgenticExecutionConfig {
    fn default() -> Self {
        Self {
            expansion: LinkGraphAgenticExpansionConfig::default(),
            worker_time_budget_ms: 120.0,
            persist_suggestions: false,
            persist_retry_attempts: 2,
            idempotency_scan_limit: 2000,
            relation: "related_to".to_string(),
            agent_id: "qianhuan-architect".to_string(),
            evidence_prefix: "agentic expansion bridge candidate".to_string(),
            created_at_unix: None,
        }
    }
}

impl LinkGraphAgenticExecutionConfig {
    /// Return a config with execution guardrails applied.
    #[must_use]
    pub fn normalized(self) -> Self {
        let relation = self.relation.trim();
        let agent_id = self.agent_id.trim();
        let evidence_prefix = self.evidence_prefix.trim();
        Self {
            expansion: self.expansion.normalized(),
            worker_time_budget_ms: if self.worker_time_budget_ms.is_finite()
                && self.worker_time_budget_ms > 0.0
            {
                self.worker_time_budget_ms
            } else {
                120.0
            },
            persist_suggestions: self.persist_suggestions,
            persist_retry_attempts: self.persist_retry_attempts.max(1),
            idempotency_scan_limit: self.idempotency_scan_limit.max(1),
            relation: if relation.is_empty() {
                "related_to".to_string()
            } else {
                relation.to_string()
            },
            agent_id: if agent_id.is_empty() {
                "qianhuan-architect".to_string()
            } else {
                agent_id.to_string()
            },
            evidence_prefix: if evidence_prefix.is_empty() {
                "agentic expansion bridge candidate".to_string()
            } else {
                evidence_prefix.to_string()
            },
            created_at_unix: self
                .created_at_unix
                .filter(|value| value.is_finite() && *value >= 0.0),
        }
    }
}
