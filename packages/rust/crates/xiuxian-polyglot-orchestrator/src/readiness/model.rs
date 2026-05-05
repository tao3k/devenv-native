//! Julia readiness evidence contracts.

use serde::{Deserialize, Serialize};

use crate::admission::AdmissionBudget;
use crate::evidence::{FallbackEvidence, HealthState, LaneEvidence, PressureLevel, ReadinessState};
use crate::lanes::{LaneCapability, PolyglotLane};

/// Validation state for an owner-supplied Julia contract fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractValidationState {
    /// No validation evidence has been supplied yet.
    Unknown,
    /// The owner package validated the contract fact.
    Valid,
    /// The owner package found the contract fact invalid.
    Invalid,
}

/// Manifest readiness for a Julia profile or package family.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifestReadinessState {
    /// No manifest evidence has been supplied yet.
    Unknown,
    /// Required manifest evidence is present and usable.
    Ready,
    /// Required manifest evidence is missing.
    Missing,
    /// Manifest evidence is present but incompatible with the selected profile.
    Incompatible,
}

/// Warmup status supplied by the Julia owner package.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarmupState {
    /// No warmup evidence has been supplied yet.
    Unknown,
    /// The profile has not warmed up yet.
    Cold,
    /// Warmup is currently in progress.
    Warming,
    /// Warmup completed for the selected profile shape.
    Ready,
    /// Warmup failed for the selected profile shape.
    Failed,
}

/// Benchmark status supplied by the Julia owner package.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkState {
    /// No benchmark evidence has been supplied yet.
    Unknown,
    /// This slice does not require benchmark evidence for normal admission.
    NotRequired,
    /// The observed benchmark is within the owner-selected threshold.
    WithinThreshold,
    /// The observed benchmark is above the owner-selected threshold.
    AboveThreshold,
    /// Benchmark collection failed.
    Failed,
}

/// Inert Julia readiness evidence supplied by an owner package.
///
/// The evidence records facts only. It does not call a Julia process, warm up a
/// worker, change routes, mutate schemas, or schedule work.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct JuliaReadinessEvidence {
    /// Lane described by this evidence.
    pub lane: PolyglotLane,
    /// Capability class being measured.
    pub capability: LaneCapability,
    /// Julia profile identifier supplied by the owner package.
    pub profile_id: String,
    /// Optional schema version supplied by the owner package.
    pub schema_version: Option<String>,
    /// Route validation state supplied by the owner package.
    pub route_validation: ContractValidationState,
    /// Schema validation state supplied by the owner package.
    pub schema_validation: ContractValidationState,
    /// Manifest readiness state supplied by the owner package.
    pub manifest_readiness: ManifestReadinessState,
    /// Warmup state supplied by the owner package.
    pub warmup: WarmupState,
    /// Benchmark state supplied by the owner package.
    pub benchmark: BenchmarkState,
    /// Optional maximum number of in-flight Julia requests.
    pub max_in_flight: Option<u32>,
    /// Current number of in-flight Julia requests supplied by the owner.
    pub active_in_flight: u32,
    /// Current queue depth supplied by the owner package.
    pub queue_depth: u32,
    /// Whether an owner-defined fallback path is available.
    pub fallback_available: bool,
}

impl JuliaReadinessEvidence {
    /// Creates empty readiness evidence for a Julia capability and profile.
    #[must_use]
    pub fn new(capability: LaneCapability, profile_id: impl Into<String>) -> Self {
        Self {
            lane: capability.owning_lane(),
            capability,
            profile_id: profile_id.into(),
            schema_version: None,
            route_validation: ContractValidationState::Unknown,
            schema_validation: ContractValidationState::Unknown,
            manifest_readiness: ManifestReadinessState::Unknown,
            warmup: WarmupState::Unknown,
            benchmark: BenchmarkState::Unknown,
            max_in_flight: None,
            active_in_flight: 0,
            queue_depth: 0,
            fallback_available: false,
        }
    }

    /// Creates memory-profile Julia readiness evidence.
    #[must_use]
    pub fn memory_profile(profile_id: impl Into<String>) -> Self {
        Self::new(LaneCapability::MemoryProfileCompute, profile_id)
    }

    /// Returns this evidence with an optional schema version.
    #[must_use]
    pub fn with_schema_version(mut self, schema_version: impl Into<String>) -> Self {
        self.schema_version = Some(schema_version.into());
        self
    }

    /// Returns this evidence with route validation state.
    #[must_use]
    pub const fn with_route_validation(
        mut self,
        route_validation: ContractValidationState,
    ) -> Self {
        self.route_validation = route_validation;
        self
    }

    /// Returns this evidence with schema validation state.
    #[must_use]
    pub const fn with_schema_validation(
        mut self,
        schema_validation: ContractValidationState,
    ) -> Self {
        self.schema_validation = schema_validation;
        self
    }

    /// Returns this evidence with manifest readiness state.
    #[must_use]
    pub const fn with_manifest_readiness(
        mut self,
        manifest_readiness: ManifestReadinessState,
    ) -> Self {
        self.manifest_readiness = manifest_readiness;
        self
    }

    /// Returns this evidence with warmup state.
    #[must_use]
    pub const fn with_warmup(mut self, warmup: WarmupState) -> Self {
        self.warmup = warmup;
        self
    }

    /// Returns this evidence with benchmark state.
    #[must_use]
    pub const fn with_benchmark(mut self, benchmark: BenchmarkState) -> Self {
        self.benchmark = benchmark;
        self
    }

    /// Returns this evidence with admission window counters.
    #[must_use]
    pub const fn with_admission_window(
        mut self,
        max_in_flight: Option<u32>,
        active_in_flight: u32,
        queue_depth: u32,
    ) -> Self {
        self.max_in_flight = max_in_flight;
        self.active_in_flight = active_in_flight;
        self.queue_depth = queue_depth;
        self
    }

    /// Returns this evidence with fallback availability.
    #[must_use]
    pub const fn with_fallback_available(mut self, fallback_available: bool) -> Self {
        self.fallback_available = fallback_available;
        self
    }

    /// Returns the coarse readiness represented by these facts.
    #[must_use]
    pub const fn readiness_state(&self) -> ReadinessState {
        if self.has_blocking_failure() {
            return ReadinessState::Disabled;
        }

        if self.has_unknown_or_warming_fact() {
            return ReadinessState::Warming;
        }

        if matches!(self.benchmark, BenchmarkState::AboveThreshold) {
            return ReadinessState::Degraded;
        }

        ReadinessState::Ready
    }

    /// Returns the coarse health represented by these facts.
    #[must_use]
    pub const fn health_state(&self) -> HealthState {
        if self.has_blocking_failure() {
            return HealthState::Unhealthy;
        }

        if matches!(
            self.readiness_state(),
            ReadinessState::Warming | ReadinessState::Degraded
        ) {
            return HealthState::Degraded;
        }

        HealthState::Healthy
    }

    /// Returns the coarse pressure represented by supplied admission counters.
    #[must_use]
    pub const fn pressure_level(&self) -> PressureLevel {
        match self.max_in_flight {
            Some(0) => PressureLevel::Critical,
            Some(max_in_flight) => self.pressure_with_capacity(max_in_flight),
            None => self.pressure_without_capacity(),
        }
    }

    /// Projects readiness evidence to an admission budget.
    #[must_use]
    pub const fn to_admission_budget(&self) -> AdmissionBudget {
        AdmissionBudget {
            lane: self.lane,
            max_in_flight: self.max_in_flight,
            active_in_flight: self.active_in_flight,
            queue_depth: self.queue_depth,
            readiness: self.readiness_state(),
            pressure: self.pressure_level(),
            fallback_available: self.fallback_available,
        }
    }

    /// Projects readiness evidence to a lane evidence envelope.
    #[must_use]
    pub fn to_lane_evidence(&self) -> LaneEvidence {
        LaneEvidence::new(
            self.lane,
            self.health_state(),
            self.readiness_state(),
            self.pressure_level(),
            FallbackEvidence::new(self.fallback_available),
        )
    }

    const fn has_blocking_failure(&self) -> bool {
        matches!(self.route_validation, ContractValidationState::Invalid)
            || matches!(self.schema_validation, ContractValidationState::Invalid)
            || matches!(
                self.manifest_readiness,
                ManifestReadinessState::Missing | ManifestReadinessState::Incompatible
            )
            || matches!(self.warmup, WarmupState::Failed)
            || matches!(self.benchmark, BenchmarkState::Failed)
    }

    const fn has_unknown_or_warming_fact(&self) -> bool {
        matches!(self.route_validation, ContractValidationState::Unknown)
            || matches!(self.schema_validation, ContractValidationState::Unknown)
            || matches!(self.manifest_readiness, ManifestReadinessState::Unknown)
            || matches!(
                self.warmup,
                WarmupState::Unknown | WarmupState::Cold | WarmupState::Warming
            )
            || matches!(self.benchmark, BenchmarkState::Unknown)
    }

    const fn pressure_with_capacity(&self, max_in_flight: u32) -> PressureLevel {
        if self.active_in_flight >= max_in_flight && self.queue_depth > 0 {
            return PressureLevel::Critical;
        }
        if self.active_in_flight >= max_in_flight || self.queue_depth >= max_in_flight {
            return PressureLevel::High;
        }
        if self.active_in_flight.saturating_mul(2) >= max_in_flight || self.queue_depth > 0 {
            return PressureLevel::Medium;
        }
        PressureLevel::Low
    }

    const fn pressure_without_capacity(&self) -> PressureLevel {
        if self.queue_depth >= 64 {
            return PressureLevel::High;
        }
        if self.active_in_flight > 0 || self.queue_depth > 0 {
            return PressureLevel::Medium;
        }
        PressureLevel::Low
    }
}
