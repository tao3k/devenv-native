//! Bounded LLM worker helpers for qianji-server.
//!
//! qianji-server owns durable control admission, hot-state polling, and
//! activity lifecycle recording. Provider HTTP execution remains delegated to
//! the shared `xiuxian-llm` OpenAI-compatible transport through the governed
//! Qianji activity executor.

use std::io;
use std::path::Path;

use crate::qianji_worker::{
    OpenAiCompatibleWorkerOnceOutput, OpenAiCompatibleWorkerOnceRequest,
    run_openai_compatible_worker_once_for_run,
};
use xiuxian_qianji_control::{
    ControlEventKind, ControlLedger, HotStateStore, RunId, TaskQueue,
    WorkerActivityHotStateMirrorRequest, mirror_worker_activity_tasks_to_hot_state,
};

const BPMN_LLM_ACTIVITY_METADATA_SCHEMA: &str = "qianji.bpmn.host_work.llm_activity_metadata.v1";

/// qianji-server request for one bounded OpenAI-compatible LLM worker loop.
#[derive(Clone, Copy)]
pub(crate) struct QianjiServerOpenAiCompatibleLlmWorkerLoopRequest<'a> {
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
pub(crate) struct QianjiServerOpenAiCompatibleLlmWorkerLoopOutput {
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
pub(crate) struct QianjiServerOpenAiCompatibleLlmWorkerStepOutput {
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
    /// BPMN host-work completion candidate derived from server-owned LLM
    /// activity metadata and provider output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bpmn_completion: Option<QianjiServerOpenAiCompatibleLlmBpmnCompletionCandidate>,
}

/// Server-owned facts needed to complete a BPMN host-work item after an LLM
/// activity finishes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QianjiServerOpenAiCompatibleLlmBpmnCompletionCandidate {
    /// BPMN workflow instance id stored in the checkpoint backend.
    pub instance_id: String,
    /// BPMN source path recorded when qianji-server admitted the activity.
    pub bpmn_source_ref: String,
    /// Runtime token identifier for the pending host work.
    pub token_id: u64,
    /// BPMN process identifier expected for the pending host work.
    pub process_id: String,
    /// BPMN activity identifier expected for the pending host work.
    pub activity_id: String,
    /// Pending host-work completion kind.
    pub completion_kind: String,
    /// Declared BPMN output binding names for deterministic single-output
    /// completion shaping.
    pub output_bindings: Vec<String>,
    /// Provider response artifact URI.
    pub output_ref_uri: String,
    /// Provider response content hash, when recorded by the worker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_hash: Option<String>,
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
pub(crate) async fn run_qianji_server_openai_compatible_llm_worker_loop<L, H>(
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
    run_scoped_worker_loop(ledger, hot_state, request).await
}

async fn run_scoped_worker_loop<L, H>(
    ledger: &L,
    hot_state: &H,
    request: QianjiServerOpenAiCompatibleLlmWorkerLoopRequest<'_>,
) -> io::Result<QianjiServerOpenAiCompatibleLlmWorkerLoopOutput>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
{
    let worker_count = request.worker_count.max(1);
    let poll_plan = worker_poll_plan(&request, worker_count)?;
    let mut accumulator = WorkerLoopAccumulator::new();

    for poll in poll_plan {
        let poll_output = execute_worker_poll(ledger, hot_state, &request, poll).await?;
        if accumulator.record(poll_output, request.empty_limit) {
            break;
        }
    }

    Ok(accumulator.into_output(&request, worker_count))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkerPollPlan {
    poll_index: u32,
    worker_id: String,
    now_ms: u64,
    settled_at_ms: u64,
}

fn worker_poll_plan(
    request: &QianjiServerOpenAiCompatibleLlmWorkerLoopRequest<'_>,
    worker_count: u32,
) -> io::Result<Vec<WorkerPollPlan>> {
    (0..request.poll_limit)
        .map(|poll_index| {
            Ok(WorkerPollPlan {
                poll_index,
                worker_id: scoped_worker_id(request.worker_id, worker_count, poll_index),
                now_ms: stepped_ms(request.now_ms, request.now_step_ms, poll_index)?,
                settled_at_ms: stepped_ms(
                    request.settled_at_ms,
                    request.settled_step_ms,
                    poll_index,
                )?,
            })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkerPollOutput {
    step: QianjiServerOpenAiCompatibleLlmWorkerStepOutput,
    claimed: bool,
    released: bool,
    heartbeat: bool,
}

async fn execute_worker_poll<L, H>(
    ledger: &L,
    hot_state: &H,
    request: &QianjiServerOpenAiCompatibleLlmWorkerLoopRequest<'_>,
    poll: WorkerPollPlan,
) -> io::Result<WorkerPollOutput>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
{
    let output = run_openai_compatible_worker_once_for_run(
        ledger,
        hot_state,
        request.run_id,
        &OpenAiCompatibleWorkerOnceRequest {
            worker_id: poll.worker_id.as_str(),
            task_queue: request.task_queue,
            now_ms: poll.now_ms,
            lease_ttl_ms: request.lease_ttl_ms,
            heartbeat_ttl_ms: request.heartbeat_ttl_ms,
            settled_at_ms: poll.settled_at_ms,
            output_artifact_id: None,
            output_artifact_kind: request.output_artifact_kind,
            output_artifact_dir: request.output_artifact_dir,
            openai_compatible_base_url: request.openai_compatible_base_url,
            openai_compatible_api_key: request.openai_compatible_api_key,
            openai_compatible_timeout_ms: request.openai_compatible_timeout_ms,
        },
    )
    .await?;
    let activity_id = output
        .claimed
        .as_ref()
        .map(|claimed| claimed.activity_task.task.activity_id.as_str().to_owned());
    let bpmn_completion = bpmn_completion_candidate(&output)?;
    Ok(WorkerPollOutput {
        step: QianjiServerOpenAiCompatibleLlmWorkerStepOutput {
            poll_index: poll.poll_index,
            now_ms: poll.now_ms,
            settled_at_ms: poll.settled_at_ms,
            activity_id,
            start_recorded: output.start.is_some(),
            terminal_recorded: output.terminal.is_some(),
            released: output.released,
            bpmn_completion,
        },
        claimed: output.claimed.is_some(),
        released: output.released,
        heartbeat: output.heartbeat.is_some(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkerLoopAccumulator {
    iterations: Vec<QianjiServerOpenAiCompatibleLlmWorkerStepOutput>,
    processed: u32,
    empty_polls: u32,
    empty_streak: u32,
    released: u32,
    heartbeats: u32,
    stopped_reason: &'static str,
}

impl WorkerLoopAccumulator {
    fn new() -> Self {
        Self {
            iterations: Vec::new(),
            processed: 0,
            empty_polls: 0,
            empty_streak: 0,
            released: 0,
            heartbeats: 0,
            stopped_reason: "PollLimit",
        }
    }

    fn record(&mut self, poll: WorkerPollOutput, empty_limit: u32) -> bool {
        if poll.claimed {
            self.processed += 1;
            self.empty_streak = 0;
        } else {
            self.empty_polls += 1;
            self.empty_streak += 1;
        }
        if poll.released {
            self.released += 1;
        }
        if poll.heartbeat {
            self.heartbeats += 1;
        }
        self.iterations.push(poll.step);
        if self.empty_streak >= empty_limit {
            self.stopped_reason = "EmptyLimit";
            return true;
        }
        false
    }

    fn into_output(
        self,
        request: &QianjiServerOpenAiCompatibleLlmWorkerLoopRequest<'_>,
        worker_count: u32,
    ) -> QianjiServerOpenAiCompatibleLlmWorkerLoopOutput {
        QianjiServerOpenAiCompatibleLlmWorkerLoopOutput {
            worker_id: request.worker_id.to_owned(),
            task_queue: request.task_queue.map(str::to_owned),
            poll_limit: request.poll_limit,
            empty_limit: request.empty_limit,
            worker_count,
            iterations: self.iterations,
            processed: self.processed,
            empty_polls: self.empty_polls,
            released: self.released,
            heartbeats: self.heartbeats,
            stopped_reason: self.stopped_reason.to_owned(),
        }
    }
}

fn bpmn_completion_candidate(
    output: &OpenAiCompatibleWorkerOnceOutput,
) -> io::Result<Option<QianjiServerOpenAiCompatibleLlmBpmnCompletionCandidate>> {
    let Some(claimed) = output.claimed.as_ref() else {
        return Ok(None);
    };
    let metadata = &claimed.activity_task.task.metadata;
    if metadata.get("schema").and_then(serde_json::Value::as_str)
        != Some(BPMN_LLM_ACTIVITY_METADATA_SCHEMA)
    {
        return Ok(None);
    }
    let Some(terminal) = output.terminal.as_ref() else {
        return Ok(None);
    };
    let ControlEventKind::ActivityCompleted { result, .. } = &terminal.record.event.kind else {
        return Ok(None);
    };
    let output_ref = result.output_ref.as_ref().ok_or_else(|| {
        invalid_input("completed BPMN LLM activity is missing provider output_ref")
    })?;
    Ok(Some(
        QianjiServerOpenAiCompatibleLlmBpmnCompletionCandidate {
            instance_id: required_metadata_str(metadata, "instance_id")?.to_owned(),
            bpmn_source_ref: required_metadata_str(metadata, "bpmn_source_ref")?.to_owned(),
            token_id: required_metadata_u64(metadata, "token_id")?,
            process_id: required_metadata_str(metadata, "process_id")?.to_owned(),
            activity_id: required_metadata_str(metadata, "activity_id")?.to_owned(),
            completion_kind: completion_kind_from_metadata(metadata)?,
            output_bindings: metadata_string_array(metadata, "output_bindings"),
            output_ref_uri: output_ref.uri.clone(),
            output_hash: result.output_hash.clone(),
        },
    ))
}

fn required_metadata_str<'a>(
    metadata: &'a serde_json::Value,
    field: &'static str,
) -> io::Result<&'a str> {
    metadata
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            invalid_input(format!(
                "BPMN LLM activity metadata is missing non-empty `{field}`"
            ))
        })
}

fn required_metadata_u64(metadata: &serde_json::Value, field: &'static str) -> io::Result<u64> {
    metadata
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| invalid_input(format!("BPMN LLM activity metadata is missing `{field}`")))
}

fn metadata_string_array(metadata: &serde_json::Value, field: &'static str) -> Vec<String> {
    metadata
        .get(field)
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn completion_kind_from_metadata(metadata: &serde_json::Value) -> io::Result<String> {
    let kind = required_metadata_str(metadata, "pending_work_kind")?;
    match kind {
        "Task" => Ok("task".to_owned()),
        "Send" => Ok("send".to_owned()),
        "Service" => Ok("service".to_owned()),
        "Script" => Ok("script".to_owned()),
        "User" => Ok("user".to_owned()),
        "Manual" => Ok("manual".to_owned()),
        other => Err(invalid_input(format!(
            "BPMN LLM activity metadata has unsupported pending_work_kind `{other}`"
        ))),
    }
}

fn scoped_worker_id(worker_id: &str, worker_count: u32, poll_index: u32) -> String {
    if worker_count <= 1 {
        return worker_id.to_owned();
    }
    let slot = poll_index % worker_count;
    format!("{worker_id}-{slot}")
}

fn stepped_ms(base_ms: u64, step_ms: u64, poll_index: u32) -> io::Result<u64> {
    let offset = step_ms
        .checked_mul(u64::from(poll_index))
        .ok_or_else(|| invalid_input("worker loop timestamp overflow"))?;
    base_ms
        .checked_add(offset)
        .ok_or_else(|| invalid_input("worker loop timestamp overflow"))
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
