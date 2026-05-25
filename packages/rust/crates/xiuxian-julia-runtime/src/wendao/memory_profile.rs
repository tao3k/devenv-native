//! Stable Wendao memory-family Julia profile identities.

use xiuxian_wendao_runtime::config::{
    DEFAULT_MEMORY_JULIA_COMPUTE_CALIBRATION_ROUTE,
    DEFAULT_MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_ROUTE,
    DEFAULT_MEMORY_JULIA_COMPUTE_GATE_SCORE_ROUTE, DEFAULT_MEMORY_JULIA_COMPUTE_PLAN_TUNING_ROUTE,
};

/// Stable capability family id for the Wendao memory Julia compute ABI.
pub const MEMORY_JULIA_COMPUTE_FAMILY_ID: &str = "memory";
/// Stable profile id for the read-only episodic recall lane.
pub const MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_PROFILE_ID: &str = "episodic_recall";
/// Stable profile id for recommendation-only gate scoring.
pub const MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID: &str = "memory_gate_score";
/// Stable profile id for advice-only plan tuning.
pub const MEMORY_JULIA_COMPUTE_PLAN_TUNING_PROFILE_ID: &str = "memory_plan_tuning";
/// Stable profile id for artifact-only calibration.
pub const MEMORY_JULIA_COMPUTE_CALIBRATION_PROFILE_ID: &str = "memory_calibration";

/// Stable request schema id for the read-only episodic recall lane.
pub const MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_REQUEST_SCHEMA_ID: &str =
    "memory.episodic_recall.request.v1";
/// Stable response schema id for the read-only episodic recall lane.
pub const MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_RESPONSE_SCHEMA_ID: &str =
    "memory.episodic_recall.response.v1";
/// Stable request schema id for memory gate scoring.
pub const MEMORY_JULIA_COMPUTE_GATE_SCORE_REQUEST_SCHEMA_ID: &str = "memory.gate_score.request.v1";
/// Stable response schema id for memory gate scoring.
pub const MEMORY_JULIA_COMPUTE_GATE_SCORE_RESPONSE_SCHEMA_ID: &str =
    "memory.gate_score.response.v1";
/// Stable request schema id for memory plan tuning.
pub const MEMORY_JULIA_COMPUTE_PLAN_TUNING_REQUEST_SCHEMA_ID: &str =
    "memory.plan_tuning.request.v1";
/// Stable response schema id for memory plan tuning.
pub const MEMORY_JULIA_COMPUTE_PLAN_TUNING_RESPONSE_SCHEMA_ID: &str =
    "memory.plan_tuning.response.v1";
/// Stable request schema id for memory calibration.
pub const MEMORY_JULIA_COMPUTE_CALIBRATION_REQUEST_SCHEMA_ID: &str =
    "memory.calibration.request.v1";
/// Stable response schema id for memory calibration.
pub const MEMORY_JULIA_COMPUTE_CALIBRATION_RESPONSE_SCHEMA_ID: &str =
    "memory.calibration.response.v1";

/// Ordered staged profiles for the memory-family Julia compute ABI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemoryJuliaComputeProfile {
    /// Read-only episodic recall over the host projection surface.
    EpisodicRecall,
    /// Recommendation-only memory gate scoring.
    MemoryGateScore,
    /// Advice-only memory plan tuning.
    MemoryPlanTuning,
    /// Artifact-only memory calibration.
    MemoryCalibration,
}

impl MemoryJuliaComputeProfile {
    /// Ordered staged profiles in binding-generation order.
    pub const ALL: [Self; 4] = [
        Self::EpisodicRecall,
        Self::MemoryGateScore,
        Self::MemoryPlanTuning,
        Self::MemoryCalibration,
    ];

    /// Parses one staged profile id.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_PROFILE_ID => Some(Self::EpisodicRecall),
            MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID => Some(Self::MemoryGateScore),
            MEMORY_JULIA_COMPUTE_PLAN_TUNING_PROFILE_ID => Some(Self::MemoryPlanTuning),
            MEMORY_JULIA_COMPUTE_CALIBRATION_PROFILE_ID => Some(Self::MemoryCalibration),
            _ => None,
        }
    }

    /// Returns the stable host capability id for this profile.
    #[must_use]
    pub fn capability_id(self) -> &'static str {
        self.profile_id()
    }

    /// Returns the stable family-level profile id.
    #[must_use]
    pub const fn profile_id(self) -> &'static str {
        match self {
            Self::EpisodicRecall => MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_PROFILE_ID,
            Self::MemoryGateScore => MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID,
            Self::MemoryPlanTuning => MEMORY_JULIA_COMPUTE_PLAN_TUNING_PROFILE_ID,
            Self::MemoryCalibration => MEMORY_JULIA_COMPUTE_CALIBRATION_PROFILE_ID,
        }
    }

    /// Returns the default route for this staged profile.
    #[must_use]
    pub const fn default_route(self) -> &'static str {
        match self {
            Self::EpisodicRecall => DEFAULT_MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_ROUTE,
            Self::MemoryGateScore => DEFAULT_MEMORY_JULIA_COMPUTE_GATE_SCORE_ROUTE,
            Self::MemoryPlanTuning => DEFAULT_MEMORY_JULIA_COMPUTE_PLAN_TUNING_ROUTE,
            Self::MemoryCalibration => DEFAULT_MEMORY_JULIA_COMPUTE_CALIBRATION_ROUTE,
        }
    }
}
