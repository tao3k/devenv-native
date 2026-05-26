use std::{error::Error, path::Path};

use serde_json::json;
use xiuxian_qianji_bpmn_engine::{
    BpmnTaskIoSpec, BpmnTaskOutputBinding, PendingHostWork, PendingHostWorkKind,
};
use xiuxian_qianji_control::{
    ControlEvent, ControlEventKind, ControlLedger, HotStateStore, InMemoryControlLedger,
    InMemoryHotStateStore, RunId, TaskQueue, WorkerActivityHotStateMirrorRequest, WorkerId,
    WorkerRef, mirror_worker_activity_tasks_to_hot_state,
    record_admitted_activity_task_schedule_idempotent,
};

use crate::{
    FLOWHUB_SERVICE_ACTIVITY_TYPE, FlowhubScenarioIdRef, FlowhubServiceActivityHttpScheduleInput,
    FlowhubServiceActivityScheduleInput, QianjiBpmnPendingHostWorkHttpResponse,
    QianjiBpmnWorkflowTaskCompletionHttpKind, QianjiBpmnWorkflowTaskCompletionKind,
    QianjiRuntimeBpmnInstanceIdRef, QianjiRuntimeInstantMs,
    build_flowhub_service_activity_schedule_record,
    build_flowhub_service_activity_schedule_record_from_http_pending_work,
    build_flowhub_service_task_complete_http_request,
    build_flowhub_service_task_completion_payload,
    build_flowhub_service_task_contract_activity_result,
    build_flowhub_service_task_contract_completion_data,
};

#[test]
fn flowhub_service_activity_adapter_builds_control_task() -> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("flowhub-agent-coding")?;
    let work = flowhub_service_work();
    let record = build_flowhub_service_activity_schedule_record(adapter_input(&run_id, &work))?;

    assert_eq!(record.run_id, run_id);
    assert!(record.step_id.is_none());
    assert_eq!(record.occurred_at_ms, 42);
    assert_eq!(
        record.task.activity_type.as_str(),
        FLOWHUB_SERVICE_ACTIVITY_TYPE
    );
    assert_eq!(record.task.task_queue.as_str(), "flowhub.agent-coding");
    assert_eq!(
        record.task.activity_id.as_str(),
        "flowhub.flowhub_agent_coding_service_boundary.agent_coding.resolve_project.7"
    );
    assert_eq!(
        record.task.idempotency_key.as_str(),
        "idempotency.flowhub.flowhub_agent_coding_service_boundary.agent_coding.resolve_project.7"
    );
    assert_eq!(
        record
            .task
            .input_ref
            .as_ref()
            .map(|input_ref| input_ref.artifact_kind.as_str()),
        Some("bpmn.pending_host_work")
    );

    let metadata = &record.task.metadata["qianji_flowhub_service_task"];
    assert_eq!(
        metadata["schema"],
        "xiuxian_qianji.flowhub.service_activity_task.v1"
    );
    assert_eq!(metadata["scenarioId"], "agent-coding");
    assert_eq!(metadata["processId"], "agent_coding");
    assert_eq!(metadata["activityId"], "resolve_project");
    assert_eq!(metadata["tokenId"], 7);
    assert_eq!(
        metadata["completion"]["requiredOutputs"],
        json!([{
            "name": "projectResolved",
            "targetRef": "flowhub.resolveProject",
            "required": true
        }])
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn flowhub_service_activity_adapter_replays_into_worker_queue() -> Result<(), Box<dyn Error>>
{
    let ledger = InMemoryControlLedger::new();
    let hot_state = InMemoryHotStateStore::new();
    let run_id = RunId::new("flowhub-agent-coding-queue")?;
    seed_control_run(&ledger, &run_id)?;
    let work = flowhub_service_work();
    let task_queue = TaskQueue::new("flowhub.agent-coding")?;

    let record = build_flowhub_service_activity_schedule_record(adapter_input(&run_id, &work))?;
    let first = record_admitted_activity_task_schedule_idempotent(&ledger, record.clone())?;
    let duplicate = record_admitted_activity_task_schedule_idempotent(&ledger, record)?;

    assert_eq!(
        first.status,
        xiuxian_qianji_control::ActivityJournalWriteStatus::Appended
    );
    assert_eq!(
        duplicate.status,
        xiuxian_qianji_control::ActivityJournalWriteStatus::AlreadyRecorded
    );
    let worker_tasks = ledger.load_worker_activity_tasks(&run_id, Some(&task_queue))?;
    assert_eq!(worker_tasks.len(), 1);
    assert_eq!(worker_tasks[0].task_queue, task_queue);
    assert_eq!(
        worker_tasks[0].metadata["qianji_flowhub_service_task"]["activityId"],
        "resolve_project"
    );

    let mirror = mirror_worker_activity_tasks_to_hot_state(
        &ledger,
        &hot_state,
        WorkerActivityHotStateMirrorRequest::new(run_id)
            .with_task_queue(task_queue.clone())
            .with_priority(9)
            .with_metadata(json!({"flowhubMirror": true})),
    )
    .await?;
    assert_eq!(mirror.mirrored_count, 1);

    let leased = hot_state
        .claim_activity_task(worker_ref()?, Some(&task_queue), 42, 1_000)
        .await?
        .ok_or("mirrored Flowhub service task should be claimable")?;
    assert_eq!(leased.activity_task.priority, 9);
    assert_eq!(leased.activity_task.metadata["flowhubMirror"], true);
    assert_eq!(
        leased.activity_task.task.metadata["qianji_flowhub_service_task"]["completion"]["httpKind"],
        "service"
    );
    let completion = build_flowhub_service_task_completion_payload(
        &leased.activity_task.task,
        json!({"projectResolved": true}),
    )?;
    assert_eq!(completion.token_id, 7);
    assert_eq!(completion.process_id.as_ref(), "agent_coding");
    assert_eq!(completion.activity_id.as_ref(), "resolve_project");
    assert_eq!(
        completion.kind,
        QianjiBpmnWorkflowTaskCompletionKind::Service
    );
    assert_eq!(completion.data["projectResolved"], true);
    let completion_request = build_flowhub_service_task_complete_http_request(
        &leased.activity_task.task,
        json!({"projectResolved": true}),
    )?;
    assert_eq!(
        completion_request.bpmn_path,
        Path::new("qianji-flowhub/plan/agent-coding.bpmn")
    );
    assert_eq!(
        completion_request.completion.kind,
        QianjiBpmnWorkflowTaskCompletionHttpKind::Service
    );
    assert_eq!(completion_request.completion.token_id, 7);
    assert_eq!(
        completion_request.completion.process_id.as_ref(),
        "agent_coding"
    );
    assert_eq!(
        completion_request.completion.activity_id.as_ref(),
        "resolve_project"
    );
    assert_eq!(completion_request.completion.data["projectResolved"], true);

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn flowhub_service_activity_adapter_derives_contract_completion_data()
-> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("flowhub-agent-coding-contract-executor")?;
    seed_control_run(&ledger, &run_id)?;
    let work = flowhub_service_work();
    let record = build_flowhub_service_activity_schedule_record(adapter_input(&run_id, &work))?;
    record_admitted_activity_task_schedule_idempotent(&ledger, record)?;
    let task = ledger
        .load_worker_activity_tasks(&run_id, None)?
        .into_iter()
        .next()
        .ok_or_else(|| std::io::Error::other("missing Flowhub worker task"))?;

    let data = build_flowhub_service_task_contract_completion_data(&task)?;
    let result = build_flowhub_service_task_contract_activity_result(&task)?;

    assert_eq!(data["projectResolved"], true);
    assert!(result.output_ref.is_none());
    assert!(result.output_hash.is_none());
    assert_eq!(
        result.metadata["qianji_flowhub_service_completion"]["data"]["projectResolved"],
        true
    );
    Ok(())
}

#[test]
fn flowhub_service_activity_adapter_rejects_non_service_work() -> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("flowhub-agent-coding-non-service")?;
    let mut work = flowhub_service_work();
    work.kind = PendingHostWorkKind::User;

    let error = build_flowhub_service_activity_schedule_record(adapter_input(&run_id, &work))
        .err()
        .ok_or("non-service work should fail")?;

    assert!(
        error
            .to_string()
            .contains("only accepts service work, got `user`"),
        "unexpected adapter error: {error}"
    );
    Ok(())
}

#[test]
fn flowhub_service_completion_adapter_rejects_missing_output() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("flowhub-agent-coding-missing-output")?;
    seed_control_run(&ledger, &run_id)?;
    let work = flowhub_service_work();
    let record = build_flowhub_service_activity_schedule_record(adapter_input(&run_id, &work))?;
    record_admitted_activity_task_schedule_idempotent(&ledger, record)?;
    let task = ledger
        .load_worker_activity_tasks(&run_id, None)?
        .into_iter()
        .next()
        .ok_or_else(|| std::io::Error::other("missing Flowhub worker task"))?;

    let error = build_flowhub_service_task_completion_payload(&task, json!({}))
        .err()
        .ok_or_else(|| std::io::Error::other("missing output should fail"))?;

    assert!(
        error
            .to_string()
            .contains("missing required output `projectResolved`"),
        "unexpected completion adapter error: {error}"
    );
    Ok(())
}

#[test]
fn flowhub_service_activity_adapter_accepts_http_pending_work() -> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("flowhub-agent-coding-http-pending")?;
    let work = flowhub_service_work();
    let http_work: QianjiBpmnPendingHostWorkHttpResponse = serde_json::from_value(json!({
        "token_id": work.token_id,
        "process_id": "agent_coding",
        "node_index": work.node_index,
        "activity_id": "resolve_project",
        "kind": "service",
        "work_id": work.work_id,
        "form": null,
        "assignment": null,
        "lane": null,
        "task_io": work.task_io,
        "claim": null
    }))?;

    let record = build_flowhub_service_activity_schedule_record_from_http_pending_work(
        FlowhubServiceActivityHttpScheduleInput {
            run_id: &run_id,
            occurred_at_ms: QianjiRuntimeInstantMs::from_millis(42),
            scenario_id: FlowhubScenarioIdRef::new("agent-coding"),
            instance_id: QianjiRuntimeBpmnInstanceIdRef::new(
                "flowhub_agent_coding_service_boundary",
            ),
            bpmn_source: Path::new("qianji-flowhub/plan/agent-coding.bpmn"),
            pending_work: &http_work,
        },
    )?;

    assert_eq!(
        record.task.metadata["qianji_flowhub_service_task"]["activityId"],
        "resolve_project"
    );
    assert_eq!(
        record.task.metadata["qianji_flowhub_service_task"]["completion"]["requiredOutputs"][0]["name"],
        "projectResolved"
    );
    Ok(())
}

fn seed_control_run(ledger: &InMemoryControlLedger, run_id: &RunId) -> Result<(), Box<dyn Error>> {
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "flowhub service adapter replay".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    Ok(())
}

fn adapter_input<'a>(
    run_id: &'a RunId,
    pending_work: &'a PendingHostWork,
) -> FlowhubServiceActivityScheduleInput<'a> {
    FlowhubServiceActivityScheduleInput {
        run_id,
        occurred_at_ms: QianjiRuntimeInstantMs::from_millis(42),
        scenario_id: FlowhubScenarioIdRef::new("agent-coding"),
        instance_id: QianjiRuntimeBpmnInstanceIdRef::new("flowhub_agent_coding_service_boundary"),
        bpmn_source: Path::new("qianji-flowhub/plan/agent-coding.bpmn"),
        pending_work,
    }
}

fn flowhub_service_work() -> PendingHostWork {
    PendingHostWork {
        token_id: 7,
        process_id: Some("agent_coding".into()),
        node_index: 3,
        activity_id: Some("resolve_project".into()),
        kind: PendingHostWorkKind::Service,
        decision: None,
        lane: None,
        script_format: None,
        script_body: None,
        human_task_form: None,
        human_task_assignment: None,
        task_io: Some(
            BpmnTaskIoSpec::new().with_output(BpmnTaskOutputBinding::new(
                "projectResolved",
                "flowhub.resolveProject",
            )),
        ),
        claim: None,
        event_reference: None,
        event_name: None,
        work_id: Some("work-resolve-project".into()),
    }
}

fn worker_ref() -> Result<WorkerRef, Box<dyn Error>> {
    Ok(WorkerRef {
        worker_id: WorkerId::new("worker-flowhub-service")?,
        capabilities: vec!["flowhub.service".to_owned()],
        metadata: serde_json::Value::Null,
    })
}
