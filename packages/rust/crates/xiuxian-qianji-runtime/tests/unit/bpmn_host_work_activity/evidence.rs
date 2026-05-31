use std::error::Error;

use serde_json::json;
use xiuxian_qianji_bpmn_engine::PendingHostWorkKind;
use xiuxian_qianji_control::{
    ActivityStatus, ControlLedger, ErrorCode, InMemoryControlLedger, RunId, WorkerId,
};
use xiuxian_qianji_runtime::{
    BpmnHostWorkCompletionActivityEvidenceInput, BpmnHostWorkFailure,
    BpmnHostWorkFailureActivityEvidenceInput, record_bpmn_host_work_completion_activity_evidence,
    record_bpmn_host_work_failure_activity_evidence,
};

use super::support::{
    activity_evidence_event_kinds, evidence_input, host_work_completion, pending_work,
    single_activity_status,
};

#[test]
fn bpmn_host_work_evidence_recorder_records_completion_sequence() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("bpmn-host-work-evidence")?;
    let work = pending_work(PendingHostWorkKind::Service);
    let worker_id = WorkerId::new("qianji-runtime-test-worker")?;
    let completion = host_work_completion();

    record_bpmn_host_work_completion_activity_evidence(
        &ledger,
        BpmnHostWorkCompletionActivityEvidenceInput {
            evidence: evidence_input(&run_id, &work, &worker_id),
            completion: &completion,
        },
    )?;

    assert_eq!(
        activity_evidence_event_kinds(&ledger, &run_id)?,
        vec![
            "run_created",
            "activity_scheduled",
            "activity_started",
            "activity_completed",
        ]
    );
    assert_eq!(
        single_activity_status(&ledger, &run_id)?,
        ActivityStatus::Completed
    );
    Ok(())
}

#[test]
fn bpmn_host_work_evidence_recorder_records_failure_sequence() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("bpmn-host-work-failure-evidence")?;
    let work = pending_work(PendingHostWorkKind::Service);
    let worker_id = WorkerId::new("qianji-runtime-test-worker")?;

    record_bpmn_host_work_failure_activity_evidence(
        &ledger,
        BpmnHostWorkFailureActivityEvidenceInput {
            evidence: evidence_input(&run_id, &work, &worker_id),
            failure: BpmnHostWorkFailure {
                error_code: ErrorCode::new("native_host_failed")?,
                message: "native host failed".to_owned(),
                retryable: true,
                metadata: json!({"source": "runtime-test"}),
            },
        },
    )?;

    assert_eq!(
        activity_evidence_event_kinds(&ledger, &run_id)?,
        vec![
            "run_created",
            "activity_scheduled",
            "activity_started",
            "activity_failed",
        ]
    );
    assert_eq!(
        single_activity_status(&ledger, &run_id)?,
        ActivityStatus::Failed
    );
    Ok(())
}

#[test]
fn bpmn_host_work_evidence_recorder_rejects_blank_failure_without_partial_events()
-> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("bpmn-host-work-blank-failure")?;
    let work = pending_work(PendingHostWorkKind::Service);
    let worker_id = WorkerId::new("qianji-runtime-test-worker")?;

    let Err(error) = record_bpmn_host_work_failure_activity_evidence(
        &ledger,
        BpmnHostWorkFailureActivityEvidenceInput {
            evidence: evidence_input(&run_id, &work, &worker_id),
            failure: BpmnHostWorkFailure {
                error_code: ErrorCode::new("native_host_failed")?,
                message: " ".to_owned(),
                retryable: true,
                metadata: json!({"source": "runtime-test"}),
            },
        },
    ) else {
        return Err("blank failure should be rejected".into());
    };

    assert!(error.to_string().contains("must not be blank"));
    assert!(ledger.load_events(&run_id)?.is_empty());
    Ok(())
}
