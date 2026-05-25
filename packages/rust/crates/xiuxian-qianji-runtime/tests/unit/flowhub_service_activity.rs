use std::{error::Error, path::Path};

use serde_json::json;
use xiuxian_qianji_bpmn_engine::{
    BpmnTaskIoSpec, BpmnTaskOutputBinding, PendingHostWork, PendingHostWorkKind,
};
use xiuxian_qianji_control::{RunId, WorkerActivityTask};
use xiuxian_qianji_runtime::{
    FLOWHUB_SERVICE_ACTIVITY_TYPE, FlowhubScenarioIdRef, FlowhubServiceActivityScheduleInput,
    QianjiRuntimeBpmnInstanceIdRef, QianjiRuntimeInstantMs,
    build_flowhub_service_activity_schedule_record, build_flowhub_service_task_completion,
    build_flowhub_service_task_contract_activity_result,
    build_flowhub_service_task_contract_completion_data, flowhub_service_task_bpmn_source_path,
};

#[test]
fn flowhub_service_schedule_record_preserves_bpmn_identity() -> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("flowhub-agent-coding")?;
    let work = flowhub_service_work();
    let record = build_flowhub_service_activity_schedule_record(adapter_input(&run_id, &work))?;

    assert_eq!(record.run_id, run_id);
    assert_eq!(record.occurred_at_ms, 42);
    assert!(record.step_id.is_none());
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

#[test]
fn flowhub_service_schedule_record_rejects_non_service_work() -> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("flowhub-agent-coding")?;
    let mut work = flowhub_service_work();
    work.kind = PendingHostWorkKind::User;

    let Err(error) = build_flowhub_service_activity_schedule_record(adapter_input(&run_id, &work))
    else {
        return Err("non-service work should be rejected".into());
    };

    assert!(
        error
            .to_string()
            .contains("only accepts service work, got `user`")
    );
    Ok(())
}

#[test]
fn flowhub_service_schedule_record_requires_stable_bpmn_identity() -> Result<(), Box<dyn Error>> {
    let run_id = RunId::new("flowhub-agent-coding")?;
    let mut work = flowhub_service_work();
    work.activity_id = None;

    let Err(error) = build_flowhub_service_activity_schedule_record(adapter_input(&run_id, &work))
    else {
        return Err("missing activity id should be rejected".into());
    };

    assert!(error.to_string().contains("requires a BPMN activity id"));
    Ok(())
}

#[test]
fn flowhub_service_completion_preserves_required_bpmn_identity() -> Result<(), Box<dyn Error>> {
    let task = flowhub_worker_task()?;

    let completion =
        build_flowhub_service_task_completion(&task, json!({"projectResolved": true}))?;

    assert_eq!(completion.token_id.as_u64(), 7);
    assert_eq!(completion.process_id.as_str(), "agent_coding");
    assert_eq!(completion.activity_id.as_str(), "resolve_project");
    assert_eq!(completion.data["projectResolved"], true);
    assert_eq!(
        flowhub_service_task_bpmn_source_path(&task)?,
        Path::new("qianji-flowhub/plan/agent-coding.bpmn")
    );
    Ok(())
}

#[test]
fn flowhub_service_contract_executor_derives_activity_result() -> Result<(), Box<dyn Error>> {
    let task = flowhub_worker_task()?;

    let data = build_flowhub_service_task_contract_completion_data(&task)?;
    let result = build_flowhub_service_task_contract_activity_result(&task)?;

    assert_eq!(data["projectResolved"], true);
    assert!(result.output_ref.is_none());
    assert!(result.output_hash.is_none());
    assert_eq!(
        result.metadata["qianji_flowhub_service_completion"]["schema"],
        "xiuxian_qianji.flowhub.service_completion.v1"
    );
    assert_eq!(
        result.metadata["qianji_flowhub_service_completion"]["data"]["projectResolved"],
        true
    );
    Ok(())
}

#[test]
fn flowhub_service_completion_rejects_missing_required_output() -> Result<(), Box<dyn Error>> {
    let task = flowhub_worker_task()?;

    let Err(error) = build_flowhub_service_task_completion(&task, json!({})) else {
        return Err("missing required output should be rejected".into());
    };

    assert!(
        error
            .to_string()
            .contains("missing required output `projectResolved`")
    );
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

fn flowhub_worker_task() -> Result<WorkerActivityTask, Box<dyn Error>> {
    let run_id = RunId::new("flowhub-agent-coding-worker")?;
    let work = flowhub_service_work();
    let record = build_flowhub_service_activity_schedule_record(adapter_input(&run_id, &work))?;
    let task = record.task;
    Ok(WorkerActivityTask {
        run_id,
        step_id: None,
        activity_id: task.activity_id,
        activity_type: task.activity_type,
        task_queue: task.task_queue,
        next_attempt: 1,
        scheduled_at_ms: record.occurred_at_ms,
        input_ref: task.input_ref,
        idempotency_key: task.idempotency_key,
        retry_policy: task.retry_policy,
        timeout_ms: task.timeout_ms,
        metadata: task.metadata,
    })
}

fn flowhub_service_work() -> PendingHostWork {
    PendingHostWork {
        token_id: 7,
        process_id: Some("agent_coding".into()),
        node_index: 11,
        activity_id: Some("resolve_project".into()),
        kind: PendingHostWorkKind::Service,
        decision: None,
        lane: None,
        script_format: None,
        script_body: None,
        human_task_form: None,
        human_task_assignment: None,
        task_io: Some(BpmnTaskIoSpec {
            inputs: Vec::new(),
            outputs: vec![BpmnTaskOutputBinding {
                name: "projectResolved".into(),
                target_ref: "flowhub.resolveProject".into(),
                required: true,
            }],
        }),
        claim: None,
        event_reference: None,
        event_name: None,
        work_id: Some("work.resolve_project.7".into()),
    }
}
