use std::error::Error;

use xiuxian_qianji_control::{
    ActivityFailure, ActivityId, ActivityResult, ActivityRetryPolicy, ActivityTask, ActivityType,
    AdmittedLlmActivityScheduleRecord, ArtifactId, ArtifactKind, ArtifactRef, ControlEvent,
    ControlEventKind, ControlLedger, ErrorCode, HotStateStore, IdempotencyKey,
    InMemoryControlLedger, InMemoryHotStateStore, LlmActivityAdmission, LlmActivityRequest,
    LlmActivityTask, LlmModelId, RecoveryItemScope, RunId, StepId, TaskQueue,
    WorkerActivityHotStateMirrorRequest, WorkerId, WorkerRef,
    mirror_worker_activity_tasks_to_hot_state, record_admitted_llm_activity_schedule,
};

#[test]
fn activity_queue_projection_selects_only_scheduled_tasks() -> Result<(), Box<dyn Error>> {
    let ledger = activity_queue_fixture()?;
    let run_id = RunId::new("run-activity-queue")?;
    let projection = ledger.load_activity_queue_projection(&run_id, None)?;

    assert_eq!(projection.run_id, run_id);
    assert_eq!(projection.task_queue, None);
    assert_eq!(projection.items.len(), 2);
    assert_eq!(projection.summary.total, 4);
    assert_eq!(projection.summary.scheduled, 2);
    assert_eq!(projection.summary.in_flight, 0);
    assert_eq!(projection.summary.completed, 1);
    assert_eq!(projection.summary.failed, 1);
    assert_eq!(
        projection.items[0].activity.activity_id,
        ActivityId::new("activity-run-scheduled")?
    );
    assert_eq!(projection.items[0].scope, RecoveryItemScope::run());
    assert_eq!(
        projection.items[1].activity.activity_id,
        ActivityId::new("activity-step-scheduled")?
    );
    assert_eq!(
        projection.items[1].scope,
        RecoveryItemScope::step(StepId::new("step-activity-queue")?)
    );
    assert_eq!(projection.worker_tasks.len(), 2);
    assert_eq!(
        projection.worker_tasks[0].activity_id,
        ActivityId::new("activity-run-scheduled")?
    );
    assert_eq!(projection.worker_tasks[0].run_id, run_id);
    assert_eq!(projection.worker_tasks[0].step_id, None);
    assert_eq!(projection.worker_tasks[0].next_attempt, 1);
    assert_eq!(projection.worker_tasks[0].scheduled_at_ms, 3);
    assert_eq!(
        projection.worker_tasks[1].step_id,
        Some(StepId::new("step-activity-queue")?)
    );
    assert_eq!(
        projection.worker_tasks[1].task_queue,
        TaskQueue::new("tool.github")?
    );
    assert_eq!(projection.worker_tasks[1].timeout_ms, Some(45_000));
    assert_eq!(
        projection.worker_tasks[1]
            .input_ref
            .as_ref()
            .map(|input| input.artifact_id.as_str()),
        Some("input-activity-step-scheduled")
    );
    assert_eq!(
        projection.worker_tasks[1].metadata["source"],
        "activity_queue"
    );
    assert_eq!(
        ledger.load_worker_activity_tasks(&run_id, None)?,
        projection.worker_tasks
    );
    Ok(())
}

#[test]
fn activity_queue_projection_filters_by_task_queue() -> Result<(), Box<dyn Error>> {
    let ledger = activity_queue_fixture()?;
    let run_id = RunId::new("run-activity-queue")?;
    let task_queue = TaskQueue::new("tool.github")?;
    let projection = ledger.load_activity_queue_projection(&run_id, Some(&task_queue))?;

    assert_eq!(projection.task_queue, Some(task_queue));
    assert_eq!(projection.items.len(), 1);
    assert_eq!(projection.summary.total, 2);
    assert_eq!(projection.summary.scheduled, 1);
    assert_eq!(projection.summary.completed, 0);
    assert_eq!(projection.summary.failed, 1);
    assert_eq!(
        projection.items[0].activity.activity_id,
        ActivityId::new("activity-step-scheduled")?
    );
    assert_eq!(projection.worker_tasks.len(), 1);
    assert_eq!(
        projection.worker_tasks[0].idempotency_key,
        IdempotencyKey::new("activity-step-scheduled/key")?
    );
    Ok(())
}

#[test]
fn activity_queue_projection_preserves_llm_request_audit_metadata() -> Result<(), Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-llm-queue-audit-projection")?;
    let activity_id = ActivityId::new("activity-llm-queue-audit-projection")?;
    let task_queue = TaskQueue::new("llm.openai")?;
    let prompt_ref = input_ref("llm-prompt")?;
    let mut task = ActivityTask::new(
        activity_id.clone(),
        ActivityType::new("llm.plan")?,
        task_queue.clone(),
        IdempotencyKey::new("activity-llm-queue-audit-projection/key")?,
    )
    .with_input_ref(prompt_ref.clone())
    .with_timeout_ms(30_000);
    task.metadata = serde_json::json!({
        "projection": "worker-task",
    });
    let request = LlmActivityRequest::new(LlmModelId::new("openai/gpt-5.2")?, prompt_ref)
        .with_context_ref(input_ref("llm-context")?)
        .with_max_tokens(1024);
    let admission = LlmActivityAdmission::from_activity(LlmActivityTask::new(task, request))?;

    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "project llm audit metadata".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    record_admitted_llm_activity_schedule(
        &ledger,
        AdmittedLlmActivityScheduleRecord::run(run_id.clone(), 2, admission),
    )?;

    let projection = ledger.load_activity_queue_projection(&run_id, Some(&task_queue))?;

    assert_eq!(projection.items.len(), 1);
    assert_eq!(projection.worker_tasks.len(), 1);
    assert_eq!(projection.worker_tasks[0].activity_id, activity_id);
    assert_eq!(
        projection.worker_tasks[0].metadata["projection"],
        "worker-task"
    );
    let item_audit = &projection.items[0]
        .activity
        .task
        .as_ref()
        .ok_or_else(|| std::io::Error::other("missing projected activity task"))?
        .metadata["qianji_llm_activity_request"];
    let worker_audit = &projection.worker_tasks[0].metadata["qianji_llm_activity_request"];
    assert_eq!(item_audit, worker_audit);
    assert_eq!(
        worker_audit["schema"],
        "qianji.llm_activity_request_audit.v1"
    );
    assert_eq!(worker_audit["model"], "openai/gpt-5.2");
    assert_eq!(
        worker_audit["prompt_ref"]["artifact_id"],
        "input-llm-prompt"
    );
    assert_eq!(
        worker_audit["context_ref"]["artifact_id"],
        "input-llm-context"
    );
    assert_eq!(worker_audit["max_tokens"], 1024);

    Ok(())
}

#[tokio::test]
async fn worker_activity_hot_state_mirror_enqueues_replay_derived_tasks()
-> Result<(), Box<dyn Error>> {
    let ledger = activity_queue_fixture()?;
    let hot_state = InMemoryHotStateStore::new();
    let run_id = RunId::new("run-activity-queue")?;
    let task_queue = TaskQueue::new("tool.github")?;
    let request = WorkerActivityHotStateMirrorRequest::new(run_id.clone())
        .with_task_queue(task_queue.clone())
        .with_priority(17)
        .with_not_before_ms(50)
        .with_metadata(serde_json::json!({"mirror": "unit"}));

    let outcome =
        mirror_worker_activity_tasks_to_hot_state(&ledger, &hot_state, request.clone()).await?;
    let repeated_outcome =
        mirror_worker_activity_tasks_to_hot_state(&ledger, &hot_state, request).await?;

    assert_eq!(outcome.run_id, run_id);
    assert_eq!(outcome.task_queue, Some(task_queue.clone()));
    assert_eq!(outcome.mirrored_count, 1);
    assert_eq!(repeated_outcome.mirrored_count, 1);

    let worker = WorkerRef {
        worker_id: WorkerId::new("worker-activity-mirror")?,
        capabilities: vec!["tool".to_owned()],
        metadata: serde_json::Value::Null,
    };
    assert!(
        hot_state
            .claim_activity_task(worker.clone(), Some(&task_queue), 49, 10)
            .await?
            .is_none()
    );
    let leased = hot_state
        .claim_activity_task(worker, Some(&task_queue), 50, 10)
        .await?
        .ok_or_else(|| std::io::Error::other("missing mirrored activity task"))?;

    assert_eq!(
        leased.activity_task.task.activity_id,
        ActivityId::new("activity-step-scheduled")?
    );
    assert_eq!(leased.activity_task.priority, 17);
    assert_eq!(leased.activity_task.not_before_ms, 50);
    assert_eq!(leased.activity_task.metadata["mirror"], "unit");
    assert!(
        hot_state
            .claim_activity_task(
                WorkerRef {
                    worker_id: WorkerId::new("worker-activity-mirror-2")?,
                    capabilities: vec!["tool".to_owned()],
                    metadata: serde_json::Value::Null,
                },
                Some(&task_queue),
                51,
                10,
            )
            .await?
            .is_none()
    );
    Ok(())
}

fn activity_queue_fixture() -> Result<InMemoryControlLedger, Box<dyn Error>> {
    let ledger = InMemoryControlLedger::new();
    let run_id = RunId::new("run-activity-queue")?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        1,
        ControlEventKind::RunCreated {
            intent: "activity queue projection".to_owned(),
            budget: None,
            metadata: serde_json::Value::Null,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        StepId::new("step-activity-queue")?,
        2,
        ControlEventKind::StepCreated {
            title: "Dispatch tool work".to_owned(),
            required_evidence: Vec::new(),
            budget: None,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        3,
        ControlEventKind::ActivityScheduled {
            task: activity_task("activity-run-scheduled", "llm.plan", "llm.openai")?,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        4,
        ControlEventKind::ActivityScheduled {
            task: activity_task("activity-run-started", "llm.plan", "llm.openai")?,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        5,
        ControlEventKind::ActivityStarted {
            activity_id: ActivityId::new("activity-run-started")?,
            worker_id: None,
            attempt: 1,
        },
    ))?;
    ledger.append_event(ControlEvent::run(
        run_id.clone(),
        6,
        ControlEventKind::ActivityCompleted {
            activity_id: ActivityId::new("activity-run-started")?,
            result: ActivityResult {
                output_ref: None,
                output_hash: Some("sha256:activity-run-started".to_owned()),
                metadata: serde_json::Value::Null,
            },
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        StepId::new("step-activity-queue")?,
        7,
        ControlEventKind::ActivityScheduled {
            task: activity_task("activity-step-scheduled", "tool.github", "tool.github")?,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        StepId::new("step-activity-queue")?,
        8,
        ControlEventKind::ActivityScheduled {
            task: activity_task("activity-step-failed", "tool.github", "tool.github")?,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id.clone(),
        StepId::new("step-activity-queue")?,
        9,
        ControlEventKind::ActivityStarted {
            activity_id: ActivityId::new("activity-step-failed")?,
            worker_id: None,
            attempt: 1,
        },
    ))?;
    ledger.append_event(ControlEvent::step(
        run_id,
        StepId::new("step-activity-queue")?,
        10,
        ControlEventKind::ActivityFailed {
            activity_id: ActivityId::new("activity-step-failed")?,
            failure: ActivityFailure {
                error_code: ErrorCode::new("rate_limited")?,
                message: "provider rejected request".to_owned(),
                retryable: true,
                attempt: 1,
                metadata: serde_json::Value::Null,
            },
        },
    ))?;
    Ok(ledger)
}

fn activity_task(
    activity_id: &str,
    activity_type: &str,
    task_queue: &str,
) -> Result<ActivityTask, Box<dyn Error>> {
    let mut task = ActivityTask::new(
        ActivityId::new(activity_id)?,
        ActivityType::new(activity_type)?,
        TaskQueue::new(task_queue)?,
        IdempotencyKey::new(format!("{activity_id}/key"))?,
    )
    .with_input_ref(input_ref(activity_id)?)
    .with_retry_policy(ActivityRetryPolicy::new(3)?.with_initial_interval_ms(100))
    .with_timeout_ms(45_000);
    task.metadata = serde_json::json!({"source": "activity_queue"});
    Ok(task)
}

fn input_ref(activity_id: &str) -> Result<ArtifactRef, Box<dyn Error>> {
    Ok(ArtifactRef {
        artifact_id: ArtifactId::new(format!("input-{activity_id}"))?,
        artifact_kind: ArtifactKind::new("claim_check")?,
        uri: format!("artifact://input-{activity_id}"),
        content_digest: None,
        metadata: serde_json::Value::Null,
    })
}
