//! Worker pressure evidence contracts.

use serde::{Deserialize, Serialize};

use crate::admission::AdmissionBudget;
use crate::evidence::{FallbackEvidence, HealthState, LaneEvidence, PressureLevel, ReadinessState};
use crate::lanes::{LaneCapability, PolyglotLane};

/// Inert worker-pressure evidence supplied by an owner package.
///
/// The evidence records counters only. It does not observe a live worker,
/// mutate a queue, or dispatch work.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkerPressureEvidence {
    /// Lane described by this evidence.
    pub lane: PolyglotLane,
    /// Capability class being measured.
    pub capability: LaneCapability,
    /// Optional maximum number of in-flight worker requests.
    pub max_in_flight: Option<u32>,
    /// Current number of in-flight worker requests.
    pub active_in_flight: u32,
    /// Current queue depth supplied by the owner package.
    pub queued_items: u32,
    /// Recent failed worker rows supplied by the owner package.
    pub failed_items: u32,
    /// Failed rows that the owner package considers retryable.
    pub retryable_failures: u32,
    /// Items waiting on stable shard ordering or merge validation.
    pub ordering_backlog: u32,
    /// Whether the owner package can fall back if pressure rejects work.
    pub fallback_available: bool,
}

impl WorkerPressureEvidence {
    /// Creates empty pressure evidence for a capability.
    #[must_use]
    pub const fn new(capability: LaneCapability) -> Self {
        Self {
            lane: capability.owning_lane(),
            capability,
            max_in_flight: None,
            active_in_flight: 0,
            queued_items: 0,
            failed_items: 0,
            retryable_failures: 0,
            ordering_backlog: 0,
            fallback_available: false,
        }
    }

    /// Creates Python Docling document-extraction pressure evidence.
    #[must_use]
    pub const fn document_extraction() -> Self {
        Self::new(LaneCapability::DocumentExtraction)
    }

    /// Creates Python Docling OCR-shard pressure evidence.
    #[must_use]
    pub const fn ocr_shard_extraction() -> Self {
        Self::new(LaneCapability::OcrShardExtraction)
    }

    /// Returns this evidence with worker budget counters.
    #[must_use]
    pub const fn with_worker_budget(
        mut self,
        max_in_flight: Option<u32>,
        active_in_flight: u32,
    ) -> Self {
        self.max_in_flight = max_in_flight;
        self.active_in_flight = active_in_flight;
        self
    }

    /// Returns this evidence with queue depth.
    #[must_use]
    pub const fn with_queue_depth(mut self, queued_items: u32) -> Self {
        self.queued_items = queued_items;
        self
    }

    /// Returns this evidence with failure counters.
    #[must_use]
    pub const fn with_failures(mut self, failed_items: u32, retryable_failures: u32) -> Self {
        self.failed_items = failed_items;
        self.retryable_failures = retryable_failures;
        self
    }

    /// Returns this evidence with ordering backlog.
    #[must_use]
    pub const fn with_ordering_backlog(mut self, ordering_backlog: u32) -> Self {
        self.ordering_backlog = ordering_backlog;
        self
    }

    /// Returns this evidence with fallback availability.
    #[must_use]
    pub const fn with_fallback_available(mut self, fallback_available: bool) -> Self {
        self.fallback_available = fallback_available;
        self
    }

    /// Returns the coarse pressure level represented by these counters.
    #[must_use]
    pub const fn pressure_level(self) -> PressureLevel {
        match self.max_in_flight {
            Some(0) => PressureLevel::Critical,
            Some(max_in_flight) => self.pressure_with_capacity(max_in_flight),
            None => self.pressure_without_capacity(),
        }
    }

    /// Projects pressure evidence to an admission budget.
    #[must_use]
    pub const fn to_admission_budget(self) -> AdmissionBudget {
        AdmissionBudget {
            lane: self.lane,
            max_in_flight: self.max_in_flight,
            active_in_flight: self.active_in_flight,
            queue_depth: self.queued_items,
            readiness: self.readiness_state(),
            pressure: self.pressure_level(),
            fallback_available: self.fallback_available,
        }
    }

    /// Projects pressure evidence to a lane evidence envelope.
    #[must_use]
    pub const fn to_lane_evidence(self) -> LaneEvidence {
        LaneEvidence::new(
            self.lane,
            self.health_state(),
            self.readiness_state(),
            self.pressure_level(),
            FallbackEvidence::new(self.fallback_available),
        )
    }

    const fn readiness_state(self) -> ReadinessState {
        match self.max_in_flight {
            Some(0) => ReadinessState::Disabled,
            _ => match self.pressure_level() {
                PressureLevel::Critical => ReadinessState::Degraded,
                PressureLevel::High => ReadinessState::Degraded,
                _ => ReadinessState::Ready,
            },
        }
    }

    const fn health_state(self) -> HealthState {
        match self.pressure_level() {
            PressureLevel::Critical => HealthState::Unhealthy,
            PressureLevel::High => HealthState::Degraded,
            _ => HealthState::Healthy,
        }
    }

    const fn pressure_with_capacity(self, max_in_flight: u32) -> PressureLevel {
        let critical_ordering = max_in_flight.saturating_mul(2);
        if self.active_in_flight >= max_in_flight && self.queued_items > 0 {
            return PressureLevel::Critical;
        }
        if critical_ordering > 0 && self.ordering_backlog >= critical_ordering {
            return PressureLevel::Critical;
        }
        if self.failed_items > 0 && self.retryable_failures == 0 {
            return PressureLevel::High;
        }
        if self.active_in_flight >= max_in_flight
            || self.queued_items >= max_in_flight
            || self.ordering_backlog >= max_in_flight
            || self.failed_items > 0
        {
            return PressureLevel::High;
        }
        if self.active_in_flight.saturating_mul(2) >= max_in_flight
            || self.queued_items > 0
            || self.ordering_backlog > 0
        {
            return PressureLevel::Medium;
        }
        PressureLevel::Low
    }

    const fn pressure_without_capacity(self) -> PressureLevel {
        if self.queued_items >= 64 || self.ordering_backlog >= 64 || self.failed_items >= 8 {
            return PressureLevel::High;
        }
        if self.queued_items > 0
            || self.ordering_backlog > 0
            || self.failed_items > 0
            || self.active_in_flight > 0
        {
            return PressureLevel::Medium;
        }
        PressureLevel::Low
    }
}
