use crate::wendao_contracts::{
    DocumentExtractPressureEvidenceInput, document_extract_pressure_evidence,
    document_extract_pressure_snapshot, document_extract_route_ref, document_extract_schedule_plan,
};
use crate::{
    AdmissionDecision, ContractOwner, DoclingScheduleAction, DoclingScheduleReason, PolyglotLane,
    PressureLevel, RejectionReason, SnapshotInvariantError,
};
use xiuxian_wendao_runtime::transport::ANALYSIS_DOCUMENT_EXTRACT_ROUTE;

#[test]
fn document_extract_ref_preserves_analyzer_route() {
    let reference = document_extract_route_ref();

    assert_eq!(reference.lane, PolyglotLane::PythonDocling);
    assert_eq!(reference.owner, ContractOwner::Analyzer);
    assert_eq!(reference.route, ANALYSIS_DOCUMENT_EXTRACT_ROUTE);
    assert!(reference.profile.is_none());
    assert!(reference.schema_version.is_none());
}

#[test]
fn document_extract_pressure_snapshot_projects_supplied_counters()
-> Result<(), SnapshotInvariantError> {
    let pressure = document_extract_pressure_evidence(DocumentExtractPressureEvidenceInput {
        max_in_flight: Some(2),
        active_in_flight: 2,
        queued_items: 1,
        failed_items: 0,
        retryable_failures: 0,
        fallback_available: true,
    });

    let snapshot = document_extract_pressure_snapshot(pressure)?;

    assert_eq!(pressure.pressure_level(), PressureLevel::Critical);
    assert_eq!(snapshot.route_refs().len(), 1);
    assert_eq!(
        snapshot.admission_decision_for_lane(PolyglotLane::PythonDocling),
        Some(AdmissionDecision::Reject {
            lane: PolyglotLane::PythonDocling,
            reason: RejectionReason::PressureCritical,
        })
    );
    assert_eq!(
        snapshot
            .evidence_for_lane(PolyglotLane::PythonDocling)
            .map(|evidence| evidence.pressure),
        Some(PressureLevel::Critical)
    );
    Ok(())
}

#[test]
fn document_extract_schedule_plan_uses_orchestrator_policy() {
    let pressure = document_extract_pressure_evidence(DocumentExtractPressureEvidenceInput {
        max_in_flight: Some(6),
        active_in_flight: 2,
        queued_items: 0,
        failed_items: 0,
        retryable_failures: 0,
        fallback_available: false,
    });

    let plan = document_extract_schedule_plan(pressure, Some(5), Some(3), 4);

    assert_eq!(plan.action, DoclingScheduleAction::Dispatch);
    assert_eq!(plan.reason, DoclingScheduleReason::CapacityAvailable);
    assert_eq!(plan.recommended_workers, 3);
    assert_eq!(plan.shard_wave_size, 3);
}
