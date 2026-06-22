//! Host-owned memory contract rows for Julia staging.

use serde::{Deserialize, Serialize};

/// Lifecycle state used by memory gate evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryLifecycleState {
    /// Memory item has been opened but not yet validated.
    Open,
    /// Memory item is currently active in short-term memory.
    Active,
    /// Memory item is cooling down pending more evidence.
    Cooling,
    /// Memory item needs explicit revalidation before next transition.
    RevalidatePending,
    /// Memory item was purged by gate policy.
    Purged,
    /// Memory item was promoted out of episodic memory by gate policy.
    Promoted,
}

impl MemoryLifecycleState {
    /// String form used in staged payload fields.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Active => "active",
            Self::Cooling => "cooling",
            Self::RevalidatePending => "revalidate_pending",
            Self::Purged => "purged",
            Self::Promoted => "promoted",
        }
    }
}

/// Utility ledger used by gate-score request staging.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryUtilityLedger {
    /// `ReAct` re-validation score.
    pub react_revalidation_score: f32,
    /// Graph structural consistency score.
    pub graph_consistency_score: f32,
    /// Omega governance alignment score.
    pub omega_alignment_score: f32,
    /// TTL/frequency score.
    pub ttl_score: f32,
    /// Final weighted utility score.
    pub utility_score: f32,
    /// Current Q-value.
    pub q_value: f32,
    /// Observed usage count.
    pub usage_count: u32,
    /// Failure ratio in [0, 1].
    pub failure_rate: f32,
}

/// Canonical read-only memory features exported to Julia compute lanes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryProjectionRow {
    /// Stable memory identifier used as the host-side join key.
    pub episode_id: String,
    /// Logical scope associated with the memory item.
    pub scope: String,
    /// Semantic embedding of the stored intent.
    pub intent_embedding: Vec<f32>,
    /// Current utility estimate sourced from the host.
    pub q_value: f32,
    /// Number of observed successful recalls.
    pub success_count: u32,
    /// Number of observed failed recalls.
    pub failure_count: u32,
    /// Number of total retrievals or accesses.
    pub retrieval_count: u32,
    /// Memory creation timestamp in Unix milliseconds.
    pub created_at_ms: MemoryProjectionTimestampMs,
    /// Memory last-update timestamp in Unix milliseconds.
    pub updated_at_ms: MemoryProjectionTimestampMs,
}

/// Unix timestamp in milliseconds for memory projection rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MemoryProjectionTimestampMs(i64);

impl MemoryProjectionTimestampMs {
    /// Returns the timestamp value.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }
}

impl From<i64> for MemoryProjectionTimestampMs {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl PartialEq<i64> for MemoryProjectionTimestampMs {
    fn eq(&self, other: &i64) -> bool {
        self.get() == *other
    }
}

/// Normalized subset of recall plan fields sent into plan-tuning lanes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RecallPlanTuning {
    /// Phase-1 candidate count.
    pub k1: usize,
    /// Phase-2 rerank output count.
    pub k2: usize,
    /// Q-value blending weight in rerank score.
    pub lambda: f32,
    /// Minimum retained similarity score.
    pub min_score: f32,
    /// Max context budget in chars for injected recall block.
    pub max_context_chars: usize,
}

/// Normalize a feedback-bias value to finite `[-1.0, 1.0]`.
#[must_use]
pub fn normalize_feedback_bias(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}
