//! Health, readiness, pressure, and fallback evidence contracts.

use serde::{Deserialize, Serialize};

use crate::lanes::PolyglotLane;

/// Coarse health state reported for a lane.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthState {
    /// No health evidence has been reported yet.
    Unknown,
    /// The lane is healthy enough for normal admission.
    Healthy,
    /// The lane is available but should receive conservative admission.
    Degraded,
    /// The lane is unhealthy and should not receive new work.
    Unhealthy,
}

/// Readiness state reported for a lane or profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessState {
    /// No readiness evidence has been reported yet.
    Unknown,
    /// The lane is warming up and should not receive normal traffic.
    Warming,
    /// The lane is ready for normal traffic.
    Ready,
    /// The lane is available with reduced confidence.
    Degraded,
    /// The lane is administratively disabled.
    Disabled,
}

impl ReadinessState {
    /// Returns true when normal admission may proceed.
    #[must_use]
    pub const fn accepts_normal_traffic(self) -> bool {
        matches!(self, Self::Ready | Self::Degraded)
    }
}

/// Coarse pressure level reported for a lane.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PressureLevel {
    /// No pressure evidence has been reported yet.
    Unknown,
    /// The lane has spare capacity.
    Low,
    /// The lane is under normal load.
    Medium,
    /// The lane is near its budget.
    High,
    /// The lane is over budget or unsafe for new work.
    Critical,
}

impl PressureLevel {
    /// Returns true when this pressure level should reject new work.
    #[must_use]
    pub const fn rejects_new_work(self) -> bool {
        matches!(self, Self::Critical)
    }
}

/// Fallback evidence for a lane or capability.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FallbackEvidence {
    /// Whether a fallback path is available.
    pub available: bool,
    /// Human-readable fallback reason or owner note.
    pub reason: Option<String>,
}

impl FallbackEvidence {
    /// Creates fallback evidence with no reason text.
    #[must_use]
    pub const fn new(available: bool) -> Self {
        Self {
            available,
            reason: None,
        }
    }

    /// Creates fallback evidence with reason text.
    #[must_use]
    pub fn with_reason(available: bool, reason: impl Into<String>) -> Self {
        Self {
            available,
            reason: Some(reason.into()),
        }
    }
}

/// Evidence envelope for one polyglot lane.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LaneEvidence {
    /// Lane described by this evidence envelope.
    pub lane: PolyglotLane,
    /// Coarse health state.
    pub health: HealthState,
    /// Coarse readiness state.
    pub readiness: ReadinessState,
    /// Coarse pressure state.
    pub pressure: PressureLevel,
    /// Fallback evidence for this lane.
    pub fallback: FallbackEvidence,
}

/// Named input for constructing one lane evidence envelope.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LaneEvidenceInput {
    /// Lane described by this evidence envelope.
    pub lane: PolyglotLane,
    /// Coarse health state.
    pub health: HealthState,
    /// Coarse readiness state.
    pub readiness: ReadinessState,
    /// Coarse pressure state.
    pub pressure: PressureLevel,
    /// Fallback evidence for this lane.
    pub fallback: FallbackEvidence,
}

impl LaneEvidence {
    /// Creates evidence for a lane with explicit states.
    #[must_use]
    pub fn new(input: LaneEvidenceInput) -> Self {
        Self {
            lane: input.lane,
            health: input.health,
            readiness: input.readiness,
            pressure: input.pressure,
            fallback: input.fallback,
        }
    }
}
