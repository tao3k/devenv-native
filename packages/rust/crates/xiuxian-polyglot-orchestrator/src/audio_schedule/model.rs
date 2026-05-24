//! Pure scheduling contracts for Python audio shard work.
//!
//! The scheduler translates owner-supplied audio shard facts into an inert
//! plan. It does not call Python, mutate queues, or know which model backend
//! will execute the shard.

use serde::{Deserialize, Serialize};

use crate::{
    AdmissionDecision, LaneCapability, PolyglotLane, PressureLevel, QueueReason, RejectionReason,
    WorkerPressureEvidence,
};

const TARGET_INITIAL_AUDIO_SHARD_WAVES: u32 = 3;

/// Scheduling action recommended for audio shard work.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioScheduleAction {
    /// Dispatch work to the owner package's existing analyzer lane.
    Dispatch,
    /// Keep work queued in the owner package.
    Queue,
    /// Reject the lane and let the owner package report the rejection.
    Reject,
    /// Use the owner package's configured fallback path.
    Fallback,
}

/// Reason attached to an audio scheduling action.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioScheduleReason {
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

impl From<QueueReason> for AudioScheduleReason {
    fn from(reason: QueueReason) -> Self {
        match reason {
            QueueReason::NotReady => Self::NotReady,
            QueueReason::AtCapacity => Self::AtCapacity,
        }
    }
}

impl From<RejectionReason> for AudioScheduleReason {
    fn from(reason: RejectionReason) -> Self {
        match reason {
            RejectionReason::LaneDisabled => Self::LaneDisabled,
            RejectionReason::PressureCritical => Self::PressureCritical,
            RejectionReason::NoCapacity => Self::NoCapacity,
        }
    }
}

/// Owner-supplied input for an inert audio scheduling plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AudioSchedulingInput {
    /// Worker-pressure evidence provided by the owner package.
    pub pressure: WorkerPressureEvidence,
    /// Optional worker count requested by the caller.
    pub requested_workers: Option<u32>,
    /// Optional caller-local maximum worker cap.
    pub max_worker_cap: Option<u32>,
    /// Optional adaptive worker budget already selected by the owner package.
    pub adaptive_worker_budget: Option<u32>,
    /// Number of audio shards waiting to be scheduled.
    pub shard_count: u32,
}

impl AudioSchedulingInput {
    /// Creates scheduling input for audio shard transcription.
    #[must_use]
    pub const fn audio_shards(pressure: WorkerPressureEvidence) -> Self {
        Self {
            pressure,
            requested_workers: None,
            max_worker_cap: None,
            adaptive_worker_budget: None,
            shard_count: 1,
        }
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
    pub fn plan(self) -> AudioSchedulePlan {
        let admission = self.pressure.to_admission_budget().decide();
        let pressure = self.pressure.pressure_level();

        match admission {
            AdmissionDecision::Allow {
                lane,
                remaining_permits,
            } => self.dispatch_plan(lane, remaining_permits, pressure),
            AdmissionDecision::Queue { reason, .. } => AudioSchedulePlan {
                capability: self.pressure.capability,
                action: AudioScheduleAction::Queue,
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
    ) -> AudioSchedulePlan {
        let recommended_workers = self.recommended_workers(remaining_permits);
        if recommended_workers == 0 {
            return self.rejection_plan(
                lane,
                AudioScheduleReason::NoCapacity,
                AdmissionDecision::Reject {
                    lane,
                    reason: RejectionReason::NoCapacity,
                },
                pressure,
            );
        }

        AudioSchedulePlan {
            capability: self.pressure.capability,
            action: AudioScheduleAction::Dispatch,
            reason: AudioScheduleReason::CapacityAvailable,
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
        reason: AudioScheduleReason,
        admission: AdmissionDecision,
        pressure: PressureLevel,
    ) -> AudioSchedulePlan {
        let action = if self.pressure.fallback_available {
            AudioScheduleAction::Fallback
        } else {
            AudioScheduleAction::Reject
        };

        AudioSchedulePlan {
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
        let max_worker_cap = self
            .max_worker_cap
            .or(self.pressure.max_in_flight)
            .unwrap_or_else(|| remaining_permits.min(self.normalized_shard_count()))
            .max(1);
        let automatic_workers = self.adaptive_worker_budget.unwrap_or_else(|| {
            initial_worker_budget(max_worker_cap, self.normalized_shard_count())
        });
        let requested_workers = self
            .requested_workers
            .map_or(automatic_workers, |requested_workers| {
                requested_workers.max(1)
            });

        requested_workers
            .min(max_worker_cap)
            .min(automatic_workers.max(1))
            .min(remaining_permits)
            .min(self.normalized_shard_count())
    }

    const fn normalized_shard_count(self) -> u32 {
        if self.shard_count == 0 {
            1
        } else {
            self.shard_count
        }
    }
}

/// Inert schedule plan returned to the owner package.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AudioSchedulePlan {
    /// Capability described by the plan.
    pub capability: LaneCapability,
    /// Recommended action.
    pub action: AudioScheduleAction,
    /// Action reason.
    pub reason: AudioScheduleReason,
    /// Underlying lane admission decision.
    pub admission: AdmissionDecision,
    /// Pressure level observed during planning.
    pub pressure: PressureLevel,
    /// Recommended worker count for the next analyzer request.
    pub recommended_workers: u32,
    /// Maximum shard count to dispatch in one wave.
    pub shard_wave_size: u32,
}

fn initial_worker_budget(max_worker_bound: u32, shard_count: u32) -> u32 {
    let max_worker_bound = max_worker_bound.max(1);
    let system_floor = ceil_sqrt_u32(max_worker_bound);
    let wave_floor = shard_count
        .max(1)
        .div_ceil(TARGET_INITIAL_AUDIO_SHARD_WAVES);
    system_floor.max(wave_floor).min(max_worker_bound).max(1)
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
