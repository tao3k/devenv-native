#[cfg(feature = "audio-shard-arrow")]
use super::{
    AudioShardScheduleRequest, audio_shard_contract_versions, audio_shard_pressure_evidence,
    audio_shard_schedule_plan,
};
#[cfg(feature = "pdf-source-range")]
use super::{
    pdf_ocr_shard_contract_snapshot, pdf_ocr_shard_input_ref, pdf_ocr_shard_pressure_evidence,
    pdf_ocr_shard_pressure_snapshot, pdf_ocr_shard_result_ref, pdf_ocr_shard_schedule_plan,
    pdf_ocr_source_range_shard_schedule_plan,
};
#[cfg(feature = "audio-shard-arrow")]
use crate::audio::{AUDIO_SHARD_INPUT_SCHEMA_VERSION, AUDIO_SHARD_RESULT_SCHEMA_VERSION};
#[cfg(feature = "pdf-source-range")]
use crate::pdf::ocr::{
    PDF_OCR_DEFAULT_PROFILE, PDF_OCR_SHARD_INPUT_SCHEMA_VERSION,
    PDF_OCR_SHARD_RESULT_SCHEMA_VERSION,
};
#[cfg(feature = "pdf-source-range")]
use xiuxian_polyglot_orchestrator::{
    AdmissionDecision, ContractOwner, DoclingScheduleAction, DoclingScheduleReason, PolyglotLane,
    PressureLevel, RejectionReason,
};
#[cfg(feature = "audio-shard-arrow")]
use xiuxian_polyglot_orchestrator::{AudioScheduleAction, AudioScheduleReason};

#[cfg(feature = "pdf-source-range")]
#[test]
fn ocr_input_ref_preserves_attachment_schema_owner() {
    let reference = pdf_ocr_shard_input_ref("/analysis/pdf-ocr-shards");

    assert_eq!(reference.lane, PolyglotLane::PythonDocling);
    assert_eq!(reference.owner, ContractOwner::Attachments);
    assert_eq!(reference.route, "/analysis/pdf-ocr-shards");
    assert_eq!(reference.profile.as_deref(), Some(PDF_OCR_DEFAULT_PROFILE));
    assert_eq!(
        reference.schema_version.as_deref(),
        Some(PDF_OCR_SHARD_INPUT_SCHEMA_VERSION)
    );
}

#[cfg(feature = "pdf-source-range")]
#[test]
fn ocr_result_ref_preserves_attachment_schema_owner() {
    let reference = pdf_ocr_shard_result_ref("/analysis/pdf-ocr-shards");

    assert_eq!(reference.lane, PolyglotLane::PythonDocling);
    assert_eq!(reference.owner, ContractOwner::Attachments);
    assert_eq!(reference.route, "/analysis/pdf-ocr-shards");
    assert_eq!(reference.profile.as_deref(), Some(PDF_OCR_DEFAULT_PROFILE));
    assert_eq!(
        reference.schema_version.as_deref(),
        Some(PDF_OCR_SHARD_RESULT_SCHEMA_VERSION)
    );
}

#[cfg(feature = "pdf-source-range")]
#[test]
fn ocr_contract_snapshot_materializes_input_and_result_refs() {
    let snapshot = match pdf_ocr_shard_contract_snapshot("/analysis/pdf-ocr-shards") {
        Ok(snapshot) => snapshot,
        Err(error) => panic!("OCR shard snapshot should validate: {error}"),
    };

    assert_eq!(snapshot.route_refs().len(), 2);
    assert!(
        snapshot
            .route_refs()
            .iter()
            .all(|reference| reference.owner == ContractOwner::Attachments)
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

#[cfg(feature = "pdf-source-range")]
#[test]
fn ocr_pressure_snapshot_projects_queue_and_ordering_backlog() {
    let pressure = pdf_ocr_shard_pressure_evidence(Some(4), 4, 2, 0, 0, 8, true);

    let snapshot = match pdf_ocr_shard_pressure_snapshot("/analysis/pdf-ocr-shards", pressure) {
        Ok(snapshot) => snapshot,
        Err(error) => panic!("OCR pressure snapshot should validate: {error}"),
    };

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

#[cfg(feature = "pdf-source-range")]
#[test]
fn ocr_schedule_plan_uses_orchestrator_policy() {
    let pressure = pdf_ocr_shard_pressure_evidence(Some(8), 1, 0, 0, 0, 0, false);

    let plan = pdf_ocr_shard_schedule_plan(pressure, Some(6), Some(4), 5);

    assert_eq!(plan.action, DoclingScheduleAction::Dispatch);
    assert_eq!(plan.reason, DoclingScheduleReason::CapacityAvailable);
    assert_eq!(plan.recommended_workers, 4);
    assert_eq!(plan.shard_wave_size, 4);
}

#[cfg(feature = "pdf-source-range")]
#[test]
fn source_range_schedule_plan_uses_orchestrator_auto_policy() {
    let pressure = pdf_ocr_shard_pressure_evidence(Some(12), 0, 0, 0, 0, 0, false);

    let plan = pdf_ocr_source_range_shard_schedule_plan(pressure, Some(12), None, Some(12), 21);

    assert_eq!(plan.action, DoclingScheduleAction::Dispatch);
    assert_eq!(plan.reason, DoclingScheduleReason::CapacityAvailable);
    assert_eq!(plan.recommended_workers, 3);
    assert_eq!(plan.shard_wave_size, 3);
}

#[cfg(feature = "pdf-source-range")]
#[test]
fn source_range_schedule_plan_keeps_override_diagnostic() {
    let pressure = pdf_ocr_shard_pressure_evidence(Some(12), 0, 0, 0, 0, 0, false);

    let plan = pdf_ocr_source_range_shard_schedule_plan(pressure, Some(12), Some(99), Some(12), 3);

    assert_eq!(plan.recommended_workers, 3);
    assert_eq!(plan.shard_wave_size, 3);
}

#[cfg(feature = "pdf-source-range")]
#[test]
fn source_range_schedule_plan_caps_override_by_adaptive_budget() {
    let pressure = pdf_ocr_shard_pressure_evidence(Some(12), 0, 0, 0, 0, 0, false);

    let plan = pdf_ocr_source_range_shard_schedule_plan(pressure, Some(2), Some(99), Some(12), 21);

    assert_eq!(plan.recommended_workers, 2);
    assert_eq!(plan.shard_wave_size, 2);
}

#[cfg(feature = "audio-shard-arrow")]
#[test]
fn audio_schedule_plan_uses_orchestrator_auto_policy() {
    let pressure = audio_shard_pressure_evidence(Some(12), 0, 0, 0, 0, 0, false);

    let plan = audio_shard_schedule_plan(&AudioShardScheduleRequest {
        pressure,
        adaptive_worker_budget: None,
        diagnostic_worker_override: None,
        max_worker_cap: Some(12),
        shard_count: 10,
    });

    assert_eq!(plan.action, AudioScheduleAction::Dispatch);
    assert_eq!(plan.reason, AudioScheduleReason::CapacityAvailable);
    assert_eq!(plan.recommended_workers, 4);
    assert_eq!(plan.shard_wave_size, 4);
}

#[cfg(feature = "audio-shard-arrow")]
#[test]
fn audio_schedule_plan_respects_adaptive_budget() {
    let pressure = audio_shard_pressure_evidence(Some(12), 0, 0, 0, 0, 0, false);

    let plan = audio_shard_schedule_plan(&AudioShardScheduleRequest {
        pressure,
        adaptive_worker_budget: Some(2),
        diagnostic_worker_override: Some(99),
        max_worker_cap: Some(12),
        shard_count: 10,
    });

    assert_eq!(plan.recommended_workers, 2);
    assert_eq!(plan.shard_wave_size, 2);
}

#[cfg(feature = "audio-shard-arrow")]
#[test]
fn audio_contract_versions_preserve_arrow_contract_names() {
    assert_eq!(
        audio_shard_contract_versions().input_schema_version,
        AUDIO_SHARD_INPUT_SCHEMA_VERSION
    );
    assert_eq!(
        audio_shard_contract_versions().result_schema_version,
        AUDIO_SHARD_RESULT_SCHEMA_VERSION
    );
}
