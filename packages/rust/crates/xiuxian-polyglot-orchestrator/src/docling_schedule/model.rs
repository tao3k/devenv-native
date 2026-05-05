//! Pure scheduling contracts for Python Docling work.
//!
//! The scheduler translates owner-supplied pressure evidence into an inert
//! plan. It does not mutate queues, launch workers, or call Python.

use serde::{Deserialize, Serialize};

use crate::{
    AdmissionDecision, LaneCapability, PolyglotLane, PressureLevel, QueueReason, RejectionReason,
    WorkerPressureEvidence,
};

/// Scheduling action recommended for Docling work.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoclingScheduleAction {
    /// Dispatch work to the owner package's existing Docling lane.
    Dispatch,
    /// Keep work queued in the owner package.
    Queue,
    /// Reject the lane and let the owner package report the rejection.
    Reject,
    /// Use the owner package's configured fallback path.
    Fallback,
}

/// Reason attached to a Docling scheduling action.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoclingScheduleReason {
    /// Capacity is available for immediate dispatch.
    CapacityAvailable,
    /// The lane is not ready for normal traffic.
    NotReady,
    /// The lane is ready but has no permits available right now.
    AtCapacity,
    /// The lane is under critical pressure.
    PressureCritical,
    /// The lane is disabled by its owner package.
    LaneDisabled,
    /// The caller or owner package reported no usable capacity.
    NoCapacity,
}

/// Worker policy used to translate owner facts into a worker recommendation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DoclingWorkerPolicy {
    /// Use the caller-supplied worker request and clamp it to available facts.
    Direct,
    /// Automatically size source-PDF page-range OCR waves from system facts.
    SourcePdfPageRange,
}

impl From<QueueReason> for DoclingScheduleReason {
    fn from(reason: QueueReason) -> Self {
        match reason {
            QueueReason::NotReady => Self::NotReady,
            QueueReason::AtCapacity => Self::AtCapacity,
        }
    }
}

impl From<RejectionReason> for DoclingScheduleReason {
    fn from(reason: RejectionReason) -> Self {
        match reason {
            RejectionReason::LaneDisabled => Self::LaneDisabled,
            RejectionReason::PressureCritical => Self::PressureCritical,
            RejectionReason::NoCapacity => Self::NoCapacity,
        }
    }
}

/// Owner-supplied input for an inert Docling scheduling plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DoclingSchedulingInput {
    /// Worker-pressure evidence provided by the owner package.
    pub pressure: WorkerPressureEvidence,
    /// Worker sizing policy used by this schedule.
    pub worker_policy: DoclingWorkerPolicy,
    /// Optional worker count requested by the caller.
    pub requested_workers: Option<u32>,
    /// Optional caller-local maximum worker cap.
    pub max_worker_cap: Option<u32>,
    /// Optional adaptive worker budget already selected by the owner package.
    pub adaptive_worker_budget: Option<u32>,
    /// Number of document or OCR shards waiting to be scheduled.
    pub shard_count: u32,
}

impl DoclingSchedulingInput {
    /// Creates scheduling input for full-document extraction.
    #[must_use]
    pub const fn document_extraction(pressure: WorkerPressureEvidence) -> Self {
        Self {
            pressure,
            worker_policy: DoclingWorkerPolicy::Direct,
            requested_workers: None,
            max_worker_cap: None,
            adaptive_worker_budget: None,
            shard_count: 1,
        }
    }

    /// Creates scheduling input for OCR shard extraction.
    #[must_use]
    pub const fn ocr_shards(pressure: WorkerPressureEvidence) -> Self {
        Self {
            pressure,
            worker_policy: DoclingWorkerPolicy::Direct,
            requested_workers: None,
            max_worker_cap: None,
            adaptive_worker_budget: None,
            shard_count: 1,
        }
    }

    /// Returns this input with a worker sizing policy.
    #[must_use]
    pub const fn with_worker_policy(mut self, worker_policy: DoclingWorkerPolicy) -> Self {
        self.worker_policy = worker_policy;
        self
    }

    /// Returns this input with caller-local worker request bounds.
    #[must_use]
    pub const fn with_worker_request(
        mut self,
        requested_workers: Option<u32>,
        max_worker_cap: Option<u32>,
    ) -> Self {
        self.requested_workers = requested_workers;
        self.max_worker_cap = max_worker_cap;
        self
    }

    /// Returns this input with an owner-supplied adaptive worker budget.
    #[must_use]
    pub const fn with_adaptive_worker_budget(
        mut self,
        adaptive_worker_budget: Option<u32>,
    ) -> Self {
        self.adaptive_worker_budget = adaptive_worker_budget;
        self
    }

    /// Returns this input with a shard count.
    #[must_use]
    pub const fn with_shard_count(mut self, shard_count: u32) -> Self {
        self.shard_count = shard_count;
        self
    }

    /// Computes an inert scheduling plan.
    #[must_use]
    pub fn plan(self) -> DoclingSchedulePlan {
        let admission = self.pressure.to_admission_budget().decide();
        let pressure = self.pressure.pressure_level();

        match admission {
            AdmissionDecision::Allow {
                lane,
                remaining_permits,
            } => self.dispatch_plan(lane, remaining_permits, pressure),
            AdmissionDecision::Queue { reason, .. } => DoclingSchedulePlan {
                capability: self.pressure.capability,
                action: DoclingScheduleAction::Queue,
                reason: reason.into(),
                admission,
                pressure,
                recommended_workers: 0,
                shard_wave_size: 0,
            },
            AdmissionDecision::Reject { lane, reason } => {
                self.rejection_plan(lane, reason.into(), admission, pressure)
            }
        }
    }

    fn dispatch_plan(
        self,
        lane: PolyglotLane,
        remaining_permits: u32,
        pressure: PressureLevel,
    ) -> DoclingSchedulePlan {
        let recommended_workers = self.recommended_workers(remaining_permits);
        if recommended_workers == 0 {
            return self.rejection_plan(
                lane,
                DoclingScheduleReason::NoCapacity,
                AdmissionDecision::Reject {
                    lane,
                    reason: RejectionReason::NoCapacity,
                },
                pressure,
            );
        }

        DoclingSchedulePlan {
            capability: self.pressure.capability,
            action: DoclingScheduleAction::Dispatch,
            reason: DoclingScheduleReason::CapacityAvailable,
            admission: AdmissionDecision::Allow {
                lane,
                remaining_permits,
            },
            pressure,
            recommended_workers,
            shard_wave_size: recommended_workers.min(self.normalized_shard_count()),
        }
    }

    fn rejection_plan(
        self,
        _lane: PolyglotLane,
        reason: DoclingScheduleReason,
        admission: AdmissionDecision,
        pressure: PressureLevel,
    ) -> DoclingSchedulePlan {
        let action = if self.pressure.fallback_available {
            DoclingScheduleAction::Fallback
        } else {
            DoclingScheduleAction::Reject
        };

        DoclingSchedulePlan {
            capability: self.pressure.capability,
            action,
            reason,
            admission,
            pressure,
            recommended_workers: 0,
            shard_wave_size: 0,
        }
    }

    fn recommended_workers(self, remaining_permits: u32) -> u32 {
        let requested_workers = self.requested_workers.map_or_else(
            || self.automatic_worker_request(),
            |requested_workers| requested_workers.max(1),
        );
        let max_worker_cap = self
            .max_worker_cap
            .or(self.pressure.max_in_flight)
            .unwrap_or(requested_workers)
            .max(1);
        let adaptive_worker_cap = match self.worker_policy {
            DoclingWorkerPolicy::Direct => max_worker_cap,
            DoclingWorkerPolicy::SourcePdfPageRange => {
                self.adaptive_worker_budget.unwrap_or(max_worker_cap).max(1)
            }
        };
        requested_workers
            .min(max_worker_cap)
            .min(adaptive_worker_cap)
            .min(remaining_permits)
            .min(self.normalized_shard_count())
    }

    fn automatic_worker_request(self) -> u32 {
        match self.worker_policy {
            DoclingWorkerPolicy::Direct => 1,
            DoclingWorkerPolicy::SourcePdfPageRange => {
                let max_worker_bound = self
                    .max_worker_cap
                    .or(self.pressure.max_in_flight)
                    .unwrap_or(1)
                    .max(1);
                let adaptive_budget = self
                    .adaptive_worker_budget
                    .or(self.pressure.max_in_flight)
                    .unwrap_or(1)
                    .max(1);
                let machine_budget = ceil_sqrt_u32(max_worker_bound);
                let page_budget = self.normalized_shard_count().div_ceil(6).max(1);
                adaptive_budget.min(machine_budget).min(page_budget).max(1)
            }
        }
    }

    fn normalized_shard_count(self) -> u32 {
        self.shard_count.max(1)
    }
}

fn ceil_sqrt_u32(value: u32) -> u32 {
    if value <= 1 {
        return value;
    }
    let mut root = 1u32;
    while root.saturating_mul(root) < value {
        root = root.saturating_add(1);
    }
    root
}

/// Inert scheduling plan for Docling work.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DoclingSchedulePlan {
    /// Capability covered by this plan.
    pub capability: LaneCapability,
    /// Recommended action.
    pub action: DoclingScheduleAction,
    /// Reason for the action.
    pub reason: DoclingScheduleReason,
    /// Admission decision used to derive the action.
    pub admission: AdmissionDecision,
    /// Coarse pressure level used by the plan.
    pub pressure: PressureLevel,
    /// Recommended worker count for this scheduling wave.
    pub recommended_workers: u32,
    /// Recommended shard count for this scheduling wave.
    pub shard_wave_size: u32,
}
