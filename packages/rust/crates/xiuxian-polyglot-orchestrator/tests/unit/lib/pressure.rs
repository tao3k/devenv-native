use crate::{
    AdmissionDecision, LaneCapability, PolyglotLane, PressureLevel, ReadinessState,
    RejectionReason, WorkerPressureEvidence,
};

#[test]
fn document_pressure_projects_to_admission_budget() {
    let evidence = WorkerPressureEvidence::document_extraction()
        .with_worker_budget(Some(4), 1)
        .with_queue_depth(0)
        .with_fallback_available(true);

    let budget = evidence.to_admission_budget();

    assert_eq!(evidence.lane, PolyglotLane::PythonDocling);
    assert_eq!(evidence.capability, LaneCapability::DocumentExtraction);
    assert_eq!(evidence.pressure_level(), PressureLevel::Low);
    assert_eq!(budget.readiness, ReadinessState::Ready);
    assert_eq!(
        budget.decide(),
        AdmissionDecision::Allow {
            lane: PolyglotLane::PythonDocling,
            remaining_permits: 3,
        }
    );
}

#[test]
fn queued_capacity_projects_critical_pressure() {
    let evidence = WorkerPressureEvidence::document_extraction()
        .with_worker_budget(Some(4), 4)
        .with_queue_depth(1);

    assert_eq!(evidence.pressure_level(), PressureLevel::Critical);
    assert_eq!(
        evidence.to_admission_budget().decide(),
        AdmissionDecision::Reject {
            lane: PolyglotLane::PythonDocling,
            reason: RejectionReason::PressureCritical,
        }
    );
}

#[test]
fn ordering_backlog_escalates_ocr_pressure() {
    let evidence = WorkerPressureEvidence::ocr_shard_extraction()
        .with_worker_budget(Some(8), 2)
        .with_ordering_backlog(8);

    assert_eq!(evidence.pressure_level(), PressureLevel::High);
    assert_eq!(
        evidence.to_lane_evidence().readiness,
        ReadinessState::Degraded
    );
}

#[test]
fn pressure_evidence_serializes_capability_and_counts() -> Result<(), serde_json::Error> {
    let evidence = WorkerPressureEvidence::ocr_shard_extraction()
        .with_worker_budget(Some(8), 2)
        .with_queue_depth(5)
        .with_failures(1, 1)
        .with_ordering_backlog(3);

    let serialized = serde_json::to_string(&evidence)?;

    assert!(serialized.contains("ocr_shard_extraction"));
    assert!(serialized.contains("queued_items"));
    assert!(serialized.contains("ordering_backlog"));
    Ok(())
}
