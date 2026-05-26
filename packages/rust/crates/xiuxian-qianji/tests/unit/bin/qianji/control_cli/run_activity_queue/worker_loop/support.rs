use std::path::Path;

use crate::qianji_cli::tests::control_cli::support::{activity_task, must_ok};
use xiuxian_qianji_control::{
    ActivityId, ActivityRetryPolicy, ArtifactId, ArtifactKind, ArtifactRef, ControlEvent,
    ControlEventKind, ControlLedger, HotStateStore, InMemoryHotStateStore, RecoveryAttempt,
    RecoveryPolicy, RunId, RunnableActivityTask, StepId,
};

pub(super) async fn enqueue_worker_task(
    ledger: &impl ControlLedger,
    hot_state: &InMemoryHotStateStore,
    run_id: &RunId,
    activity_id: &str,
) -> Result<(), String> {
    let task = worker_task(ledger, run_id, activity_id)?;
    must_ok(
        hot_state
            .enqueue_activity_task(RunnableActivityTask {
                task,
                priority: 10,
                not_before_ms: 7_000,
                metadata: serde_json::json!({"mirror": "worker-loop"}),
            })
            .await,
        "should enqueue activity task",
    );
    Ok(())
}

fn worker_task(
    ledger: &impl ControlLedger,
    run_id: &RunId,
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

pub(super) fn append_control_run_with_openai_compatible_local_prompt(
    ledger: &impl ControlLedger,
    prompt_path: &Path,
    activity_id: &str,
) -> RunId {
    let run_id = append_empty_control_run_to_ledger(ledger);
    let activity_id = must_ok(
        ActivityId::new(activity_id),
        "should build governed LLM activity id",
    );
    let prompt_ref = ArtifactRef {
        artifact_id: must_ok(
            ArtifactId::new("artifact-openai-compatible-loop-prompt"),
            "should build prompt artifact id",
        ),
        artifact_kind: must_ok(
            ArtifactKind::new("llm.prompt"),
            "should build prompt artifact kind",
        ),
        uri: prompt_path.display().to_string(),
        content_digest: None,
        metadata: serde_json::Value::Null,
    };
    let mut task =
        activity_task(activity_id, "llm.plan", "llm.openrouter").with_input_ref(prompt_ref.clone());
    task.retry_policy = Some(must_ok(
        ActivityRetryPolicy::new(3).map(|policy| policy.with_initial_interval_ms(25)),
        "should build governed LLM retry policy",
    ));
    task.metadata = serde_json::json!({
        "qianji_llm_activity_request": {
            "schema": "qianji.llm_activity_request_audit.v1",
            "model": "openrouter/qwen/qwen3-coder",
            "prompt_ref": prompt_ref,
            "context_ref": null,
            "tool_schema_hash": null,
            "temperature_millis": 0,
            "max_tokens": 1024,
            "response_schema_ref": null,
            "budget": null,
            "request_metadata": null,
            "admission_metadata": null
        }
    });
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            2,
            ControlEventKind::ActivityScheduled { task },
        )),
        "should append governed OpenAI-compatible loop LLM route activity",
    );
    run_id
}

pub(super) fn recovery_attempt() -> RecoveryAttempt {
    RecoveryAttempt {
        attempt: 1,
        reason: "recover OpenAI-compatible provider failure".to_string(),
        policy: RecoveryPolicy {
            max_attempts: 3,
            backoff_ms: 25,
            require_human_approval: false,
        },
    }
}

pub(super) fn append_control_run_with_scheduled_activity_queue(
    ledger: &impl ControlLedger,
) -> RunId {
    let run_id = append_control_run_with_step(ledger);
    let step_id = must_ok(
        StepId::new("run-control-step"),
        "should build control step id",
    );
    let scheduled_run_activity = must_ok(
        ActivityId::new("activity-run-scheduled"),
        "should build scheduled run activity id",
    );
    let started_activity = must_ok(
        ActivityId::new("activity-run-started"),
        "should build started run activity id",
    );
    let scheduled_step_activity = must_ok(
        ActivityId::new("activity-step-scheduled"),
        "should build scheduled step activity id",
    );

    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            3,
            ControlEventKind::ActivityScheduled {
                task: activity_task(scheduled_run_activity, "llm.plan", "llm.openai"),
            },
        )),
        "should append scheduled run activity",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            4,
            ControlEventKind::ActivityScheduled {
                task: activity_task(started_activity.clone(), "llm.plan", "llm.openai"),
            },
        )),
        "should append started run activity schedule",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            5,
            ControlEventKind::ActivityStarted {
                activity_id: started_activity,
                worker_id: None,
                attempt: 1,
            },
        )),
        "should append started run activity",
    );
    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id,
            6,
            ControlEventKind::ActivityScheduled {
                task: activity_task(scheduled_step_activity, "tool.github", "tool.github"),
            },
        )),
        "should append scheduled step activity",
    );
    run_id
}

pub(super) fn append_scheduled_run_activity(
    ledger: &impl ControlLedger,
    run_id: &RunId,
    sequence: u64,
    activity_id: &str,
    task_queue: &str,
) {
    let activity_id = must_ok(
        ActivityId::new(activity_id),
        "should build additional scheduled run activity id",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            sequence,
            ControlEventKind::ActivityScheduled {
                task: activity_task(activity_id, "llm.plan", task_queue),
            },
        )),
        "should append additional scheduled run activity",
    );
}

pub(super) fn assert_worker_iteration_workers(
    json: &serde_json::Value,
    expected_worker_ids: &[&str],
) {
    for (index, worker_id) in expected_worker_ids.iter().enumerate() {
        assert_eq!(json["iterations"][index]["output"]["worker_id"], *worker_id);
    }
}

fn append_control_run_with_step(ledger: &impl ControlLedger) -> RunId {
    let run_id = append_empty_control_run_to_ledger(ledger);
    let step_id = must_ok(
        StepId::new("run-control-step"),
        "should build control step id",
    );
    must_ok(
        ledger.append_event(ControlEvent::step(
            run_id.clone(),
            step_id,
            2,
            ControlEventKind::StepCreated {
                title: "Review durable state".to_string(),
                required_evidence: vec!["history_visible".to_string()],
                budget: None,
            },
        )),
        "should append step-created event",
    );
    run_id
}

fn append_empty_control_run_to_ledger(ledger: &impl ControlLedger) -> RunId {
    let run_id = must_ok(RunId::new("run-control-cli"), "should build control run id");
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            1,
            ControlEventKind::RunCreated {
                intent: "test qianji control recovery snapshot".to_string(),
                budget: None,
                metadata: serde_json::Value::Null,
            },
        )),
        "should append run-created event",
    );
    run_id
}
