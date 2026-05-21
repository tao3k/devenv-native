//! Runtime profile helpers for Julia memory compute backends.

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

const MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_REQUEST_SCHEMA_ID: &str =
    "memory.episodic_recall.request.v1";
const MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_RESPONSE_SCHEMA_ID: &str =
    "memory.episodic_recall.response.v1";
const MEMORY_JULIA_COMPUTE_GATE_SCORE_REQUEST_SCHEMA_ID: &str = "memory.gate_score.request.v1";
const MEMORY_JULIA_COMPUTE_GATE_SCORE_RESPONSE_SCHEMA_ID: &str = "memory.gate_score.response.v1";
const MEMORY_JULIA_COMPUTE_PLAN_TUNING_REQUEST_SCHEMA_ID: &str = "memory.plan_tuning.request.v1";
const MEMORY_JULIA_COMPUTE_PLAN_TUNING_RESPONSE_SCHEMA_ID: &str = "memory.plan_tuning.response.v1";
const MEMORY_JULIA_COMPUTE_CALIBRATION_REQUEST_SCHEMA_ID: &str = "memory.calibration.request.v1";
const MEMORY_JULIA_COMPUTE_CALIBRATION_RESPONSE_SCHEMA_ID: &str = "memory.calibration.response.v1";

/// Family-level profile metadata for one staged memory Julia compute profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MemoryJuliaComputeProfileContract {
    /// Stable capability family id.
    pub(crate) family: &'static str,
    /// Stable capability id used by the host binding.
    pub(crate) capability_id: &'static str,
    /// Stable profile id carried by family-aware manifests.
    pub(crate) profile_id: &'static str,
    /// Stable request schema id for semantic versioning.
    pub(crate) request_schema_id: &'static str,
    /// Stable response schema id for semantic versioning.
    pub(crate) response_schema_id: &'static str,
}

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

    /// Parse one staged profile id.
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

    /// Return the stable host capability id for this profile.
    #[must_use]
    pub fn capability_id(self) -> &'static str {
        self.profile_id()
    }

    /// Return the stable family-level profile id.
    #[must_use]
    pub fn profile_id(self) -> &'static str {
        match self {
            Self::EpisodicRecall => MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_PROFILE_ID,
            Self::MemoryGateScore => MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID,
            Self::MemoryPlanTuning => MEMORY_JULIA_COMPUTE_PLAN_TUNING_PROFILE_ID,
            Self::MemoryCalibration => MEMORY_JULIA_COMPUTE_CALIBRATION_PROFILE_ID,
        }
    }

    /// Return the default route for this staged profile.
    #[must_use]
    pub fn default_route(self) -> &'static str {
        match self {
            Self::EpisodicRecall => DEFAULT_MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_ROUTE,
            Self::MemoryGateScore => DEFAULT_MEMORY_JULIA_COMPUTE_GATE_SCORE_ROUTE,
            Self::MemoryPlanTuning => DEFAULT_MEMORY_JULIA_COMPUTE_PLAN_TUNING_ROUTE,
            Self::MemoryCalibration => DEFAULT_MEMORY_JULIA_COMPUTE_CALIBRATION_ROUTE,
        }
    }

    /// Return the staged semantic contract metadata for this profile.
    #[must_use]
    pub(crate) fn contract(self) -> MemoryJuliaComputeProfileContract {
        match self {
            Self::EpisodicRecall => MemoryJuliaComputeProfileContract {
                family: MEMORY_JULIA_COMPUTE_FAMILY_ID,
                capability_id: MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_PROFILE_ID,
                profile_id: MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_PROFILE_ID,
                request_schema_id: MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_REQUEST_SCHEMA_ID,
                response_schema_id: MEMORY_JULIA_COMPUTE_EPISODIC_RECALL_RESPONSE_SCHEMA_ID,
            },
            Self::MemoryGateScore => MemoryJuliaComputeProfileContract {
                family: MEMORY_JULIA_COMPUTE_FAMILY_ID,
                capability_id: MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID,
                profile_id: MEMORY_JULIA_COMPUTE_GATE_SCORE_PROFILE_ID,
                request_schema_id: MEMORY_JULIA_COMPUTE_GATE_SCORE_REQUEST_SCHEMA_ID,
                response_schema_id: MEMORY_JULIA_COMPUTE_GATE_SCORE_RESPONSE_SCHEMA_ID,
            },
            Self::MemoryPlanTuning => MemoryJuliaComputeProfileContract {
                family: MEMORY_JULIA_COMPUTE_FAMILY_ID,
                capability_id: MEMORY_JULIA_COMPUTE_PLAN_TUNING_PROFILE_ID,
                profile_id: MEMORY_JULIA_COMPUTE_PLAN_TUNING_PROFILE_ID,
                request_schema_id: MEMORY_JULIA_COMPUTE_PLAN_TUNING_REQUEST_SCHEMA_ID,
                response_schema_id: MEMORY_JULIA_COMPUTE_PLAN_TUNING_RESPONSE_SCHEMA_ID,
            },
            Self::MemoryCalibration => MemoryJuliaComputeProfileContract {
                family: MEMORY_JULIA_COMPUTE_FAMILY_ID,
                capability_id: MEMORY_JULIA_COMPUTE_CALIBRATION_PROFILE_ID,
                profile_id: MEMORY_JULIA_COMPUTE_CALIBRATION_PROFILE_ID,
                request_schema_id: MEMORY_JULIA_COMPUTE_CALIBRATION_REQUEST_SCHEMA_ID,
                response_schema_id: MEMORY_JULIA_COMPUTE_CALIBRATION_RESPONSE_SCHEMA_ID,
            },
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/memory/profile.rs"]
mod tests;
