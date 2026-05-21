use crate::{
    AdmissionBudget, AdmissionDecision, PolyglotLane, PressureLevel, QueueReason, ReadinessState,
    RejectionReason,
};

#[test]
fn ready_lane_with_capacity_allows_work() {
    let budget = AdmissionBudget {
        lane: PolyglotLane::PythonDocling,
        max_in_flight: Some(4),
        active_in_flight: 1,
        queue_depth: 0,
        readiness: ReadinessState::Ready,
        pressure: PressureLevel::Medium,
        fallback_available: true,
    };

    assert_eq!(
        budget.decide(),
        AdmissionDecision::Allow {
            lane: PolyglotLane::PythonDocling,
            remaining_permits: 3,
        }
    );
}

#[test]
fn warming_lane_queues_work() {
    let budget = AdmissionBudget {
        lane: PolyglotLane::JuliaCompute,
        readiness: ReadinessState::Warming,
        queue_depth: 2,
        ..AdmissionBudget::new(PolyglotLane::JuliaCompute)
    };

    assert_eq!(
        budget.decide(),
        AdmissionDecision::Queue {
            lane: PolyglotLane::JuliaCompute,
            reason: QueueReason::NotReady,
            queue_depth: 2,
        }
    );
}

#[test]
fn critical_pressure_rejects_work() {
    let budget = AdmissionBudget {
        lane: PolyglotLane::PythonDocling,
        readiness: ReadinessState::Ready,
        pressure: PressureLevel::Critical,
        ..AdmissionBudget::new(PolyglotLane::PythonDocling)
    };

    assert_eq!(
        budget.decide(),
        AdmissionDecision::Reject {
            lane: PolyglotLane::PythonDocling,
            reason: RejectionReason::PressureCritical,
        }
    );
}
