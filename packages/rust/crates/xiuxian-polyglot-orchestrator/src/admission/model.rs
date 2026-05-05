//! Admission budget and decision contracts.

use serde::{Deserialize, Serialize};

use crate::evidence::{PressureLevel, ReadinessState};
use crate::lanes::PolyglotLane;

/// Input budget used to make a lane admission decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AdmissionBudget {
    /// Lane receiving the admission decision.
    pub lane: PolyglotLane,
    /// Optional maximum number of in-flight requests.
    pub max_in_flight: Option<u32>,
    /// Current number of in-flight requests.
    pub active_in_flight: u32,
    /// Current queue depth observed by the owner package.
    pub queue_depth: u32,
    /// Current readiness state.
    pub readiness: ReadinessState,
    /// Current pressure level.
    pub pressure: PressureLevel,
    /// Whether the owner package can fall back.
    pub fallback_available: bool,
}

impl AdmissionBudget {
    /// Creates an admission budget for a lane.
    #[must_use]
    pub const fn new(lane: PolyglotLane) -> Self {
        Self {
            lane,
            max_in_flight: None,
            active_in_flight: 0,
            queue_depth: 0,
            readiness: ReadinessState::Unknown,
            pressure: PressureLevel::Unknown,
            fallback_available: false,
        }
    }

    /// Returns a conservative admission decision for this budget.
    #[must_use]
    pub const fn decide(self) -> AdmissionDecision {
        if matches!(self.readiness, ReadinessState::Disabled) {
            return AdmissionDecision::Reject {
                lane: self.lane,
                reason: RejectionReason::LaneDisabled,
            };
        }

        if self.pressure.rejects_new_work() {
            return AdmissionDecision::Reject {
                lane: self.lane,
                reason: RejectionReason::PressureCritical,
            };
        }

        if !self.readiness.accepts_normal_traffic() {
            return AdmissionDecision::Queue {
                lane: self.lane,
                reason: QueueReason::NotReady,
                queue_depth: self.queue_depth,
            };
        }

        if let Some(max_in_flight) = self.max_in_flight {
            if max_in_flight == 0 {
                return AdmissionDecision::Reject {
                    lane: self.lane,
                    reason: RejectionReason::NoCapacity,
                };
            }

            if self.active_in_flight >= max_in_flight {
                return AdmissionDecision::Queue {
                    lane: self.lane,
                    reason: QueueReason::AtCapacity,
                    queue_depth: self.queue_depth,
                };
            }

            return AdmissionDecision::Allow {
                lane: self.lane,
                remaining_permits: max_in_flight - self.active_in_flight,
            };
        }

        AdmissionDecision::Allow {
            lane: self.lane,
            remaining_permits: u32::MAX,
        }
    }
}

/// Result of lane admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "decision")]
pub enum AdmissionDecision {
    /// Work may be dispatched to the lane.
    Allow {
        /// Lane receiving work.
        lane: PolyglotLane,
        /// Remaining permit count after admitting the request.
        remaining_permits: u32,
    },
    /// Work should wait behind the owner package queue.
    Queue {
        /// Lane receiving work.
        lane: PolyglotLane,
        /// Queue reason.
        reason: QueueReason,
        /// Current queue depth observed by the owner package.
        queue_depth: u32,
    },
    /// Work should be rejected or sent to fallback by the owner package.
    Reject {
        /// Lane receiving work.
        lane: PolyglotLane,
        /// Rejection reason.
        reason: RejectionReason,
    },
}

/// Reason a request should remain queued.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueReason {
    /// The lane is not ready for normal traffic.
    NotReady,
    /// The lane has no permits available right now.
    AtCapacity,
}

/// Reason a request should be rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectionReason {
    /// The lane is disabled.
    LaneDisabled,
    /// The lane is under critical pressure.
    PressureCritical,
    /// The lane has no configured capacity.
    NoCapacity,
}
