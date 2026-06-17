use std::io;
use std::path::Path;

use crate::qianji_cli::invalid_input;

use super::activity_args::ActivitySettleOutcomeArg;
#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
use super::activity_artifact::{
    ActivityOutputArtifact, ActivityOutputArtifactRequest, write_activity_output_artifact,
};
use super::activity_executor::ActivityExecutorKindArg;
#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
use super::activity_executor::{
    ActivityExecutionRequest, ActivityExecutorOutcome, ActivityExecutorRegistry,
};
use super::types::ControlCliOutput;

pub(super) use super::activity_worker_once_args::parse;

#[derive(Clone, Copy)]
pub(super) struct ActivityWorkerOnceRunRequest<'a> {
    pub(super) ledger_path: &'a Path,
    pub(super) valkey_url: &'a str,
    pub(super) namespace: Option<&'a str>,
    pub(super) worker_id: &'a str,
    pub(super) task_queue: Option<&'a str>,
    pub(super) now_ms: u64,
    pub(super) lease_ttl_ms: u64,
    pub(super) executor: ActivityExecutorKindArg,
    pub(super) outcome: ActivitySettleOutcomeArg,
    pub(super) settled_at_ms: u64,
    pub(super) output_ref_json: Option<&'a str>,
    pub(super) output_hash: Option<&'a str>,
    pub(super) output_artifact_path: Option<&'a Path>,
    pub(super) output_artifact_content: Option<&'a str>,
    pub(super) output_artifact_id: Option<&'a str>,
    pub(super) output_artifact_kind: Option<&'a str>,
    pub(super) openai_compatible_base_url: Option<&'a str>,
    pub(super) openai_compatible_api_key: Option<&'a str>,
    pub(super) openai_compatible_timeout_ms: Option<u64>,
    pub(super) error_code: Option<&'a str>,
    pub(super) message: Option<&'a str>,
    pub(super) retryable: Option<bool>,
    pub(super) metadata: Option<&'a str>,
    pub(super) json: bool,
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
#[derive(Clone, Copy)]
pub(crate) struct ActivityWorkerOnceStoreRequest<'a> {
    pub(crate) worker_id: &'a str,
    pub(crate) task_queue: Option<&'a str>,
    pub(crate) now_ms: u64,
    pub(crate) lease_ttl_ms: u64,
    pub(crate) heartbeat_ttl_ms: Option<u64>,
    pub(crate) executor: ActivityExecutorKindArg,
    pub(crate) outcome: ActivitySettleOutcomeArg,
    pub(crate) settled_at_ms: u64,
    pub(crate) output_ref_json: Option<&'a str>,
    pub(crate) output_hash: Option<&'a str>,
    pub(crate) output_artifact_path: Option<&'a Path>,
    pub(crate) output_artifact_dir: Option<&'a Path>,
    pub(crate) output_artifact_content: Option<&'a str>,
    pub(crate) output_artifact_id: Option<&'a str>,
    pub(crate) output_artifact_kind: Option<&'a str>,
    pub(crate) openai_compatible_base_url: Option<&'a str>,
    pub(crate) openai_compatible_api_key: Option<&'a str>,
    pub(crate) openai_compatible_timeout_ms: Option<u64>,
    pub(crate) error_code: Option<&'a str>,
    pub(crate) message: Option<&'a str>,
    pub(crate) retryable: Option<bool>,
    pub(crate) metadata: Option<&'a str>,
    pub(crate) json: bool,
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub(crate) struct ActivityWorkerOnceOutput {
    pub(crate) worker_id: xiuxian_qianji_control::WorkerId,
    pub(crate) task_queue: Option<xiuxian_qianji_control::TaskQueue>,
    pub(crate) executor: ActivityExecutorKindArg,
    pub(crate) executor_contract: Option<super::activity_executor::ActivityExecutorContract>,
    pub(crate) outcome: ActivitySettleOutcomeArg,
    pub(crate) claimed: Option<xiuxian_qianji_control::HotStateLeasedActivityTask>,
    pub(crate) start: Option<xiuxian_qianji_control::ActivityJournalWriteOutcome>,
    pub(crate) heartbeat: Option<xiuxian_qianji_control::ControlEventRecord>,
    pub(crate) terminal: Option<xiuxian_qianji_control::ActivityJournalWriteOutcome>,
    pub(crate) released: bool,
}

#[cfg(all(feature = "duckdb", feature = "valkey"))]
pub(super) fn run(request: &ActivityWorkerOnceRunRequest<'_>) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{DuckDbControlLedger, ValkeyHotStateConfig, ValkeyHotStateStore};

    let ledger =
        DuckDbControlLedger::open(request.ledger_path).map_err(|error| control_error(&error))?;
    let config = ValkeyHotStateConfig::new(request.valkey_url.to_owned())
        .map_err(|error| control_error(&error))?;
    let config = if let Some(namespace) = request.namespace {
        config
            .with_namespace(namespace)
            .map_err(|error| control_error(&error))?
    } else {
        config
    };
    let store = ValkeyHotStateStore::new(config);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(io::Error::other)?;
    runtime.block_on(worker_once_with_hot_state(
        &ledger,
        &store,
        &ActivityWorkerOnceStoreRequest {
            worker_id: request.worker_id,
            task_queue: request.task_queue,
            now_ms: request.now_ms,
            lease_ttl_ms: request.lease_ttl_ms,
            heartbeat_ttl_ms: None,
            executor: request.executor,
            outcome: request.outcome,
            settled_at_ms: request.settled_at_ms,
            output_ref_json: request.output_ref_json,
            output_hash: request.output_hash,
            output_artifact_path: request.output_artifact_path,
            output_artifact_dir: None,
            output_artifact_content: request.output_artifact_content,
            output_artifact_id: request.output_artifact_id,
            output_artifact_kind: request.output_artifact_kind,
            openai_compatible_base_url: request.openai_compatible_base_url,
            openai_compatible_api_key: request.openai_compatible_api_key,
            openai_compatible_timeout_ms: request.openai_compatible_timeout_ms,
            error_code: request.error_code,
            message: request.message,
            retryable: request.retryable,
            metadata: request.metadata,
            json: request.json,
        },
    ))
}

#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
pub(super) fn run(request: &ActivityWorkerOnceRunRequest<'_>) -> io::Result<ControlCliOutput> {
    let _ = (
        request.ledger_path,
        request.valkey_url,
        request.namespace,
        request.worker_id,
        request.task_queue,
        request.now_ms,
        request.lease_ttl_ms,
        request.executor,
        request.outcome,
        request.settled_at_ms,
        request.output_ref_json,
        request.output_hash,
        request.output_artifact_path,
        request.output_artifact_content,
        request.output_artifact_id,
        request.output_artifact_kind,
        request.openai_compatible_base_url,
        request.openai_compatible_api_key,
        request.openai_compatible_timeout_ms,
        request.error_code,
        request.message,
        request.retryable,
        request.metadata,
        request.json,
    );
    Err(invalid_input(
        "`control activity-worker-once` requires the `duckdb` and `valkey` features",
    ))
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
pub(crate) async fn worker_once_with_hot_state<L, H>(
    ledger: &L,
    hot_state: &H,
    request: &ActivityWorkerOnceStoreRequest<'_>,
) -> io::Result<ControlCliOutput>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
    H: xiuxian_qianji_control::HotStateStore + ?Sized,
{
    let json = request.json;
    let output = worker_once_output_with_hot_state(ledger, hot_state, request).await?;
    render_output(&output, json)
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
pub(crate) async fn worker_once_output_with_hot_state<L, H>(
    ledger: &L,
    hot_state: &H,
    request: &ActivityWorkerOnceStoreRequest<'_>,
) -> io::Result<ActivityWorkerOnceOutput>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
    H: xiuxian_qianji_control::HotStateStore + ?Sized,
{
    worker_once_output_with_claim_scope(ledger, hot_state, None, request).await
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
async fn worker_once_output_with_claim_scope<L, H>(
    ledger: &L,
    hot_state: &H,
    run_id: Option<&xiuxian_qianji_control::RunId>,
    request: &ActivityWorkerOnceStoreRequest<'_>,
) -> io::Result<ActivityWorkerOnceOutput>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
    H: xiuxian_qianji_control::HotStateStore + ?Sized,
{
    use xiuxian_qianji_control::{TaskQueue, WorkerId, WorkerRef};

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
    let claimed = match run_id {
        Some(run_id) => {
            hot_state
                .claim_activity_task_for_run(run_scoped_claim_request(
                    worker,
                    run_id,
                    task_queue.as_ref(),
                    request.now_ms,
                    request.lease_ttl_ms,
                ))
                .await
        }
        None => {
            hot_state
                .claim_activity_task(
                    worker,
                    task_queue.as_ref(),
                    request.now_ms,
                    request.lease_ttl_ms,
                )
                .await
        }
    }
    .map_err(|error| control_error(&error))?;
    let Some(claimed_task) = claimed.clone() else {
        return Ok(empty_worker_once_output(worker_id, task_queue, request));
    };
    execute_claimed_worker_once(
        ledger,
        hot_state,
        request,
        worker_id,
        task_queue,
        claimed_task,
    )
    .await
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn run_scoped_claim_request(
    worker: xiuxian_qianji_control::WorkerRef,
    run_id: &xiuxian_qianji_control::RunId,
    task_queue: Option<&xiuxian_qianji_control::TaskQueue>,
    now_ms: u64,
    lease_ttl_ms: u64,
) -> xiuxian_qianji_control::RunScopedActivityTaskClaimRequest {
    let request = xiuxian_qianji_control::RunScopedActivityTaskClaimRequest::new(
        worker,
        run_id.clone(),
        now_ms,
        lease_ttl_ms,
    );
    if let Some(task_queue) = task_queue {
        request.with_task_queue(task_queue.clone())
    } else {
        request
    }
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn empty_worker_once_output(
    worker_id: xiuxian_qianji_control::WorkerId,
    task_queue: Option<xiuxian_qianji_control::TaskQueue>,
    request: &ActivityWorkerOnceStoreRequest<'_>,
) -> ActivityWorkerOnceOutput {
    ActivityWorkerOnceOutput {
        worker_id,
        task_queue,
        executor: request.executor,
        executor_contract: None,
        outcome: request.outcome,
        claimed: None,
        start: None,
        heartbeat: None,
        terminal: None,
        released: false,
    }
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
async fn execute_claimed_worker_once<L, H>(
    ledger: &L,
    hot_state: &H,
    request: &ActivityWorkerOnceStoreRequest<'_>,
    worker_id: xiuxian_qianji_control::WorkerId,
    task_queue: Option<xiuxian_qianji_control::TaskQueue>,
    claimed_task: xiuxian_qianji_control::HotStateLeasedActivityTask,
) -> io::Result<ActivityWorkerOnceOutput>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
    H: xiuxian_qianji_control::HotStateStore + ?Sized,
{
    let executor_registry = ActivityExecutorRegistry::fixture_only();
    let executor_contract = executor_registry
        .validate_task(request.executor, Some(&claimed_task.activity_task.task))?;
    if !executor_registry.can_execute(request.executor) {
        return Err(invalid_input(format!(
            "activity executor `{}` passed admission but provider execution is not enabled in this slice",
            executor_contract.executor_label()
        )));
    }
    validate_openai_compatible_request(request)?;
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
        request.executor,
        request.now_ms,
        request.heartbeat_ttl_ms,
    )
    .await?;
    let executor_outcome =
        execute_worker_outcome(executor_registry, &claimed_task.activity_task.task, request)
            .await?;
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
    let output = ActivityWorkerOnceOutput {
        worker_id,
        task_queue,
        executor: request.executor,
        executor_contract: Some(executor_contract),
        outcome: request.outcome,
        claimed: Some(claimed_task),
        start: Some(start),
        heartbeat,
        terminal: Some(terminal),
        released,
    };
    Ok(output)
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
async fn execute_worker_outcome(
    executor_registry: ActivityExecutorRegistry,
    task: &xiuxian_qianji_control::WorkerActivityTask,
    request: &ActivityWorkerOnceStoreRequest<'_>,
) -> io::Result<ActivityExecutorOutcome> {
    match request.executor {
        ActivityExecutorKindArg::Fixture => {
            execute_fixture_outcome(executor_registry, task, request)
        }
        ActivityExecutorKindArg::OpenAiCompatibleLlm => {
            let _ = (task, request);
            Err(invalid_input(
                "`control activity-worker-once --executor openai-compatible-llm` is an admission gate only; local Qianji LLM provider execution is retired, use marlin-agent-core or an external service adapter",
            ))
        }
        ActivityExecutorKindArg::FlowhubService => {
            executor_registry.execute(ActivityExecutionRequest {
                task: Some(task),
                executor: request.executor,
                outcome: request.outcome,
                output_ref_json: None,
                output_hash: None,
                error_code: None,
                message: None,
                retryable: None,
                metadata: None,
            })
        }
    }
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn execute_fixture_outcome(
    executor_registry: ActivityExecutorRegistry,
    task: &xiuxian_qianji_control::WorkerActivityTask,
    request: &ActivityWorkerOnceStoreRequest<'_>,
) -> io::Result<ActivityExecutorOutcome> {
    let output_artifact = write_output_artifact_if_requested(task, request)?;
    let output_ref_json = output_artifact
        .as_ref()
        .map(|artifact| serde_json::to_string(&artifact.output_ref).map_err(io::Error::other))
        .transpose()?;
    let output_ref_json = request.output_ref_json.or(output_ref_json.as_deref());
    let output_hash = request.output_hash.or_else(|| {
        output_artifact
            .as_ref()
            .map(|artifact| artifact.output_hash.as_str())
    });
    executor_registry.execute(ActivityExecutionRequest {
        task: Some(task),
        executor: request.executor,
        outcome: request.outcome,
        output_ref_json,
        output_hash,
        error_code: request.error_code,
        message: request.message,
        retryable: request.retryable,
        metadata: request.metadata,
    })
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn validate_openai_compatible_request(
    request: &ActivityWorkerOnceStoreRequest<'_>,
) -> io::Result<()> {
    if request.executor != ActivityExecutorKindArg::OpenAiCompatibleLlm {
        return Ok(());
    }
    let _ = (
        request.output_artifact_dir,
        request.openai_compatible_base_url,
        request.openai_compatible_api_key,
        request.openai_compatible_timeout_ms,
    );
    Err(invalid_input(
        "`control activity-worker-once --executor openai-compatible-llm` is an admission gate only; local Qianji LLM provider execution is retired, use marlin-agent-core or an external service adapter",
    ))
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn write_output_artifact_if_requested(
    task: &xiuxian_qianji_control::WorkerActivityTask,
    request: &ActivityWorkerOnceStoreRequest<'_>,
) -> io::Result<Option<ActivityOutputArtifact>> {
    let Some(path) = request.output_artifact_path else {
        return Ok(None);
    };
    let content = request.output_artifact_content.ok_or_else(|| {
        invalid_input(
            "missing `--output-artifact-content <text>` for `control activity-worker-once`",
        )
    })?;
    write_activity_output_artifact(
        task,
        ActivityOutputArtifactRequest {
            path,
            content,
            artifact_id: request.output_artifact_id,
            artifact_kind: request.output_artifact_kind,
        },
    )
    .map(Some)
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn record_terminal<L>(
    ledger: &L,
    claimed_task: &xiuxian_qianji_control::HotStateLeasedActivityTask,
    settled_at_ms: u64,
    outcome: ActivityExecutorOutcome,
) -> io::Result<xiuxian_qianji_control::ActivityJournalWriteOutcome>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
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

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
async fn record_worker_heartbeat_if_enabled<L, H>(
    ledger: &L,
    hot_state: &H,
    claimed_task: &xiuxian_qianji_control::HotStateLeasedActivityTask,
    worker_id: &xiuxian_qianji_control::WorkerId,
    executor: ActivityExecutorKindArg,
    observed_at_ms: u64,
    heartbeat_ttl_ms: Option<u64>,
) -> io::Result<Option<xiuxian_qianji_control::ControlEventRecord>>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
    H: xiuxian_qianji_control::HotStateStore + ?Sized,
{
    let Some(heartbeat_ttl_ms) = heartbeat_ttl_ms else {
        return Ok(None);
    };
    let expires_at_ms = observed_at_ms
        .checked_add(heartbeat_ttl_ms)
        .ok_or_else(|| {
            invalid_input("`control activity-worker-loop --heartbeat-ttl-ms` overflowed u64")
        })?;
    let heartbeat = xiuxian_qianji_control::WorkerHeartbeat {
        worker_id: worker_id.clone(),
        observed_at_ms,
        expires_at_ms,
        metadata: heartbeat_metadata(claimed_task, executor),
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

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn heartbeat_metadata(
    claimed_task: &xiuxian_qianji_control::HotStateLeasedActivityTask,
    executor: ActivityExecutorKindArg,
) -> serde_json::Value {
    serde_json::json!({
        "source": "qianji-control-activity-worker",
        "executor": activity_executor_label(executor),
        "phase": activity_worker_phase(executor),
        "activity_id": claimed_task.activity_task.task.activity_id.as_str(),
        "activity_type": claimed_task.activity_task.task.activity_type.as_str(),
        "task_queue": claimed_task.activity_task.task.task_queue.as_str(),
        "next_attempt": claimed_task.activity_task.task.next_attempt,
    })
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
const fn activity_executor_label(executor: ActivityExecutorKindArg) -> &'static str {
    match executor {
        ActivityExecutorKindArg::Fixture => "fixture",
        ActivityExecutorKindArg::OpenAiCompatibleLlm => "openai-compatible-llm",
        ActivityExecutorKindArg::FlowhubService => "flowhub-service",
    }
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
const fn activity_worker_phase(executor: ActivityExecutorKindArg) -> &'static str {
    match executor {
        ActivityExecutorKindArg::OpenAiCompatibleLlm => "provider_request_active",
        ActivityExecutorKindArg::Fixture | ActivityExecutorKindArg::FlowhubService => {
            "activity_execution_active"
        }
    }
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn record_complete<L>(
    ledger: &L,
    claimed_task: &xiuxian_qianji_control::HotStateLeasedActivityTask,
    settled_at_ms: u64,
    result: xiuxian_qianji_control::ActivityResult,
) -> io::Result<xiuxian_qianji_control::ActivityJournalWriteOutcome>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
{
    use xiuxian_qianji_control::{
        WorkerActivityCompletedRecord, record_worker_activity_completed_idempotent,
    };

    let complete_record = WorkerActivityCompletedRecord::new(
        claimed_task.activity_task.task.clone(),
        settled_at_ms,
        result,
    );
    record_worker_activity_completed_idempotent(ledger, complete_record)
        .map_err(|error| control_error(&error))
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn record_fail<L>(
    ledger: &L,
    claimed_task: &xiuxian_qianji_control::HotStateLeasedActivityTask,
    settled_at_ms: u64,
    error_code: xiuxian_qianji_control::ErrorCode,
    message: &str,
    retryable: bool,
    metadata: serde_json::Value,
) -> io::Result<xiuxian_qianji_control::ActivityJournalWriteOutcome>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
{
    use xiuxian_qianji_control::{
        WorkerActivityFailedRecord, WorkerActivityFailureInput,
        record_worker_activity_failed_idempotent,
    };

    let failure_record = WorkerActivityFailedRecord::new(
        WorkerActivityFailureInput::new(
            claimed_task.activity_task.task.clone(),
            error_code,
            message,
        )
        .with_failed_at_ms(settled_at_ms)
        .with_retryable(retryable),
    )
    .with_metadata(metadata);
    record_worker_activity_failed_idempotent(ledger, failure_record)
        .map_err(|error| control_error(&error))
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn render_output(output: &ActivityWorkerOnceOutput, json: bool) -> io::Result<ControlCliOutput> {
    let rendered = if json {
        serde_json::to_string_pretty(output).map_err(io::Error::other)?
    } else {
        render_activity_worker_once_text(output)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn render_activity_worker_once_text(output: &ActivityWorkerOnceOutput) -> String {
    let task_queue = output
        .task_queue
        .as_ref()
        .map_or("<all>", |queue| queue.as_str());
    format!(
        concat!(
            "# Qianji Control Activity Worker Once\n\n",
            "- Worker: `{}`\n",
            "- Task queue: `{}`\n",
            "- Executor: `{:?}`\n",
            "- Executor contract: `{}`\n",
            "- Outcome: `{:?}`\n",
            "- Claimed: `{}`\n",
            "- Durable start: `{}`\n",
            "- Durable terminal: `{}`\n",
            "- Released: `{}`\n"
        ),
        output.worker_id.as_str(),
        task_queue,
        output.executor,
        output.executor_contract.is_some(),
        output.outcome,
        output.claimed.is_some(),
        output.start.is_some(),
        output.terminal.is_some(),
        output.released
    )
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
