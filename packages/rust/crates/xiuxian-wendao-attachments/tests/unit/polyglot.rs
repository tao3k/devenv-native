use super::{
    pdf_ocr_shard_contract_snapshot, pdf_ocr_shard_input_ref, pdf_ocr_shard_pressure_evidence,
    pdf_ocr_shard_pressure_snapshot, pdf_ocr_shard_result_ref, pdf_ocr_shard_schedule_plan,
};
use crate::pdf::ocr::{
    PDF_OCR_DEFAULT_PROFILE, PDF_OCR_SHARD_INPUT_SCHEMA_VERSION,
    PDF_OCR_SHARD_RESULT_SCHEMA_VERSION,
};
use xiuxian_polyglot_orchestrator::{
    AdmissionDecision, ContractOwner, DoclingScheduleAction, DoclingScheduleReason, PolyglotLane,
    PressureLevel, RejectionReason,
};

#[test]
fn ocr_input_ref_preserves_attachment_schema_owner() {
    let reference = pdf_ocr_shard_input_ref("/analysis/pdf-ocr-shards");

    assert_eq!(reference.lane, PolyglotLane::PythonDocling);
    assert_eq!(reference.owner, ContractOwner::WendaoAttachments);
    assert_eq!(reference.route, "/analysis/pdf-ocr-shards");
    assert_eq!(reference.profile.as_deref(), Some(PDF_OCR_DEFAULT_PROFILE));
    assert_eq!(
        reference.schema_version.as_deref(),
        Some(PDF_OCR_SHARD_INPUT_SCHEMA_VERSION)
    );
}

#[test]
fn ocr_result_ref_preserves_attachment_schema_owner() {
    let reference = pdf_ocr_shard_result_ref("/analysis/pdf-ocr-shards");

    assert_eq!(reference.lane, PolyglotLane::PythonDocling);
    assert_eq!(reference.owner, ContractOwner::WendaoAttachments);
    assert_eq!(reference.route, "/analysis/pdf-ocr-shards");
    assert_eq!(reference.profile.as_deref(), Some(PDF_OCR_DEFAULT_PROFILE));
    assert_eq!(
        reference.schema_version.as_deref(),
        Some(PDF_OCR_SHARD_RESULT_SCHEMA_VERSION)
    );
}

#[test]
fn ocr_contract_snapshot_materializes_input_and_result_refs() {
    let snapshot = pdf_ocr_shard_contract_snapshot("/analysis/pdf-ocr-shards")
        .expect("OCR shard snapshot should validate");

    assert_eq!(snapshot.route_refs().len(), 2);
    assert!(
        snapshot
            .route_refs()
            .iter()
            .all(|reference| reference.owner == ContractOwner::WendaoAttachments)
    );
    assert!(
        snapshot
            .route_refs()
            .iter()
            .any(|reference| reference.schema_version.as_deref()
                == Some(PDF_OCR_SHARD_INPUT_SCHEMA_VERSION))
    );
    assert!(
        snapshot
            .route_refs()
            .iter()
            .any(|reference| reference.schema_version.as_deref()
                == Some(PDF_OCR_SHARD_RESULT_SCHEMA_VERSION))
    );
}

#[test]
fn ocr_pressure_snapshot_projects_queue_and_ordering_backlog() {
    let pressure = pdf_ocr_shard_pressure_evidence(Some(4), 4, 2, 0, 0, 8, true);

    let snapshot = pdf_ocr_shard_pressure_snapshot("/analysis/pdf-ocr-shards", pressure)
        .expect("OCR pressure snapshot should validate");

    assert_eq!(pressure.pressure_level(), PressureLevel::Critical);
    assert_eq!(snapshot.route_refs().len(), 2);
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
}

#[test]
fn ocr_schedule_plan_uses_orchestrator_policy() {
    let pressure = pdf_ocr_shard_pressure_evidence(Some(8), 1, 0, 0, 0, 0, false);

    let plan = pdf_ocr_shard_schedule_plan(pressure, Some(6), Some(4), 5);

    assert_eq!(plan.action, DoclingScheduleAction::Dispatch);
    assert_eq!(plan.reason, DoclingScheduleReason::CapacityAvailable);
    assert_eq!(plan.recommended_workers, 4);
    assert_eq!(plan.shard_wave_size, 4);
}
