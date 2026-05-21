//! 3-in-1 memory gate: retain / obsolete / promote out of episodic memory.
//!
//! This module provides a deterministic utility ledger and gate policy that
//! can be replayed for audits.

use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
pub use xiuxian_types::{MemoryGateDecision, MemoryGateVerdict, MemoryPromotionTarget};

use crate::Episode;

fn u32_to_f32(value: u32) -> f32 {
    value.to_f32().unwrap_or(f32::MAX)
}

macro_rules! memory_gate_string_type {
    ($name:ident) => {
        #[doc = "String identifier used by memory-gate event APIs."]
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Borrows the serialized identifier value.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                self.as_str()
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }
    };
}

memory_gate_string_type!(MemoryGateSessionId);
memory_gate_string_type!(MemoryGateMemoryId);

/// Monotonic turn id for a memory-gate event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MemoryGateTurnId(u64);

impl MemoryGateTurnId {
    /// Returns the numeric turn id.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl From<u64> for MemoryGateTurnId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// Lifecycle state used by gate events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryLifecycleState {
    /// Episode has been opened but not yet validated.
    Open,
    /// Episode is currently active in short-term memory.
    Active,
    /// Episode is cooling down pending more evidence.
    Cooling,
    /// Episode needs explicit revalidation before next transition.
    RevalidatePending,
    /// Episode was purged by gate policy.
    Purged,
    /// Episode was promoted out of episodic memory by gate policy.
    Promoted,
}

impl MemoryLifecycleState {
    /// String form used in event payload fields.
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

/// Utility ledger that powers gate decisions.
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

impl MemoryUtilityLedger {
    /// Build utility ledger from an episode and runtime evidence scores.
    #[must_use]
    pub fn from_episode(
        episode: &Episode,
        react_revalidation_score: f32,
        graph_consistency_score: f32,
        omega_alignment_score: f32,
    ) -> Self {
        let feedback = episode_feedback_facts(episode);
        let q_value = episode.q_value.clamp(0.0, 1.0);
        let ttl_score = ttl_score_from_feedback(&feedback);
        let evidence = MemoryEvidenceScores::new(
            react_revalidation_score,
            graph_consistency_score,
            omega_alignment_score,
        );
        let utility_score = utility_score_from_evidence(evidence, q_value);

        Self {
            react_revalidation_score: evidence.react,
            graph_consistency_score: evidence.graph,
            omega_alignment_score: evidence.omega,
            ttl_score,
            utility_score,
            q_value,
            usage_count: feedback.usage_count,
            failure_rate: feedback.failure_rate,
        }
    }
}

struct EpisodeFeedbackFacts {
    usage_count: u32,
    usage_count_f32: f32,
    success: f32,
    failure: f32,
    failure_rate: f32,
}

fn episode_feedback_facts(episode: &Episode) -> EpisodeFeedbackFacts {
    let usage_count = episode.total_uses();
    let success = u32_to_f32(episode.success_count);
    let failure = u32_to_f32(episode.failure_count);
    let feedback_count = episode.feedback_count();
    EpisodeFeedbackFacts {
        usage_count,
        usage_count_f32: u32_to_f32(usage_count),
        success,
        failure,
        failure_rate: failure_rate_for_episode(episode, failure, feedback_count),
    }
}

fn failure_rate_for_episode(episode: &Episode, failure: f32, feedback_count: u32) -> f32 {
    if feedback_count == 0 {
        return if episode.outcome.eq_ignore_ascii_case("error") {
            1.0
        } else {
            0.0
        };
    }
    (failure / u32_to_f32(feedback_count)).clamp(0.0, 1.0)
}

fn ttl_score_from_feedback(feedback: &EpisodeFeedbackFacts) -> f32 {
    let frequency_score = frequency_score(feedback.usage_count, feedback.usage_count_f32);
    let stability_score = (1.0 - feedback.failure_rate).clamp(0.0, 1.0);
    let success_bias =
        ((feedback.success + 1.0) / (feedback.success + feedback.failure + 2.0)).clamp(0.0, 1.0);
    (0.45 * frequency_score + 0.35 * stability_score + 0.20 * success_bias).clamp(0.0, 1.0)
}

fn frequency_score(usage_count: u32, usage_count_f32: f32) -> f32 {
    if usage_count == 0 {
        return 0.0;
    }
    (usage_count_f32 / (usage_count_f32 + 3.0)).clamp(0.0, 1.0)
}

#[derive(Clone, Copy)]
struct MemoryEvidenceScores {
    react: f32,
    graph: f32,
    omega: f32,
}

impl MemoryEvidenceScores {
    fn new(react: f32, graph: f32, omega: f32) -> Self {
        Self {
            react: react.clamp(0.0, 1.0),
            graph: graph.clamp(0.0, 1.0),
            omega: omega.clamp(0.0, 1.0),
        }
    }
}

fn utility_score_from_evidence(evidence: MemoryEvidenceScores, q_value: f32) -> f32 {
    (0.32 * evidence.react + 0.23 * evidence.graph + 0.25 * evidence.omega + 0.20 * q_value)
        .clamp(0.0, 1.0)
}

/// Deterministic policy thresholds for memory gate.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MemoryGatePolicy {
    /// Minimum utility score for promotion.
    pub promote_threshold: f32,
    /// Maximum utility score for obsolescence.
    pub obsolete_threshold: f32,
    /// Minimum usage before promotion is allowed.
    pub promote_min_usage: u32,
    /// Minimum usage before obsolescence is allowed.
    pub obsolete_min_usage: u32,
    /// Failure-rate ceiling for promotion.
    pub promote_failure_rate_ceiling: f32,
    /// Failure-rate floor for obsolescence.
    pub obsolete_failure_rate_floor: f32,
    /// Minimum TTL score for promotion.
    pub promote_min_ttl_score: f32,
    /// Maximum TTL score for obsolescence.
    pub obsolete_max_ttl_score: f32,
}

impl Default for MemoryGatePolicy {
    fn default() -> Self {
        Self {
            promote_threshold: 0.78,
            obsolete_threshold: 0.32,
            promote_min_usage: 3,
            obsolete_min_usage: 2,
            promote_failure_rate_ceiling: 0.25,
            obsolete_failure_rate_floor: 0.70,
            promote_min_ttl_score: 0.50,
            obsolete_max_ttl_score: 0.45,
        }
    }
}

impl MemoryGatePolicy {
    /// Evaluate one utility ledger with explicit evidence references.
    #[must_use]
    pub fn evaluate(
        self,
        ledger: &MemoryUtilityLedger,
        react_evidence_refs: Vec<String>,
        graph_evidence_refs: Vec<String>,
        mut omega_factors: Vec<String>,
    ) -> MemoryGateDecision {
        omega_factors.push(format!("utility_score={:.3}", ledger.utility_score));
        omega_factors.push(format!("ttl_score={:.3}", ledger.ttl_score));
        omega_factors.push(format!("q_value={:.3}", ledger.q_value));
        omega_factors.push(format!("failure_rate={:.3}", ledger.failure_rate));
        omega_factors.push(format!("usage_count={}", ledger.usage_count));

        let verdict = if ledger.usage_count >= self.promote_min_usage
            && ledger.utility_score >= self.promote_threshold
            && ledger.failure_rate <= self.promote_failure_rate_ceiling
            && ledger.ttl_score >= self.promote_min_ttl_score
        {
            MemoryGateVerdict::Promote
        } else if ledger.usage_count >= self.obsolete_min_usage
            && ledger.utility_score <= self.obsolete_threshold
            && ledger.failure_rate >= self.obsolete_failure_rate_floor
            && ledger.ttl_score <= self.obsolete_max_ttl_score
        {
            MemoryGateVerdict::Obsolete
        } else {
            MemoryGateVerdict::Retain
        };

        let confidence = match verdict {
            MemoryGateVerdict::Promote => {
                let utility_margin = (ledger.utility_score - self.promote_threshold).max(0.0);
                let failure_margin =
                    (self.promote_failure_rate_ceiling - ledger.failure_rate).max(0.0);
                (0.55 + utility_margin * 1.8 + failure_margin * 0.8).clamp(0.0, 1.0)
            }
            MemoryGateVerdict::Obsolete => {
                let utility_margin = (self.obsolete_threshold - ledger.utility_score).max(0.0);
                let failure_margin =
                    (ledger.failure_rate - self.obsolete_failure_rate_floor).max(0.0);
                (0.55 + utility_margin * 1.8 + failure_margin * 0.8).clamp(0.0, 1.0)
            }
            MemoryGateVerdict::Retain => {
                let to_promote = (self.promote_threshold - ledger.utility_score).abs();
                let to_obsolete = (ledger.utility_score - self.obsolete_threshold).abs();
                (0.42 + to_promote.min(to_obsolete) * 0.6).clamp(0.0, 0.85)
            }
        };

        let reason = format!(
            "utility={:.3}, ttl={:.3}, uses={}, failure_rate={:.3}, react={:.3}, graph={:.3}, omega={:.3}",
            ledger.utility_score,
            ledger.ttl_score,
            ledger.usage_count,
            ledger.failure_rate,
            ledger.react_revalidation_score,
            ledger.graph_consistency_score,
            ledger.omega_alignment_score
        );

        MemoryGateDecision {
            verdict,
            confidence,
            react_evidence_refs,
            graph_evidence_refs,
            omega_factors,
            reason,
            next_action: verdict.as_str().to_string(),
            promotion_target: match verdict {
                MemoryGateVerdict::Promote => Some(MemoryPromotionTarget::WorkingKnowledge),
                _ => None,
            },
        }
    }
}

/// Auditable gate-event payload aligned with `xiuxian.memory.gate_event.v1`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryGateEvent {
    /// Session identifier of the decision context.
    pub session_id: MemoryGateSessionId,
    /// Monotonic turn identifier within the runtime.
    pub turn_id: MemoryGateTurnId,
    /// Memory episode id this decision targets.
    pub memory_id: MemoryGateMemoryId,
    /// Lifecycle state before decision.
    pub state_before: MemoryLifecycleState,
    /// Lifecycle state after decision.
    pub state_after: MemoryLifecycleState,
    /// TTL/frequency score used by the decision.
    pub ttl_score: f32,
    /// Full decision payload with evidence and verdict.
    pub decision: MemoryGateDecision,
}

/// Input required to build a canonical memory-gate event.
#[derive(Debug, Clone)]
pub struct MemoryGateEventInput {
    /// Session identifier of the decision context.
    pub session_id: MemoryGateSessionId,
    /// Monotonic turn identifier within the runtime.
    pub turn_id: MemoryGateTurnId,
    /// Memory episode id this decision targets.
    pub memory_id: MemoryGateMemoryId,
    /// Utility ledger used by the decision.
    pub ledger: MemoryUtilityLedger,
    /// Gate decision payload.
    pub decision: MemoryGateDecision,
}

impl MemoryGateEvent {
    /// Build a canonical gate event from one decision.
    #[must_use]
    pub fn from_decision(input: MemoryGateEventInput) -> Self {
        let state_before = MemoryLifecycleState::Active;
        let state_after = match input.decision.verdict {
            MemoryGateVerdict::Retain => MemoryLifecycleState::Active,
            MemoryGateVerdict::Obsolete => MemoryLifecycleState::Purged,
            MemoryGateVerdict::Promote => MemoryLifecycleState::Promoted,
        };

        Self {
            session_id: input.session_id,
            turn_id: input.turn_id,
            memory_id: input.memory_id,
            state_before,
            state_after,
            ttl_score: input.ledger.ttl_score,
            decision: input.decision,
        }
    }
}
