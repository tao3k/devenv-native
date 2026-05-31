use std::io;
use std::path::{Path, PathBuf};

use xiuxian_qianji_control::{
    ActivityJournalWriteOutcome, ActivityResult, ArtifactRef, ControlEventRecord, ControlLedger,
    ErrorCode, HotStateLeasedActivityTask, HotStateStore, RunId, RunScopedActivityTaskClaimRequest,
    TaskQueue, WorkerActivityTask, WorkerId, WorkerRef,
};

use super::{
    ActivityExecutorOutcome, OpenAiCompatibleLlmExecutionRequest, control_error,
    execute_openai_compatible_llm, invalid_input,
};

const OPENAI_COMPATIBLE_LLM_ACTIVITY_TYPES: &[&str] = &[
    "llm.plan",
    "llm.tool_select",
    "llm.repair",
    "episteme.ontology.reasoning_fill",
];
const OPENAI_COMPATIBLE_LLM_TASK_QUEUES: &[&str] = &[
    "llm.*",
    "llm.openai",
    "llm.openrouter",
    "llm.local",
    "episteme.ontology.reasoning",
];
const OPENAI_COMPATIBLE_LLM_TASK_QUEUE_PREFIX: &str = "llm.";
const LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY: &str = "qianji_llm_activity_request";
const EPISTEME_REASONING_ACTIVITY_TYPE: &str = "episteme.ontology.reasoning_fill";

#[derive(Clone, Copy)]
pub(crate) struct OpenAiCompatibleWorkerOnceRequest<'a> {
    pub(crate) worker_id: &'a str,
    pub(crate) task_queue: Option<&'a str>,
    pub(crate) now_ms: u64,
    pub(crate) lease_ttl_ms: u64,
    pub(crate) heartbeat_ttl_ms: Option<u64>,
    pub(crate) settled_at_ms: u64,
    pub(crate) output_artifact_dir: &'a Path,
    pub(crate) output_artifact_id: Option<&'a str>,
    pub(crate) output_artifact_kind: Option<&'a str>,
    pub(crate) openai_compatible_base_url: &'a str,
    pub(crate) openai_compatible_api_key: Option<&'a str>,
    pub(crate) openai_compatible_timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub(crate) struct OpenAiCompatibleWorkerOnceOutput {
    pub(crate) worker_id: WorkerId,
    pub(crate) task_queue: Option<TaskQueue>,
    pub(crate) claimed: Option<HotStateLeasedActivityTask>,
    pub(crate) start: Option<ActivityJournalWriteOutcome>,
    pub(crate) heartbeat: Option<ControlEventRecord>,
    pub(crate) terminal: Option<ActivityJournalWriteOutcome>,
    pub(crate) released: bool,
}

pub(crate) async fn run_openai_compatible_worker_once_for_run<L, H>(
    ledger: &L,
    hot_state: &H,
    run_id: &RunId,
    request: &OpenAiCompatibleWorkerOnceRequest<'_>,
) -> io::Result<OpenAiCompatibleWorkerOnceOutput>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
{
    let worker_id = WorkerId::new(request.worker_id).map_err(|error| control_error(&error))?;
    let task_queue = request
        .task_queue
        .map(TaskQueue::new)
        .transpose()
        .map_err(|error| control_error(&error))?;
    let worker = WorkerRef {
        worker_id: worker_id.clone(),
        capabilities: Vec::new(),
        metadata: serde_json::Value::Null,
    };
    let claimed = hot_state
        .claim_activity_task_for_run(run_scoped_claim_request(
            worker,
            run_id,
            task_queue.as_ref(),
            request.now_ms,
            request.lease_ttl_ms,
        ))
        .await
        .map_err(|error| control_error(&error))?;
    let Some(claimed_task) = claimed.clone() else {
        return Ok(empty_output(worker_id, task_queue));
    };
    execute_claimed_openai_compatible_worker_once(
        ledger,
        hot_state,
        request,
        worker_id,
        task_queue,
        claimed_task,
    )
    .await
}

fn run_scoped_claim_request(
    worker: WorkerRef,
    run_id: &RunId,
    task_queue: Option<&TaskQueue>,
    now_ms: u64,
    lease_ttl_ms: u64,
) -> RunScopedActivityTaskClaimRequest {
    let request =
        RunScopedActivityTaskClaimRequest::new(worker, run_id.clone(), now_ms, lease_ttl_ms);
    if let Some(task_queue) = task_queue {
        request.with_task_queue(task_queue.clone())
    } else {
        request
    }
}

fn empty_output(
    worker_id: WorkerId,
    task_queue: Option<TaskQueue>,
) -> OpenAiCompatibleWorkerOnceOutput {
    OpenAiCompatibleWorkerOnceOutput {
        worker_id,
        task_queue,
        claimed: None,
        start: None,
        heartbeat: None,
        terminal: None,
        released: false,
    }
}

async fn execute_claimed_openai_compatible_worker_once<L, H>(
    ledger: &L,
    hot_state: &H,
    request: &OpenAiCompatibleWorkerOnceRequest<'_>,
    worker_id: WorkerId,
    task_queue: Option<TaskQueue>,
    claimed_task: HotStateLeasedActivityTask,
) -> io::Result<OpenAiCompatibleWorkerOnceOutput>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
{
    validate_openai_compatible_task(&claimed_task.activity_task.task)?;
    validate_request(request)?;
    let start = xiuxian_qianji_control::record_worker_activity_started_idempotent(
        ledger,
        xiuxian_qianji_control::WorkerActivityStartRecord::new(
            claimed_task.activity_task.task.clone(),
            worker_id.clone(),
            request.now_ms,
        ),
    )
    .map_err(|error| control_error(&error))?;
    let heartbeat = record_worker_heartbeat_if_enabled(
        ledger,
        hot_state,
        &claimed_task,
        &worker_id,
        request.now_ms,
        request.heartbeat_ttl_ms,
    )
    .await?;
    let executor_outcome =
        execute_openai_compatible_worker_outcome(&claimed_task.activity_task.task, request).await?;
    let terminal = record_terminal(
        ledger,
        &claimed_task,
        request.settled_at_ms,
        executor_outcome,
    )?;
    let released = hot_state
        .release_activity_task_lease(&claimed_task.lease)
        .await
        .map_err(|error| control_error(&error))?;
    Ok(OpenAiCompatibleWorkerOnceOutput {
        worker_id,
        task_queue,
        claimed: Some(claimed_task),
        start: Some(start),
        heartbeat,
        terminal: Some(terminal),
        released,
    })
}

async fn execute_openai_compatible_worker_outcome(
    task: &WorkerActivityTask,
    request: &OpenAiCompatibleWorkerOnceRequest<'_>,
) -> io::Result<ActivityExecutorOutcome> {
    let output_artifact_path = output_artifact_path(task, request);
    let execution_request = OpenAiCompatibleLlmExecutionRequest {
        task,
        base_url: request.openai_compatible_base_url,
        api_key: request.openai_compatible_api_key,
        timeout_ms: request.openai_compatible_timeout_ms,
        output_artifact_path: output_artifact_path.as_path(),
        output_artifact_id: request.output_artifact_id,
        output_artifact_kind: request.output_artifact_kind,
    };
    execute_openai_compatible_llm(&execution_request).await
}

fn validate_openai_compatible_task(task: &WorkerActivityTask) -> io::Result<()> {
    if task.next_attempt == 0 {
        return Err(invalid_input(
            "activity executor worker task must have a positive next_attempt",
        ));
    }
    validate_allowed_route(
        "activity_type",
        task.activity_type.as_str(),
        OPENAI_COMPATIBLE_LLM_ACTIVITY_TYPES,
    )?;
    validate_allowed_route(
        "task_queue",
        task.task_queue.as_str(),
        OPENAI_COMPATIBLE_LLM_TASK_QUEUES,
    )?;
    if task.input_ref.is_none() {
        return Err(invalid_input(
            "activity executor `openai-compatible-llm` requires task input_ref",
        ));
    }
    validate_llm_request_audit(task)
}

fn validate_allowed_route(field: &'static str, value: &str, allowed: &[&str]) -> io::Result<()> {
    if field == "task_queue" && value.starts_with(OPENAI_COMPATIBLE_LLM_TASK_QUEUE_PREFIX) {
        return Ok(());
    }
    if allowed.contains(&value) {
        return Ok(());
    }
    Err(invalid_input(format!(
        "activity executor `OpenAiCompatibleLlm` does not allow {field} `{value}`"
    )))
}

fn validate_llm_request_audit(task: &WorkerActivityTask) -> io::Result<()> {
    let audit = task
        .metadata
        .get(LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY)
        .ok_or_else(|| {
            invalid_input(format!(
                "activity executor `openai-compatible-llm` requires `{LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY}` metadata"
            ))
        })?;
    let prompt_ref = audit.get("prompt_ref").ok_or_else(|| {
        invalid_input("activity executor `openai-compatible-llm` requires request audit prompt_ref")
    })?;
    let prompt_ref: ArtifactRef = serde_json::from_value(prompt_ref.clone()).map_err(|error| {
        invalid_input(format!(
            "activity executor `openai-compatible-llm` has invalid request audit prompt_ref: {error}"
        ))
    })?;
    if task.input_ref.as_ref() != Some(&prompt_ref) {
        return Err(invalid_input(
            "activity executor `openai-compatible-llm` requires task input_ref to match request audit prompt_ref",
        ));
    }
    let model = audit
        .get("model")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .trim();
    if model.is_empty() {
        return Err(invalid_input(
            "activity executor `openai-compatible-llm` requires request audit model",
        ));
    }
    if task.activity_type.as_str() == EPISTEME_REASONING_ACTIVITY_TYPE
        && audit
            .get("context_ref")
            .is_none_or(serde_json::Value::is_null)
    {
        return Err(invalid_input(
            "activity executor `openai-compatible-llm` requires Episteme reasoning context_ref",
        ));
    }
    Ok(())
}

fn validate_request(request: &OpenAiCompatibleWorkerOnceRequest<'_>) -> io::Result<()> {
    if request.openai_compatible_base_url.trim().is_empty() {
        return Err(invalid_input(
            "missing OpenAI-compatible base URL for qianji worker execution",
        ));
    }
    if request.output_artifact_dir.as_os_str().is_empty() {
        return Err(invalid_input(
            "missing output artifact directory for OpenAI-compatible worker execution",
        ));
    }
    Ok(())
}

fn output_artifact_path(
    task: &WorkerActivityTask,
    request: &OpenAiCompatibleWorkerOnceRequest<'_>,
) -> PathBuf {
    request.output_artifact_dir.join(format!(
        "{}-attempt-{}.openai-compatible-llm.json",
        activity_artifact_stem(task.activity_id.as_str()),
        task.next_attempt
    ))
}

fn activity_artifact_stem(activity_id: &str) -> String {
    let stem: String = activity_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if stem.is_empty() {
        "activity".to_string()
    } else {
        stem
    }
}

fn record_terminal<L>(
    ledger: &L,
    claimed_task: &HotStateLeasedActivityTask,
    settled_at_ms: u64,
    outcome: ActivityExecutorOutcome,
) -> io::Result<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    match outcome {
        ActivityExecutorOutcome::Complete { result } => {
            record_complete(ledger, claimed_task, settled_at_ms, result)
        }
        ActivityExecutorOutcome::Fail {
            error_code,
            message,
            retryable,
            metadata,
        } => record_fail(
            ledger,
            claimed_task,
            settled_at_ms,
            error_code,
            &message,
            retryable,
            metadata,
        ),
    }
}

fn record_complete<L>(
    ledger: &L,
    claimed_task: &HotStateLeasedActivityTask,
    settled_at_ms: u64,
    result: ActivityResult,
) -> io::Result<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    let complete_record = xiuxian_qianji_control::WorkerActivityCompletedRecord::new(
        claimed_task.activity_task.task.clone(),
        settled_at_ms,
        result,
    );
    xiuxian_qianji_control::record_worker_activity_completed_idempotent(ledger, complete_record)
        .map_err(|error| control_error(&error))
}

fn record_fail<L>(
    ledger: &L,
    claimed_task: &HotStateLeasedActivityTask,
    settled_at_ms: u64,
    error_code: ErrorCode,
    message: &str,
    retryable: bool,
    metadata: serde_json::Value,
) -> io::Result<ActivityJournalWriteOutcome>
where
    L: ControlLedger + ?Sized,
{
    let failure_record = xiuxian_qianji_control::WorkerActivityFailedRecord::new(
        xiuxian_qianji_control::WorkerActivityFailureInput::new(
            claimed_task.activity_task.task.clone(),
            error_code,
            message,
        )
        .with_failed_at_ms(settled_at_ms)
        .with_retryable(retryable),
    )
    .with_metadata(metadata);
    xiuxian_qianji_control::record_worker_activity_failed_idempotent(ledger, failure_record)
        .map_err(|error| control_error(&error))
}

async fn record_worker_heartbeat_if_enabled<L, H>(
    ledger: &L,
    hot_state: &H,
    claimed_task: &HotStateLeasedActivityTask,
    worker_id: &WorkerId,
    observed_at_ms: u64,
    heartbeat_ttl_ms: Option<u64>,
) -> io::Result<Option<ControlEventRecord>>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
{
    let Some(heartbeat_ttl_ms) = heartbeat_ttl_ms else {
        return Ok(None);
    };
    let expires_at_ms = observed_at_ms
        .checked_add(heartbeat_ttl_ms)
        .ok_or_else(|| invalid_input("qianji worker heartbeat TTL overflowed u64"))?;
    let heartbeat = xiuxian_qianji_control::WorkerHeartbeat {
        worker_id: worker_id.clone(),
        observed_at_ms,
        expires_at_ms,
        metadata: heartbeat_metadata(claimed_task),
    };
    xiuxian_qianji_control::record_worker_heartbeat_with_hot_state(
        ledger,
        hot_state,
        xiuxian_qianji_control::WorkerHeartbeatJournalRecord::new(
            claimed_task.activity_task.task.run_id.clone(),
            heartbeat,
        ),
    )
    .await
    .map(Some)
    .map_err(|error| control_error(&error))
}

fn heartbeat_metadata(claimed_task: &HotStateLeasedActivityTask) -> serde_json::Value {
    serde_json::json!({
        "source": "qianji-activity-worker",
        "executor": "openai-compatible-llm",
        "phase": "provider_request_active",
        "activity_id": claimed_task.activity_task.task.activity_id.as_str(),
        "activity_type": claimed_task.activity_task.task.activity_type.as_str(),
        "task_queue": claimed_task.activity_task.task.task_queue.as_str(),
        "next_attempt": claimed_task.activity_task.task.next_attempt,
    })
}
