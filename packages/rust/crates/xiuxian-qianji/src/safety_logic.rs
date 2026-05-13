//! Safety logic surface for `xiuxian-qianji`.

use serde::{Deserialize, Serialize};

/// Basic logical proposition extracted from LLM output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposition {
    /// The name of the fact or action (for example, `RefinedFact`).
    pub predicate: String,
    /// Whether this proposition carries a valid source reference.
    pub has_grounding: bool,
    /// Confidence level assigned by the Analyzer.
    pub confidence: f32,
}

/// Linear Temporal Logic inspired invariants.
#[derive(Debug, Clone, Copy)]
pub enum Invariant {
    /// Globally: Every proposition must be grounded.
    MustBeGrounded,
    /// Future: Eventually, confidence must reach threshold.
    MinConfidence(f32),
}

impl Invariant {
    /// Validates a trace of propositions against the invariant.
    #[must_use]
    pub fn check(self, trace: &[Proposition]) -> bool {
        match self {
            Self::MustBeGrounded => trace.iter().all(|p| p.has_grounding),
            Self::MinConfidence(min) => trace.iter().any(|p| p.confidence >= min),
        }
    }
}
