use std::{error::Error, path::Path};

use serde_json::json;
use xiuxian_qianji_bpmn_engine::{
    BpmnTaskIoSpec, BpmnTaskOutputBinding, PendingHostWork, PendingHostWorkKind,
};
use xiuxian_qianji_control::{
    ActivityStatus, ControlEventKind, ControlLedger, InMemoryControlLedger, RunId, WorkerId,
};
use xiuxian_qianji_runtime::{
    BpmnHostWorkActivityEvidenceInput, BpmnHostWorkActivityScheduleInput, BpmnHostWorkCompletion,
    BpmnHostWorkCompletionKind, BpmnHostWorkIdentity, QianjiRuntimeBpmnActivityId,
    QianjiRuntimeBpmnInstanceIdRef, QianjiRuntimeBpmnProcessId, QianjiRuntimeBpmnTokenId,
    QianjiRuntimeInstantMs,
};

pub(crate) fn adapter_input<'a>(
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

pub(crate) fn evidence_input<'a>(
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

pub(crate) fn host_work_completion() -> BpmnHostWorkCompletion {
    BpmnHostWorkCompletion {
        token_id: QianjiRuntimeBpmnTokenId::new(9),
        process_id: QianjiRuntimeBpmnProcessId::new("Process_1"),
        activity_id: QianjiRuntimeBpmnActivityId::new("Task_Review"),
        kind: BpmnHostWorkCompletionKind::Service,
        data: json!({"approved": true}),
        claimant: Some("worker-1".to_owned()),
    }
}

pub(crate) fn host_work_identity(kind: PendingHostWorkKind) -> BpmnHostWorkIdentity {
    BpmnHostWorkIdentity::new(
        QianjiRuntimeBpmnTokenId::new(9),
        QianjiRuntimeBpmnProcessId::new("Process_1"),
        QianjiRuntimeBpmnActivityId::new("Task_Review"),
        kind,
    )
}

pub(crate) fn activity_evidence_event_kinds(
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

pub(crate) fn single_activity_status(
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

pub(crate) fn pending_work(kind: PendingHostWorkKind) -> PendingHostWork {
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
