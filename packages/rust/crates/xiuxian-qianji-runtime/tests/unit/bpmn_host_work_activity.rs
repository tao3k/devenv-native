use std::{error::Error, path::Path};

use serde_json::json;
use xiuxian_qianji_bpmn_engine::{
    BpmnTaskIoSpec, BpmnTaskOutputBinding, PendingHostWork, PendingHostWorkKind,
};
use xiuxian_qianji_control::{
    ActivityStatus, ControlEventKind, ControlLedger, ErrorCode, InMemoryControlLedger, RunId,
    WorkerId,
};
use xiuxian_qianji_runtime::{
    BPMN_HOST_WORK_ACTIVITY_TYPE, BPMN_HOST_WORK_COMPLETION_METADATA_KEY,
    BpmnHostWorkActivityEvidenceInput, BpmnHostWorkActivityScheduleInput, BpmnHostWorkCompletion,
    BpmnHostWorkCompletionActivityEvidenceInput, BpmnHostWorkCompletionKind, BpmnHostWorkFailure,
    BpmnHostWorkFailureActivityEvidenceInput, BpmnHostWorkIdentity, QianjiRuntimeBpmnActivityId,
    QianjiRuntimeBpmnInstanceIdRef, QianjiRuntimeBpmnProcessId, QianjiRuntimeBpmnTokenId,
    QianjiRuntimeInstantMs, build_bpmn_host_work_activity_result,
    build_bpmn_host_work_activity_schedule_record, find_matching_bpmn_host_work,
    pending_bpmn_host_work_matches_identity, record_bpmn_host_work_completion_activity_evidence,
    record_bpmn_host_work_failure_activity_evidence,
};

#[test]
fn bpmn_host_work_schedule_record_preserves_identity() -> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("bpmn-host-work")?;
    let work = pending_work(PendingHostWorkKind::Service);
    let record = build_bpmn_host_work_activity_schedule_record(adapter_input(&run_id, &work))?;

    assert_eq!(record.run_id, run_id);
    assert_eq!(record.occurred_at_ms, 77);
    assert_eq!(
        record.task.activity_type.as_str(),
        BPMN_HOST_WORK_ACTIVITY_TYPE
    );
    assert_eq!(record.task.task_queue.as_str(), "bpmn.host_work.service");
    assert_eq!(
        record.task.activity_id.as_str(),
        "bpmn.instance_1.Process_1.Task_Review.9"
    );
    assert_eq!(
        record.task.idempotency_key.as_str(),
        "idempotency.bpmn.instance_1.Process_1.Task_Review.9"
    );
    assert_eq!(
        record
            .task
            .input_ref
            .as_ref()
            .map(|input| input.uri.as_str()),
        Some("bpmn://instances/instance_1/processes/Process_1/tokens/9/host-work/Task_Review")
    );

    let metadata = &record.task.metadata["qianji_bpmn_host_work_activity"];
    assert_eq!(
        metadata["schema"],
        "xiuxian_qianji.bpmn.host_work_activity.v1"
    );
    assert_eq!(metadata["instanceId"], "instance_1");
    assert_eq!(metadata["processId"], "Process_1");
    assert_eq!(metadata["activityId"], "Task_Review");
    assert_eq!(metadata["tokenId"], 9);
    assert_eq!(metadata["workKind"], "service");
    assert_eq!(
        metadata["requiredOutputs"],
        json!([{
            "name": "approved",
            "targetRef": "review.approved",
            "required": true
        }])
    );

    Ok(())
}

#[test]
fn bpmn_host_work_schedule_record_supports_business_rule_work() -> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("bpmn-host-work-business-rule")?;
    let work = pending_work(PendingHostWorkKind::BusinessRule);
    let record = build_bpmn_host_work_activity_schedule_record(adapter_input(&run_id, &work))?;

    assert_eq!(
        record.task.task_queue.as_str(),
        "bpmn.host_work.business_rule"
    );
    assert_eq!(
        record.task.metadata["qianji_bpmn_host_work_activity"]["workKind"],
        "business_rule"
    );
    Ok(())
}

#[test]
fn bpmn_host_work_schedule_record_requires_bpmn_identity() -> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("bpmn-host-work-missing-id")?;
    let mut work = pending_work(PendingHostWorkKind::User);
    work.process_id = None;

    let Err(error) = build_bpmn_host_work_activity_schedule_record(adapter_input(&run_id, &work))
    else {
        return Err("missing process id should be rejected".into());
    };

    assert!(error.to_string().contains("requires a process id"));
    Ok(())
}

#[test]
fn bpmn_host_work_completion_result_preserves_metadata_hash() -> Result<(), Box<dyn Error>> {
    let completion = BpmnHostWorkCompletion {
        token_id: QianjiRuntimeBpmnTokenId::new(9),
        process_id: QianjiRuntimeBpmnProcessId::new("Process_1"),
        activity_id: QianjiRuntimeBpmnActivityId::new("Task_Review"),
        kind: BpmnHostWorkCompletionKind::Service,
        data: json!({"approved": true}),
        claimant: Some("worker-1".to_owned()),
    };

    let result = build_bpmn_host_work_activity_result(&completion)?;

    assert!(result.output_ref.is_none());
    assert!(
        result
            .output_hash
            .as_deref()
            .is_some_and(|hash| hash.starts_with("sha256:"))
    );
    let metadata = &result.metadata[BPMN_HOST_WORK_COMPLETION_METADATA_KEY];
    assert_eq!(
        metadata["schema"],
        "xiuxian_qianji.bpmn.host_work_completion.v1"
    );
    assert_eq!(metadata["tokenId"], 9);
    assert_eq!(metadata["processId"], "Process_1");
    assert_eq!(metadata["activityId"], "Task_Review");
    assert_eq!(metadata["kind"], "service");
    assert_eq!(metadata["data"], json!({"approved": true}));
    assert_eq!(metadata["claimant"], "worker-1");

    Ok(())
}

#[test]
fn bpmn_host_work_identity_matches_exact_pending_work() {
    let work = pending_work(PendingHostWorkKind::Service);
    let identity = host_work_identity(PendingHostWorkKind::Service);

    assert!(pending_bpmn_host_work_matches_identity(&work, &identity));
    assert_eq!(
        find_matching_bpmn_host_work(std::slice::from_ref(&work), &identity)
            .map(|matched| matched.token_id),
        Some(9)
    );
}

#[test]
fn bpmn_host_work_identity_rejects_kind_mismatch() {
    let work = pending_work(PendingHostWorkKind::User);
    let identity = host_work_identity(PendingHostWorkKind::Service);

    assert!(!pending_bpmn_host_work_matches_identity(&work, &identity));
    assert!(find_matching_bpmn_host_work(std::slice::from_ref(&work), &identity).is_none());
}

#[test]
fn bpmn_host_work_identity_rejects_missing_bpmn_identity() {
    let mut work = pending_work(PendingHostWorkKind::Service);
    work.activity_id = None;
    let identity = host_work_identity(PendingHostWorkKind::Service);

    assert!(!pending_bpmn_host_work_matches_identity(&work, &identity));
    assert!(find_matching_bpmn_host_work(std::slice::from_ref(&work), &identity).is_none());
}

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

fn adapter_input<'a>(
    run_id: &'a RunId,
    pending_work: &'a PendingHostWork,
) -> BpmnHostWorkActivityScheduleInput<'a> {
    BpmnHostWorkActivityScheduleInput {
        run_id,
        occurred_at_ms: QianjiRuntimeInstantMs::from_millis(77),
        instance_id: QianjiRuntimeBpmnInstanceIdRef::new("instance_1"),
        bpmn_source: Path::new("fixtures/workflow.bpmn"),
        pending_work,
    }
}

fn evidence_input<'a>(
    run_id: &'a RunId,
    pending_work: &'a PendingHostWork,
    worker_id: &'a WorkerId,
) -> BpmnHostWorkActivityEvidenceInput<'a> {
    BpmnHostWorkActivityEvidenceInput {
        run_id,
        instance_id: QianjiRuntimeBpmnInstanceIdRef::new("instance_1"),
        bpmn_source: Path::new("fixtures/workflow.bpmn"),
        pending_work,
        worker_id,
        scheduled_at_ms: QianjiRuntimeInstantMs::from_millis(77),
        started_at_ms: QianjiRuntimeInstantMs::from_millis(78),
        terminal_at_ms: QianjiRuntimeInstantMs::from_millis(79),
    }
}

fn host_work_completion() -> BpmnHostWorkCompletion {
    BpmnHostWorkCompletion {
        token_id: QianjiRuntimeBpmnTokenId::new(9),
        process_id: QianjiRuntimeBpmnProcessId::new("Process_1"),
        activity_id: QianjiRuntimeBpmnActivityId::new("Task_Review"),
        kind: BpmnHostWorkCompletionKind::Service,
        data: json!({"approved": true}),
        claimant: Some("worker-1".to_owned()),
    }
}

fn host_work_identity(kind: PendingHostWorkKind) -> BpmnHostWorkIdentity {
    BpmnHostWorkIdentity::new(
        QianjiRuntimeBpmnTokenId::new(9),
        QianjiRuntimeBpmnProcessId::new("Process_1"),
        QianjiRuntimeBpmnActivityId::new("Task_Review"),
        kind,
    )
}

fn activity_evidence_event_kinds(
    ledger: &InMemoryControlLedger,
    run_id: &RunId,
) -> Result<Vec<&'static str>, Box<dyn Error>> {
    let records = ledger.load_events(run_id)?;
    Ok(records
        .iter()
        .map(|record| match &record.event.kind {
            ControlEventKind::RunCreated { .. } => "run_created",
            ControlEventKind::ActivityScheduled { .. } => "activity_scheduled",
            ControlEventKind::ActivityStarted { .. } => "activity_started",
            ControlEventKind::ActivityCompleted { .. } => "activity_completed",
            ControlEventKind::ActivityFailed { .. } => "activity_failed",
            other => panic!("unexpected activity evidence event: {other:?}"),
        })
        .collect())
}

fn single_activity_status(
    ledger: &InMemoryControlLedger,
    run_id: &RunId,
) -> Result<ActivityStatus, Box<dyn Error>> {
    let view = ledger.load_run_view(run_id)?;
    let activity = view
        .activities
        .values()
        .next()
        .ok_or("activity evidence should include one activity")?;
    Ok(activity.status)
}

fn pending_work(kind: PendingHostWorkKind) -> PendingHostWork {
    PendingHostWork {
        token_id: 9,
        process_id: Some("Process_1".into()),
        node_index: 12,
        activity_id: Some("Task_Review".into()),
        kind,
        decision: None,
        lane: None,
        script_format: None,
        script_body: None,
        human_task_form: None,
        human_task_assignment: None,
        task_io: Some(BpmnTaskIoSpec {
            inputs: Vec::new(),
            outputs: vec![BpmnTaskOutputBinding {
                name: "approved".into(),
                target_ref: "review.approved".into(),
                required: true,
            }],
        }),
        claim: None,
        event_reference: None,
        event_name: None,
        work_id: Some("work.Task_Review.9".into()),
    }
}
