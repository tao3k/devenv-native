use crate::qianji_cli::tests::control_cli::support::{
    activity_task, append_empty_control_run, must_ok,
};
use xiuxian_qianji_control::{
    ActivityId, ControlEvent, ControlEventKind, ControlLedger, DuckDbControlLedger, HotStateStore,
    InMemoryHotStateStore, RunId, RunnableActivityTask, WorkerActivityTask,
};

pub(super) fn registry_worker_task() -> WorkerActivityTask {
    registry_worker_task_with("llm.plan", "llm.openai")
}

pub(super) fn registry_worker_task_with(
    activity_type: &str,
    task_queue: &str,
) -> WorkerActivityTask {
    let task = activity_task(
        must_ok(
            ActivityId::new("activity-registry-fixture"),
            "should build registry activity id",
        ),
        activity_type,
        task_queue,
    );
    WorkerActivityTask {
        run_id: must_ok(RunId::new("run-registry"), "should build registry run id"),
        step_id: None,
        activity_id: task.activity_id,
        activity_type: task.activity_type,
        task_queue: task.task_queue,
        next_attempt: 1,
        scheduled_at_ms: 1_000,
        input_ref: task.input_ref,
        idempotency_key: task.idempotency_key,
        retry_policy: task.retry_policy,
        timeout_ms: task.timeout_ms,
        metadata: task.metadata,
    }
}

pub(super) fn append_control_run_with_disallowed_activity_route(
    ledger_path: &std::path::Path,
) -> RunId {
    let run_id = append_empty_control_run(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let activity_id = must_ok(
        ActivityId::new("activity-disallowed-route"),
        "should build disallowed activity id",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            2,
            ControlEventKind::ActivityScheduled {
                task: activity_task(activity_id, "provider.unknown", "llm.openai"),
            },
        )),
        "should append disallowed route activity",
    );
    run_id
}

pub(super) fn append_control_run_with_llm_route(
    ledger_path: &std::path::Path,
    activity_id: &str,
    activity_type: &str,
    task_queue: &str,
) -> RunId {
    let run_id = append_empty_control_run(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let activity_id = must_ok(
        ActivityId::new(activity_id),
        "should build governed LLM activity id",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            2,
            ControlEventKind::ActivityScheduled {
                task: activity_task(activity_id, activity_type, task_queue),
            },
        )),
        "should append governed LLM route activity",
    );
    run_id
}

pub(super) async fn enqueue_worker_task(
    ledger: &DuckDbControlLedger,
    hot_state: &InMemoryHotStateStore,
    run_id: &xiuxian_qianji_control::RunId,
    activity_id: &str,
) -> Result<(), String> {
    let task = worker_task(ledger, run_id, activity_id)?;
    must_ok(
        hot_state
            .enqueue_activity_task(RunnableActivityTask {
                task,
                priority: 10,
                not_before_ms: 7_000,
                metadata: serde_json::json!({"mirror": "worker-once"}),
            })
            .await,
        "should enqueue activity task",
    );
    Ok(())
}

fn worker_task(
    ledger: &DuckDbControlLedger,
    run_id: &xiuxian_qianji_control::RunId,
    activity_id: &str,
) -> Result<xiuxian_qianji_control::WorkerActivityTask, String> {
    must_ok(
        ledger.load_worker_activity_tasks(run_id, None),
        "should load worker activity tasks",
    )
    .into_iter()
    .find(|task| task.activity_id.as_str() == activity_id)
    .ok_or_else(|| format!("missing worker task for {activity_id}"))
}
