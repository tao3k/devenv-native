//! Bounded LLM worker helpers for qianji-server.
//!
//! qianji-server owns durable control admission, hot-state polling, and
//! activity lifecycle recording. Provider HTTP execution remains delegated to
//! the shared `xiuxian-llm` OpenAI-compatible transport through the governed
//! Qianji activity executor.

use std::io;
use std::path::Path;

use crate::qianji_cli::{
    ActivityExecutorKindArg, ActivitySettleOutcomeArg, ActivityWorkerLoopStoreRequest,
    worker_loop_output_with_hot_state,
};
use xiuxian_qianji_control::{
    ControlLedger, HotStateStore, RunId, TaskQueue, WorkerActivityHotStateMirrorRequest,
    mirror_worker_activity_tasks_to_hot_state,
};

/// qianji-server request for one bounded OpenAI-compatible LLM worker loop.
#[derive(Clone, Copy)]
pub struct QianjiServerOpenAiCompatibleLlmWorkerLoopRequest<'a> {
    /// Durable control run that owns the queued `llm.*` work.
    pub run_id: &'a RunId,
    /// Server worker identity used for claims and lifecycle events.
    pub worker_id: &'a str,
    /// Optional `llm.*` task queue filter.
    pub task_queue: Option<&'a str>,
    /// Logical poll timestamp for the first worker attempt.
    pub now_ms: u64,
    /// Per-poll logical timestamp increment.
    pub now_step_ms: u64,
    /// Hot-state lease TTL.
    pub lease_ttl_ms: u64,
    /// Optional heartbeat TTL.
    pub heartbeat_ttl_ms: Option<u64>,
    /// Maximum poll attempts for this bounded loop.
    pub poll_limit: u32,
    /// Number of consecutive empty polls that stops the loop.
    pub empty_limit: u32,
    /// Number of concurrent worker identities used per batch.
    pub worker_count: u32,
    /// Logical terminal-event timestamp for the first worker attempt.
    pub settled_at_ms: u64,
    /// Per-poll terminal timestamp increment.
    pub settled_step_ms: u64,
    /// OpenAI-compatible base URL resolved by qianji-server config.
    pub openai_compatible_base_url: &'a str,
    /// Optional API key resolved by qianji-server config.
    pub openai_compatible_api_key: Option<&'a str>,
    /// Optional provider timeout.
    pub openai_compatible_timeout_ms: Option<u64>,
    /// Directory for provider response artifacts.
    pub output_artifact_dir: &'a Path,
    /// Optional artifact kind for provider response artifacts.
    pub output_artifact_kind: Option<&'a str>,
}

/// qianji-server result of one bounded OpenAI-compatible LLM worker loop.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QianjiServerOpenAiCompatibleLlmWorkerLoopOutput {
    /// Server worker identity used for claims and lifecycle events.
    pub worker_id: String,
    /// Optional `llm.*` task queue filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_queue: Option<String>,
    /// Maximum poll attempts for this bounded loop.
    pub poll_limit: u32,
    /// Number of consecutive empty polls that stops the loop.
    pub empty_limit: u32,
    /// Number of concurrent worker identities used per batch.
    pub worker_count: u32,
    /// Per-poll activity execution trace.
    pub iterations: Vec<QianjiServerOpenAiCompatibleLlmWorkerStepOutput>,
    /// Number of claimed and processed tasks.
    pub processed: u32,
    /// Number of empty polls.
    pub empty_polls: u32,
    /// Number of released leases.
    pub released: u32,
    /// Number of recorded heartbeats.
    pub heartbeats: u32,
    /// Loop stop reason rendered as a stable label.
    pub stopped_reason: String,
}

/// qianji-server trace row for one OpenAI-compatible LLM worker poll.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QianjiServerOpenAiCompatibleLlmWorkerStepOutput {
    /// Poll index inside the bounded loop.
    pub poll_index: u32,
    /// Logical poll timestamp.
    pub now_ms: u64,
    /// Logical terminal timestamp.
    pub settled_at_ms: u64,
    /// Claimed activity id, when the poll found runnable work.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_id: Option<String>,
    /// Whether a start event was recorded.
    pub start_recorded: bool,
    /// Whether a terminal completed/failed event was recorded.
    pub terminal_recorded: bool,
    /// Whether the hot-state lease was released.
    pub released: bool,
}

/// Runs one bounded qianji-server OpenAI-compatible LLM worker loop.
///
/// The function mirrors replay-derived pending `llm.*` tasks into hot state,
/// claims work, records start/terminal lifecycle events, executes provider
/// calls through `xiuxian-llm`, and releases leases. It is intentionally
/// bounded so the HTTP/server owner can choose whether to invoke it once,
/// repeat it, or run it in a supervised background process.
///
/// # Errors
///
/// Returns an I/O error when task-queue parsing, hot-state mirroring,
/// provider execution, artifact writing, or durable activity lifecycle
/// recording fails.
pub async fn run_qianji_server_openai_compatible_llm_worker_loop<L, H>(
    ledger: &L,
    hot_state: &H,
    request: QianjiServerOpenAiCompatibleLlmWorkerLoopRequest<'_>,
) -> io::Result<QianjiServerOpenAiCompatibleLlmWorkerLoopOutput>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
{
    validate_loop_bounds(&request)?;
    let task_queue = request
        .task_queue
        .map(TaskQueue::new)
        .transpose()
        .map_err(control_error)?;
    let mirror_request = match task_queue.clone() {
        Some(queue) => WorkerActivityHotStateMirrorRequest::new(request.run_id.clone())
            .with_task_queue(queue)
            .with_not_before_ms(request.now_ms)
            .with_metadata(serde_json::json!({
                "source": "qianji-server-llm-worker",
                "worker_id": request.worker_id,
            })),
        None => WorkerActivityHotStateMirrorRequest::new(request.run_id.clone())
            .with_not_before_ms(request.now_ms)
            .with_metadata(serde_json::json!({
                "source": "qianji-server-llm-worker",
                "worker_id": request.worker_id,
            })),
    };
    mirror_worker_activity_tasks_to_hot_state(ledger, hot_state, mirror_request)
        .await
        .map_err(control_error)?;
    let output = worker_loop_output_with_hot_state(
        ledger,
        hot_state,
        ActivityWorkerLoopStoreRequest {
            worker_id: request.worker_id,
            task_queue: request.task_queue,
            now_ms: request.now_ms,
            now_step_ms: request.now_step_ms,
            lease_ttl_ms: request.lease_ttl_ms,
            heartbeat_ttl_ms: request.heartbeat_ttl_ms,
            poll_limit: request.poll_limit,
            empty_limit: request.empty_limit,
            worker_count: request.worker_count,
            executor: ActivityExecutorKindArg::OpenAiCompatibleLlm,
            outcome: ActivitySettleOutcomeArg::Complete,
            settled_at_ms: request.settled_at_ms,
            settled_step_ms: request.settled_step_ms,
            output_hash: None,
            output_artifact_dir: Some(request.output_artifact_dir),
            output_artifact_kind: request.output_artifact_kind,
            openai_compatible_base_url: Some(request.openai_compatible_base_url),
            openai_compatible_api_key: request.openai_compatible_api_key,
            openai_compatible_timeout_ms: request.openai_compatible_timeout_ms,
            error_code: None,
            message: None,
            retryable: None,
            metadata: None,
            json: true,
        },
    )
    .await?;
    Ok(QianjiServerOpenAiCompatibleLlmWorkerLoopOutput {
        worker_id: output.worker_id,
        task_queue: output.task_queue,
        poll_limit: output.poll_limit,
        empty_limit: output.empty_limit,
        worker_count: output.worker_count,
        iterations: output
            .iterations
            .into_iter()
            .map(|iteration| {
                let activity_id = iteration
                    .output
                    .claimed
                    .as_ref()
                    .map(|claimed| claimed.activity_task.task.activity_id.as_str().to_owned());
                QianjiServerOpenAiCompatibleLlmWorkerStepOutput {
                    poll_index: iteration.poll_index,
                    now_ms: iteration.now_ms,
                    settled_at_ms: iteration.settled_at_ms,
                    activity_id,
                    start_recorded: iteration.output.start.is_some(),
                    terminal_recorded: iteration.output.terminal.is_some(),
                    released: iteration.output.released,
                }
            })
            .collect(),
        processed: output.processed,
        empty_polls: output.empty_polls,
        released: output.released,
        heartbeats: output.heartbeats,
        stopped_reason: format!("{:?}", output.stopped_reason),
    })
}

fn validate_loop_bounds(
    request: &QianjiServerOpenAiCompatibleLlmWorkerLoopRequest<'_>,
) -> io::Result<()> {
    if request.worker_id.trim().is_empty() {
        return Err(invalid_input(
            "qianji-server LLM worker requires a non-empty worker_id",
        ));
    }
    if request.poll_limit == 0 {
        return Err(invalid_input(
            "qianji-server LLM worker requires a positive poll_limit",
        ));
    }
    if request.empty_limit == 0 {
        return Err(invalid_input(
            "qianji-server LLM worker requires a positive empty_limit",
        ));
    }
    if request.worker_count == 0 {
        return Err(invalid_input(
            "qianji-server LLM worker requires a positive worker_count",
        ));
    }
    if request.lease_ttl_ms == 0 {
        return Err(invalid_input(
            "qianji-server LLM worker requires a positive lease_ttl_ms",
        ));
    }
    if request.openai_compatible_base_url.trim().is_empty() {
        return Err(invalid_input(
            "qianji-server LLM worker requires an OpenAI-compatible base URL",
        ));
    }
    Ok(())
}

fn control_error(error: impl std::fmt::Display) -> io::Error {
    invalid_input(format!("{error}"))
}

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}
