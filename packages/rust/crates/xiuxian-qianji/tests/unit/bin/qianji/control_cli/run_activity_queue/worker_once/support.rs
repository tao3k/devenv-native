use crate::qianji_cli::tests::control_cli::support::{
    activity_task, append_empty_control_run, must_ok,
};
use xiuxian_qianji_control::{
    ActivityId, ArtifactId, ArtifactKind, ArtifactRef, ControlEvent, ControlEventKind,
    ControlLedger, DuckDbControlLedger, HotStateStore, InMemoryHotStateStore, RunId,
    RunnableActivityTask, WorkerActivityTask,
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

pub(super) fn registry_openai_compatible_llm_task() -> WorkerActivityTask {
    let prompt_ref = registry_artifact_ref("artifact-registry-prompt");
    let mut task = activity_task(
        must_ok(
            ActivityId::new("activity-registry-openai-compatible-llm"),
            "should build registry activity id",
        ),
        "llm.plan",
        "llm.openrouter",
    )
    .with_input_ref(prompt_ref.clone());
    task.metadata = serde_json::json!({
        "qianji_llm_activity_request": {
            "schema": "qianji.llm_activity_request_audit.v1",
            "model": "openrouter/qwen/qwen3-coder",
            "prompt_ref": prompt_ref,
            "context_ref": null,
            "tool_schema_hash": "sha256:tool-schema",
            "temperature_millis": 0,
            "max_tokens": 1024,
            "response_schema_ref": null,
            "budget": null,
            "request_metadata": null,
            "admission_metadata": null
        }
    });
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

pub(super) fn registry_episteme_openai_compatible_llm_task() -> WorkerActivityTask {
    let prompt_ref = registry_artifact_ref("artifact-episteme-reasoning-prompt");
    let context_ref = registry_episteme_context_artifact_ref("artifact-episteme-reasoning-context");
    let mut task = activity_task(
        must_ok(
            ActivityId::new("activity-episteme-reasoning-openai-compatible-llm"),
            "should build Episteme reasoning activity id",
        ),
        "episteme.ontology.reasoning_fill",
        "episteme.ontology.reasoning",
    )
    .with_input_ref(prompt_ref.clone());
    task.metadata = serde_json::json!({
        "qianji_llm_activity_request": {
            "schema": "qianji.llm_activity_request_audit.v1",
            "model": "deepseek/deepseek-v4-pro",
            "prompt_ref": prompt_ref,
            "context_ref": context_ref,
            "temperature_millis": 0,
            "max_tokens": 1024,
            "response_schema_ref": null,
            "request_metadata": {
                "activityType": "episteme.ontology.reasoning_fill",
                "taskQueue": "episteme.ontology.reasoning",
                "reviewOnly": true,
                "rdfMutationAllowed": false
            }
        }
    });
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

fn registry_artifact_ref(artifact_id: &str) -> ArtifactRef {
    ArtifactRef {
        artifact_id: must_ok(ArtifactId::new(artifact_id), "should build artifact id"),
        artifact_kind: must_ok(
            ArtifactKind::new("llm.prompt"),
            "should build artifact kind",
        ),
        uri: format!("artifact://{artifact_id}"),
        content_digest: Some("sha256:prompt".to_string()),
        metadata: serde_json::Value::Null,
    }
}

fn registry_episteme_context_artifact_ref(artifact_id: &str) -> ArtifactRef {
    ArtifactRef {
        artifact_id: must_ok(ArtifactId::new(artifact_id), "should build artifact id"),
        artifact_kind: must_ok(
            ArtifactKind::new("episteme.reasoning_fill_context"),
            "should build artifact kind",
        ),
        uri: format!("artifact://{artifact_id}"),
        content_digest: Some("sha256:context".to_string()),
        metadata: serde_json::Value::Null,
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
    append_control_run_with_activity_route(ledger_path, activity_id, activity_type, task_queue)
}

pub(super) fn append_control_run_with_activity_route(
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
        "should build governed activity id",
    );
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            2,
            ControlEventKind::ActivityScheduled {
                task: activity_task(activity_id, activity_type, task_queue),
            },
        )),
        "should append governed activity route",
    );
    run_id
}

pub(super) fn append_control_run_with_openai_compatible_llm_route(
    ledger_path: &std::path::Path,
) -> RunId {
    let run_id = append_empty_control_run(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let activity_id = must_ok(
        ActivityId::new("activity-openai-compatible-llm"),
        "should build governed LLM activity id",
    );
    let prompt_ref = registry_artifact_ref("artifact-openai-compatible-prompt");
    let mut task =
        activity_task(activity_id, "llm.plan", "llm.openrouter").with_input_ref(prompt_ref.clone());
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
        "should append governed OpenAI-compatible LLM route activity",
    );
    run_id
}

pub(super) fn append_control_run_with_openai_compatible_local_prompt(
    ledger_path: &std::path::Path,
    prompt_path: &std::path::Path,
) -> RunId {
    let run_id = append_empty_control_run(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let activity_id = must_ok(
        ActivityId::new("activity-openai-compatible-llm"),
        "should build governed LLM activity id",
    );
    let prompt_ref = ArtifactRef {
        artifact_id: must_ok(
            ArtifactId::new("artifact-openai-compatible-prompt"),
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
        "should append governed OpenAI-compatible LLM route activity",
    );
    run_id
}

pub(super) fn append_control_run_with_episteme_openai_compatible_local_prompt(
    ledger_path: &std::path::Path,
    prompt_path: &std::path::Path,
    context_path: &std::path::Path,
) -> RunId {
    let run_id = append_empty_control_run(ledger_path);
    let ledger = must_ok(
        DuckDbControlLedger::open(ledger_path),
        "should reopen temporary control ledger",
    );
    let activity_id = must_ok(
        ActivityId::new("activity-episteme-reasoning-openai-compatible-llm"),
        "should build Episteme reasoning activity id",
    );
    let prompt_ref = ArtifactRef {
        artifact_id: must_ok(
            ArtifactId::new("artifact-episteme-reasoning-prompt"),
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
    let context_ref = ArtifactRef {
        artifact_id: must_ok(
            ArtifactId::new("artifact-episteme-reasoning-context"),
            "should build context artifact id",
        ),
        artifact_kind: must_ok(
            ArtifactKind::new("episteme.reasoning_fill_context"),
            "should build context artifact kind",
        ),
        uri: context_path.display().to_string(),
        content_digest: None,
        metadata: serde_json::Value::Null,
    };
    let mut task = activity_task(
        activity_id,
        "episteme.ontology.reasoning_fill",
        "episteme.ontology.reasoning",
    )
    .with_input_ref(prompt_ref.clone());
    task.metadata = serde_json::json!({
        "qianji_llm_activity_request": {
            "schema": "qianji.llm_activity_request_audit.v1",
            "model": "deepseek/deepseek-v4-pro",
            "prompt_ref": prompt_ref,
            "context_ref": context_ref,
            "temperature_millis": 0,
            "max_tokens": 1024,
            "response_schema_ref": null,
            "request_metadata": {
                "activityType": "episteme.ontology.reasoning_fill",
                "taskQueue": "episteme.ontology.reasoning",
                "reviewOnly": true,
                "rdfMutationAllowed": false
            }
        }
    });
    must_ok(
        ledger.append_event(ControlEvent::run(
            run_id.clone(),
            2,
            ControlEventKind::ActivityScheduled { task },
        )),
        "should append governed Episteme OpenAI-compatible LLM route activity",
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
