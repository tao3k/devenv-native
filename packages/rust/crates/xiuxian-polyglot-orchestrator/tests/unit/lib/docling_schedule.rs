use crate::{
    AdmissionDecision, DoclingScheduleAction, DoclingScheduleReason, DoclingSchedulingInput,
    DoclingWorkerPolicy, LaneCapability, PolyglotLane, PressureLevel, QueueReason,
    WorkerPressureEvidence,
};

#[test]
fn dispatch_plan_bounds_workers_by_permits_request_cap_and_shards() {
    let pressure = WorkerPressureEvidence::ocr_shard_extraction()
        .with_worker_budget(Some(8), 3)
        .with_queue_depth(0);

    let plan = DoclingSchedulingInput::ocr_shards(pressure)
        .with_worker_request(Some(6), Some(4))
        .with_shard_count(3)
        .plan();

    assert_eq!(plan.capability, LaneCapability::OcrShardExtraction);
    assert_eq!(plan.action, DoclingScheduleAction::Dispatch);
    assert_eq!(plan.reason, DoclingScheduleReason::CapacityAvailable);
    assert_eq!(plan.pressure, PressureLevel::Low);
    assert_eq!(plan.recommended_workers, 3);
    assert_eq!(plan.shard_wave_size, 3);
    assert_eq!(
        plan.admission,
        AdmissionDecision::Allow {
            lane: PolyglotLane::PythonDocling,
            remaining_permits: 5,
        }
    );
}

#[test]
fn critical_pressure_uses_fallback_when_available() {
    let pressure = WorkerPressureEvidence::document_extraction()
        .with_worker_budget(Some(4), 4)
        .with_queue_depth(1)
        .with_fallback_available(true);

    let plan = DoclingSchedulingInput::document_extraction(pressure).plan();

    assert_eq!(plan.action, DoclingScheduleAction::Fallback);
    assert_eq!(plan.reason, DoclingScheduleReason::PressureCritical);
    assert_eq!(plan.recommended_workers, 0);
    assert_eq!(plan.shard_wave_size, 0);
}

#[test]
fn at_capacity_remains_queued() {
    let pressure = WorkerPressureEvidence::document_extraction()
        .with_worker_budget(Some(4), 4)
        .with_queue_depth(0);

    let plan = DoclingSchedulingInput::document_extraction(pressure).plan();

    assert_eq!(plan.action, DoclingScheduleAction::Queue);
    assert_eq!(plan.reason, DoclingScheduleReason::AtCapacity);
    assert_eq!(
        plan.admission,
        AdmissionDecision::Queue {
            lane: PolyglotLane::PythonDocling,
            reason: QueueReason::AtCapacity,
            queue_depth: 0,
        }
    );
}

#[test]
fn critical_pressure_rejects_without_fallback() {
    let pressure = WorkerPressureEvidence::document_extraction()
        .with_worker_budget(Some(2), 2)
        .with_queue_depth(1);

    let plan = DoclingSchedulingInput::document_extraction(pressure).plan();

    assert_eq!(plan.action, DoclingScheduleAction::Reject);
    assert_eq!(plan.reason, DoclingScheduleReason::PressureCritical);
}

#[test]
fn serialization_uses_snake_case_action_and_reason() {
    let pressure = WorkerPressureEvidence::document_extraction().with_worker_budget(Some(2), 0);
    let plan = DoclingSchedulingInput::document_extraction(pressure).plan();

    let serialized = serde_json::to_string(&plan).expect("serialize docling schedule plan");

    assert!(serialized.contains("\"action\":\"dispatch\""));
    assert!(serialized.contains("\"reason\":\"capacity_available\""));
}

#[test]
fn source_pdf_page_range_policy_recommends_milestone_worker_budget() {
    let pressure = WorkerPressureEvidence::ocr_shard_extraction()
        .with_worker_budget(Some(12), 0)
        .with_queue_depth(0);

    let plan = DoclingSchedulingInput::ocr_shards(pressure)
        .with_worker_policy(DoclingWorkerPolicy::SourcePdfPageRange)
        .with_adaptive_worker_budget(Some(12))
        .with_worker_request(None, Some(12))
        .with_shard_count(21)
        .plan();

    assert_eq!(plan.action, DoclingScheduleAction::Dispatch);
    assert_eq!(plan.recommended_workers, 4);
    assert_eq!(plan.shard_wave_size, 4);
}

#[test]
fn source_pdf_page_range_policy_respects_pressure_reduced_budget() {
    let pressure = WorkerPressureEvidence::ocr_shard_extraction()
        .with_worker_budget(Some(12), 0)
        .with_queue_depth(0);

    let plan = DoclingSchedulingInput::ocr_shards(pressure)
        .with_worker_policy(DoclingWorkerPolicy::SourcePdfPageRange)
        .with_adaptive_worker_budget(Some(2))
        .with_worker_request(None, Some(12))
        .with_shard_count(21)
        .plan();

    assert_eq!(plan.recommended_workers, 2);
    assert_eq!(plan.shard_wave_size, 2);
}

#[test]
fn source_pdf_page_range_policy_keeps_fixed_worker_override_diagnostic() {
    let pressure = WorkerPressureEvidence::ocr_shard_extraction()
        .with_worker_budget(Some(12), 0)
        .with_queue_depth(0);

    let plan = DoclingSchedulingInput::ocr_shards(pressure)
        .with_worker_policy(DoclingWorkerPolicy::SourcePdfPageRange)
        .with_adaptive_worker_budget(Some(12))
        .with_worker_request(Some(99), Some(12))
        .with_shard_count(3)
        .plan();

    assert_eq!(plan.recommended_workers, 3);
    assert_eq!(plan.shard_wave_size, 3);
}

#[test]
fn source_pdf_page_range_override_respects_adaptive_budget() {
    let pressure = WorkerPressureEvidence::ocr_shard_extraction()
        .with_worker_budget(Some(12), 0)
        .with_queue_depth(0);

    let plan = DoclingSchedulingInput::ocr_shards(pressure)
        .with_worker_policy(DoclingWorkerPolicy::SourcePdfPageRange)
        .with_adaptive_worker_budget(Some(2))
        .with_worker_request(Some(99), Some(12))
        .with_shard_count(21)
        .plan();

    assert_eq!(plan.recommended_workers, 2);
    assert_eq!(plan.shard_wave_size, 2);
}
