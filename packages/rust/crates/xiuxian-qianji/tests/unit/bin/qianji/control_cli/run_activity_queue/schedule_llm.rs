use crate::qianji_cli::test_exports::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ActivityWorkerOnceStoreRequest,
    ControlCliCommand, WorkerActivityMirrorStoreRequest, mirror_with_hot_state,
    run_control_command, worker_once_with_hot_state,
};
use crate::qianji_cli::tests::control_cli::support::{append_control_run_with_step, must_ok};
use tempfile::TempDir;
use xiuxian_qianji_control::{
    ActivityId, ActivityTask, ActivityType, ArtifactId, ArtifactKind, ArtifactRef, ControlLedger,
    DuckDbControlLedger, HotStateStore, IdempotencyKey, InMemoryHotStateStore, LlmActivityRequest,
    LlmActivityTask, LlmModelId, TaskQueue,
};

#[test]
fn run_control_activity_schedule_llm_appends_json_and_updates_queue() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);
    let llm_activity_json = llm_activity_json("activity-cli-llm")?;

    let output = must_ok(
        run_control_command(&ControlCliCommand::ActivityScheduleLlm {
            ledger_path: ledger_path.clone(),
            run_id: run_id.as_str().to_string(),
            step_id: Some("run-control-step".to_string()),
            occurred_at_ms: 3_000,
            llm_activity_json,
            json: true,
        }),
        "control activity-schedule-llm json should render",
    );
    let json: serde_json::Value = must_ok(
        serde_json::from_str(&output.rendered),
        "activity-schedule-llm output should be valid json",
    );
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let queue = must_ok(
        ledger.load_activity_queue_projection(&run_id, None),
        "activity-schedule-llm should replay into queue projection",
    );

    assert_eq!(json["status"], "appended");
    assert_eq!(
        json["record"]["event"]["kind"]["event"],
        "activity_scheduled"
    );
    assert_eq!(
        json["record"]["event"]["kind"]["task"]["activity_id"],
        "activity-cli-llm"
    );
    assert_eq!(queue.summary.scheduled, 1);
    assert_eq!(
        queue.items[0].activity.activity_id.as_str(),
        "activity-cli-llm"
    );
    Ok(())
}

#[test]
fn run_control_activity_schedule_llm_is_idempotent_for_exact_duplicate() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);
    let command = ControlCliCommand::ActivityScheduleLlm {
        ledger_path,
        run_id: run_id.as_str().to_string(),
        step_id: Some("run-control-step".to_string()),
        occurred_at_ms: 3_000,
        llm_activity_json: llm_activity_json("activity-cli-llm")?,
        json: true,
    };

    let first = must_ok(
        run_control_command(&command),
        "first activity-schedule-llm should append",
    );
    let second = must_ok(
        run_control_command(&command),
        "duplicate activity-schedule-llm should be idempotent",
    );
    let first_json: serde_json::Value = must_ok(
        serde_json::from_str(&first.rendered),
        "first output should be valid json",
    );
    let second_json: serde_json::Value = must_ok(
        serde_json::from_str(&second.rendered),
        "second output should be valid json",
    );

    assert_eq!(first_json["status"], "appended");
    assert_eq!(second_json["status"], "already_recorded");
    assert_eq!(
        first_json["record"]["sequence"],
        second_json["record"]["sequence"]
    );
    Ok(())
}

#[test]
fn run_control_activity_schedule_llm_rejects_invalid_claim_check_binding() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);

    let Err(error) = run_control_command(&ControlCliCommand::ActivityScheduleLlm {
        ledger_path,
        run_id: run_id.as_str().to_string(),
        step_id: Some("run-control-step".to_string()),
        occurred_at_ms: 3_000,
        llm_activity_json: mismatched_llm_activity_json()?,
        json: false,
    }) else {
        return Err("invalid claim-check LLM activity schedule should fail".to_string());
    };

    assert!(
        error.to_string().contains("prompt_ref"),
        "unexpected error: {error}"
    );
    Ok(())
}

#[tokio::test]
async fn scheduled_llm_activity_flows_through_mirror_and_worker_once() -> Result<(), String> {
    let temp_dir =
        TempDir::new().map_err(|error| format!("should create temporary directory: {error}"))?;
    let ledger_path = temp_dir.path().join("control.duckdb");
    let run_id = append_control_run_with_step(&ledger_path);
    append_lifecycle_llm_schedule(&ledger_path, run_id.as_str())?;
    let ledger = must_ok(
        DuckDbControlLedger::open(&ledger_path),
        "should reopen temporary control ledger",
    );
    let hot_state = InMemoryHotStateStore::new();

    let before_mirror_count = assert_lifecycle_llm_is_queued(&ledger, &run_id)?;
    mirror_lifecycle_llm_activity(&ledger, &hot_state, run_id.as_str()).await?;
    assert_mirror_does_not_append_events(&ledger, &run_id, before_mirror_count);
    let worker_json = complete_lifecycle_llm_activity(&ledger, &hot_state).await?;
    assert_lifecycle_llm_completed(&ledger, &hot_state, &run_id, &worker_json).await?;
    Ok(())
}

fn append_lifecycle_llm_schedule(
    ledger_path: &std::path::Path,
    run_id: &str,
) -> Result<(), String> {
    must_ok(
        run_control_command(&ControlCliCommand::ActivityScheduleLlm {
            ledger_path: ledger_path.to_path_buf(),
            run_id: run_id.to_string(),
            step_id: Some("run-control-step".to_string()),
            occurred_at_ms: 3_000,
            llm_activity_json: llm_activity_json_with_route(
                "activity-cli-llm-lifecycle",
                "llm.repair",
                "llm.openrouter",
            )?,
            json: true,
        }),
        "control activity-schedule-llm should append lifecycle proof activity",
    );
    Ok(())
}

fn assert_lifecycle_llm_is_queued(
    ledger: &DuckDbControlLedger,
    run_id: &xiuxian_qianji_control::RunId,
) -> Result<usize, String> {
    let before_mirror_count = must_ok(
        ledger.load_events(run_id),
        "should load events before mirror",
    )
    .len();
    let queue = must_ok(
        ledger.load_activity_queue_projection(run_id, Some(&task_queue("llm.openrouter")?)),
        "scheduled LLM activity should replay into the OpenRouter queue",
    );

    assert_eq!(queue.summary.scheduled, 1);
    assert_eq!(
        queue.items[0].activity.activity_id.as_str(),
        "activity-cli-llm-lifecycle"
    );
    Ok(before_mirror_count)
}

async fn mirror_lifecycle_llm_activity(
    ledger: &DuckDbControlLedger,
    hot_state: &InMemoryHotStateStore,
    run_id: &str,
) -> Result<(), String> {
    must_ok(
        mirror_with_hot_state(
            ledger,
            hot_state,
            WorkerActivityMirrorStoreRequest {
                run_id,
                task_queue: Some("llm.openrouter"),
                priority: 11,
                not_before_ms: 4_000,
                metadata: Some(r#"{"proof":"llm-lifecycle"}"#),
                json: true,
            },
        )
        .await,
        "mirror should enqueue the scheduled LLM activity into hot state",
    );
    Ok(())
}

fn assert_mirror_does_not_append_events(
    ledger: &DuckDbControlLedger,
    run_id: &xiuxian_qianji_control::RunId,
    before_mirror_count: usize,
) {
    let after_mirror_count = must_ok(
        ledger.load_events(run_id),
        "should load events after mirror",
    )
    .len();

    assert_eq!(before_mirror_count, after_mirror_count);
}

async fn complete_lifecycle_llm_activity(
    ledger: &DuckDbControlLedger,
    hot_state: &InMemoryHotStateStore,
) -> Result<serde_json::Value, String> {
    let worker_output = must_ok(
        worker_once_with_hot_state(
            ledger,
            hot_state,
            ActivityWorkerOnceStoreRequest {
                worker_id: "worker-llm-lifecycle",
                task_queue: Some("llm.openrouter"),
                now_ms: 4_000,
                lease_ttl_ms: 500,
                heartbeat_ttl_ms: None,
                executor: ActivityExecutorKindArg::Fixture,
                outcome: ActivitySettleOutcomeArg::Complete,
                settled_at_ms: 4_500,
                output_hash: Some("sha256:llm-lifecycle-output"),
                error_code: None,
                message: None,
                retryable: None,
                metadata: Some(r#"{"proof":"terminal"}"#),
                json: true,
            },
        )
        .await,
        "worker-once should complete the mirrored LLM activity through fixture",
    );
    let worker_json: serde_json::Value = must_ok(
        serde_json::from_str(&worker_output.rendered),
        "worker-once lifecycle proof output should be valid json",
    );
    Ok(worker_json)
}

async fn assert_lifecycle_llm_completed(
    ledger: &DuckDbControlLedger,
    hot_state: &InMemoryHotStateStore,
    run_id: &xiuxian_qianji_control::RunId,
    worker_json: &serde_json::Value,
) -> Result<(), String> {
    let replayed = must_ok(
        ledger.load_activity_queue_projection(run_id, Some(&task_queue("llm.openrouter")?)),
        "worker-once lifecycle proof should replay into completed queue state",
    );
    let snapshot = must_ok(
        hot_state.load_snapshot(4_600).await,
        "hot state snapshot should load after worker-once lifecycle proof",
    );

    assert_eq!(worker_json["outcome"], "complete");
    assert_eq!(
        worker_json["claimed"]["activity_task"]["task"]["activity_type"],
        "llm.repair"
    );
    assert_eq!(
        worker_json["terminal"]["record"]["event"]["kind"]["result"]["output_hash"],
        "sha256:llm-lifecycle-output"
    );
    assert_eq!(worker_json["released"], true);
    assert_eq!(replayed.summary.completed, 1);
    assert_eq!(snapshot.active_activity_task_lease_count(), 0);
    Ok(())
}

fn llm_activity_json(activity_id: &str) -> Result<String, String> {
    llm_activity_json_with_route(activity_id, "llm.plan", "llm.openai")
}

fn llm_activity_json_with_route(
    activity_id: &str,
    activity_type: &str,
    task_queue: &str,
) -> Result<String, String> {
    let prompt_ref = artifact_ref("artifact-cli-llm-prompt")?;
    let task = ActivityTask::new(
        ActivityId::new(activity_id).map_err(|error| format!("activity id: {error}"))?,
        ActivityType::new(activity_type).map_err(|error| format!("activity type: {error}"))?,
        TaskQueue::new(task_queue).map_err(|error| format!("task queue: {error}"))?,
        IdempotencyKey::new(format!("run/{activity_id}/llm"))
            .map_err(|error| format!("idempotency key: {error}"))?,
    )
    .with_input_ref(prompt_ref.clone())
    .with_timeout_ms(30_000);
    let request = LlmActivityRequest::new(
        LlmModelId::new("openai/gpt-5.2").map_err(|error| format!("model id: {error}"))?,
        prompt_ref,
    )
    .with_max_tokens(1_024);
    serde_json::to_string(&LlmActivityTask::new(task, request))
        .map_err(|error| format!("should serialize LLM activity task: {error}"))
}

fn task_queue(value: &str) -> Result<TaskQueue, String> {
    TaskQueue::new(value).map_err(|error| format!("task queue: {error}"))
}

fn mismatched_llm_activity_json() -> Result<String, String> {
    let task_ref = artifact_ref("artifact-cli-llm-other")?;
    let prompt_ref = artifact_ref("artifact-cli-llm-prompt")?;
    let task = ActivityTask::new(
        ActivityId::new("activity-cli-llm-invalid")
            .map_err(|error| format!("activity id: {error}"))?,
        ActivityType::new("llm.plan").map_err(|error| format!("activity type: {error}"))?,
        TaskQueue::new("llm.openai").map_err(|error| format!("task queue: {error}"))?,
        IdempotencyKey::new("run/activity-cli-llm-invalid/llm")
            .map_err(|error| format!("idempotency key: {error}"))?,
    )
    .with_input_ref(task_ref);
    let request = LlmActivityRequest::new(
        LlmModelId::new("openai/gpt-5.2").map_err(|error| format!("model id: {error}"))?,
        prompt_ref,
    );
    serde_json::to_string(&LlmActivityTask::new(task, request))
        .map_err(|error| format!("should serialize invalid LLM activity task: {error}"))
}

fn artifact_ref(artifact_id: &str) -> Result<ArtifactRef, String> {
    Ok(ArtifactRef {
        artifact_id: ArtifactId::new(artifact_id)
            .map_err(|error| format!("artifact id: {error}"))?,
        artifact_kind: ArtifactKind::new("claim_check")
            .map_err(|error| format!("artifact kind: {error}"))?,
        uri: format!("artifact://{artifact_id}"),
        content_digest: Some(format!("sha256:{artifact_id}")),
        metadata: serde_json::Value::Null,
    })
}
