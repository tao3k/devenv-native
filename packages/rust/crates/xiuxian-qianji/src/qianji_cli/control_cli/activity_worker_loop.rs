use std::io;
use std::path::{Path, PathBuf};

use crate::qianji_cli::{invalid_input, parse_flag_value};

use super::activity_args::ActivitySettleOutcomeArg;
use super::activity_executor::ActivityExecutorKindArg;
#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
use super::activity_worker_once::{
    ActivityWorkerOnceOutput, ActivityWorkerOnceStoreRequest, worker_once_output_with_hot_state,
};
use super::types::{ControlCliCommand, ControlCliOutput};

pub(super) fn parse(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut parsed = ActivityWorkerLoopArgs::default();
    let mut index = 3;
    while index < args.len() {
        parsed.parse_flag(args, &mut index)?;
        index += 1;
    }
    parsed.into_command()
}

#[derive(Default)]
struct ActivityWorkerLoopArgs {
    ledger_path: Option<PathBuf>,
    valkey_url: Option<String>,
    namespace: Option<String>,
    worker_id: Option<String>,
    task_queue: Option<String>,
    now_ms: Option<u64>,
    now_step_ms: Option<u64>,
    lease_ttl_ms: Option<u64>,
    heartbeat_ttl_ms: Option<u64>,
    poll_limit: Option<u32>,
    empty_limit: Option<u32>,
    executor: Option<ActivityExecutorKindArg>,
    outcome: Option<ActivitySettleOutcomeArg>,
    settled_at_ms: Option<u64>,
    settled_step_ms: Option<u64>,
    output_hash: Option<String>,
    output_artifact_dir: Option<PathBuf>,
    output_artifact_kind: Option<String>,
    openai_compatible_base_url: Option<String>,
    openai_compatible_api_key: Option<String>,
    openai_compatible_timeout_ms: Option<u64>,
    error_code: Option<String>,
    message: Option<String>,
    retryable: Option<bool>,
    metadata: Option<String>,
    json: bool,
}

impl ActivityWorkerLoopArgs {
    fn parse_flag(&mut self, args: &[String], index: &mut usize) -> io::Result<()> {
        if self.parse_timing_flag(args, index)? {
            return Ok(());
        }
        match args[*index].as_str() {
            "--ledger" => {
                self.ledger_path = Some(PathBuf::from(parse_flag_value(args, index, "--ledger")?));
            }
            "--valkey-url" => {
                self.valkey_url = Some(parse_flag_value(args, index, "--valkey-url")?);
            }
            "--namespace" => {
                self.namespace = Some(parse_flag_value(args, index, "--namespace")?);
            }
            "--worker-id" => {
                self.worker_id = Some(parse_flag_value(args, index, "--worker-id")?);
            }
            "--task-queue" => {
                self.task_queue = Some(parse_flag_value(args, index, "--task-queue")?);
            }
            "--executor" => {
                self.executor = Some(super::activity_executor::parse_executor(
                    &parse_flag_value(args, index, "--executor")?,
                )?);
            }
            "--outcome" => {
                self.outcome = Some(parse_outcome(&parse_flag_value(args, index, "--outcome")?)?);
            }
            "--output-hash" => {
                self.output_hash = Some(parse_flag_value(args, index, "--output-hash")?);
            }
            "--output-artifact-dir" => {
                self.output_artifact_dir = Some(PathBuf::from(parse_flag_value(
                    args,
                    index,
                    "--output-artifact-dir",
                )?));
            }
            "--output-artifact-kind" => {
                self.output_artifact_kind =
                    Some(parse_flag_value(args, index, "--output-artifact-kind")?);
            }
            "--openai-compatible-base-url" => {
                self.openai_compatible_base_url = Some(parse_flag_value(
                    args,
                    index,
                    "--openai-compatible-base-url",
                )?);
            }
            "--openai-compatible-api-key" => {
                self.openai_compatible_api_key = Some(parse_flag_value(
                    args,
                    index,
                    "--openai-compatible-api-key",
                )?);
            }
            "--openai-compatible-timeout-ms" => {
                self.openai_compatible_timeout_ms =
                    Some(parse_u64_flag(args, index, "openai-compatible-timeout-ms")?);
            }
            "--error-code" => {
                self.error_code = Some(parse_flag_value(args, index, "--error-code")?);
            }
            "--message" => {
                self.message = Some(parse_flag_value(args, index, "--message")?);
            }
            "--retryable" => {
                self.retryable = Some(parse_bool(&parse_flag_value(args, index, "--retryable")?)?);
            }
            "--metadata" => {
                self.metadata = Some(parse_flag_value(args, index, "--metadata")?);
            }
            "--json" => {
                self.json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control activity-worker-loop` does not accept argument `{other}`"
                )));
            }
        }
        Ok(())
    }

    fn parse_timing_flag(&mut self, args: &[String], index: &mut usize) -> io::Result<bool> {
        match args[*index].as_str() {
            "--now-ms" => {
                self.now_ms = Some(parse_u64_flag(args, index, "now-ms")?);
            }
            "--now-step-ms" => {
                self.now_step_ms = Some(parse_u64_flag(args, index, "now-step-ms")?);
            }
            "--lease-ttl-ms" => {
                self.lease_ttl_ms = Some(parse_u64_flag(args, index, "lease-ttl-ms")?);
            }
            "--heartbeat-ttl-ms" => {
                self.heartbeat_ttl_ms = Some(parse_u64_flag(args, index, "heartbeat-ttl-ms")?);
            }
            "--poll-limit" => {
                self.poll_limit = Some(parse_u32_flag(args, index, "poll-limit")?);
            }
            "--empty-limit" => {
                self.empty_limit = Some(parse_u32_flag(args, index, "empty-limit")?);
            }
            "--settled-at-ms" => {
                self.settled_at_ms = Some(parse_u64_flag(args, index, "settled-at-ms")?);
            }
            "--settled-step-ms" => {
                self.settled_step_ms = Some(parse_u64_flag(args, index, "settled-step-ms")?);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn into_command(self) -> io::Result<ControlCliCommand> {
        let outcome = self.outcome.ok_or_else(|| {
            invalid_input("missing `--outcome <complete|fail>` for `control activity-worker-loop`")
        })?;
        let executor = self.executor.ok_or_else(|| {
            invalid_input("missing `--executor fixture` for `control activity-worker-loop`")
        })?;
        validate_outcome_args(executor, outcome, &self)?;
        Ok(ControlCliCommand::ActivityWorkerLoop {
            ledger_path: self.ledger_path.ok_or_else(|| {
                invalid_input("missing `--ledger <path>` for `control activity-worker-loop`")
            })?,
            valkey_url: self.valkey_url.ok_or_else(|| {
                invalid_input("missing `--valkey-url <url>` for `control activity-worker-loop`")
            })?,
            namespace: self.namespace,
            worker_id: self.worker_id.ok_or_else(|| {
                invalid_input("missing `--worker-id <id>` for `control activity-worker-loop`")
            })?,
            task_queue: self.task_queue,
            now_ms: self.now_ms.ok_or_else(|| {
                invalid_input("missing `--now-ms <ms>` for `control activity-worker-loop`")
            })?,
            now_step_ms: self.now_step_ms.unwrap_or(1),
            lease_ttl_ms: self.lease_ttl_ms.ok_or_else(|| {
                invalid_input("missing `--lease-ttl-ms <ms>` for `control activity-worker-loop`")
            })?,
            heartbeat_ttl_ms: self
                .heartbeat_ttl_ms
                .map(|ttl_ms| positive_u64("heartbeat-ttl-ms", ttl_ms))
                .transpose()?,
            poll_limit: positive_u32(
                "poll-limit",
                self.poll_limit.ok_or_else(|| {
                    invalid_input("missing `--poll-limit <n>` for `control activity-worker-loop`")
                })?,
            )?,
            empty_limit: positive_u32("empty-limit", self.empty_limit.unwrap_or(1))?,
            executor,
            outcome,
            settled_at_ms: self.settled_at_ms.ok_or_else(|| {
                invalid_input("missing `--settled-at-ms <ms>` for `control activity-worker-loop`")
            })?,
            settled_step_ms: self.settled_step_ms.unwrap_or(1),
            output_hash: self.output_hash,
            output_artifact_dir: self.output_artifact_dir,
            output_artifact_kind: self.output_artifact_kind,
            openai_compatible_base_url: self.openai_compatible_base_url,
            openai_compatible_api_key: self.openai_compatible_api_key,
            openai_compatible_timeout_ms: self.openai_compatible_timeout_ms,
            error_code: self.error_code,
            message: self.message,
            retryable: self.retryable,
            metadata: self.metadata,
            json: self.json,
        })
    }
}

#[derive(Clone, Copy)]
pub(super) struct ActivityWorkerLoopRunRequest<'a> {
    pub(super) ledger_path: &'a Path,
    pub(super) valkey_url: &'a str,
    pub(super) namespace: Option<&'a str>,
    pub(super) worker_id: &'a str,
    pub(super) task_queue: Option<&'a str>,
    pub(super) now_ms: u64,
    pub(super) now_step_ms: u64,
    pub(super) lease_ttl_ms: u64,
    pub(super) heartbeat_ttl_ms: Option<u64>,
    pub(super) poll_limit: u32,
    pub(super) empty_limit: u32,
    pub(super) executor: ActivityExecutorKindArg,
    pub(super) outcome: ActivitySettleOutcomeArg,
    pub(super) settled_at_ms: u64,
    pub(super) settled_step_ms: u64,
    pub(super) output_hash: Option<&'a str>,
    pub(super) output_artifact_dir: Option<&'a Path>,
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
pub(crate) struct ActivityWorkerLoopStoreRequest<'a> {
    pub(crate) worker_id: &'a str,
    pub(crate) task_queue: Option<&'a str>,
    pub(crate) now_ms: u64,
    pub(crate) now_step_ms: u64,
    pub(crate) lease_ttl_ms: u64,
    pub(crate) heartbeat_ttl_ms: Option<u64>,
    pub(crate) poll_limit: u32,
    pub(crate) empty_limit: u32,
    pub(crate) executor: ActivityExecutorKindArg,
    pub(crate) outcome: ActivitySettleOutcomeArg,
    pub(crate) settled_at_ms: u64,
    pub(crate) settled_step_ms: u64,
    pub(crate) output_hash: Option<&'a str>,
    pub(crate) output_artifact_dir: Option<&'a Path>,
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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ActivityWorkerLoopStopReason {
    PollLimit,
    EmptyLimit,
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub(crate) struct ActivityWorkerLoopIteration {
    pub(crate) poll_index: u32,
    pub(crate) now_ms: u64,
    pub(crate) settled_at_ms: u64,
    pub(crate) output: ActivityWorkerOnceOutput,
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub(crate) struct ActivityWorkerLoopOutput {
    pub(crate) worker_id: String,
    pub(crate) task_queue: Option<String>,
    pub(crate) executor: ActivityExecutorKindArg,
    pub(crate) outcome: ActivitySettleOutcomeArg,
    pub(crate) poll_limit: u32,
    pub(crate) empty_limit: u32,
    pub(crate) iterations: Vec<ActivityWorkerLoopIteration>,
    pub(crate) processed: u32,
    pub(crate) empty_polls: u32,
    pub(crate) released: u32,
    pub(crate) heartbeats: u32,
    pub(crate) stopped_reason: ActivityWorkerLoopStopReason,
}

#[cfg(all(feature = "duckdb", feature = "valkey"))]
pub(super) fn run(request: &ActivityWorkerLoopRunRequest<'_>) -> io::Result<ControlCliOutput> {
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
    runtime.block_on(worker_loop_with_hot_state(
        &ledger,
        &store,
        ActivityWorkerLoopStoreRequest {
            worker_id: request.worker_id,
            task_queue: request.task_queue,
            now_ms: request.now_ms,
            now_step_ms: request.now_step_ms,
            lease_ttl_ms: request.lease_ttl_ms,
            heartbeat_ttl_ms: request.heartbeat_ttl_ms,
            poll_limit: request.poll_limit,
            empty_limit: request.empty_limit,
            executor: request.executor,
            outcome: request.outcome,
            settled_at_ms: request.settled_at_ms,
            settled_step_ms: request.settled_step_ms,
            output_hash: request.output_hash,
            output_artifact_dir: request.output_artifact_dir,
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
pub(super) fn run(request: &ActivityWorkerLoopRunRequest<'_>) -> io::Result<ControlCliOutput> {
    let _ = (
        request.ledger_path,
        request.valkey_url,
        request.namespace,
        request.worker_id,
        request.task_queue,
        request.now_ms,
        request.now_step_ms,
        request.lease_ttl_ms,
        request.heartbeat_ttl_ms,
        request.poll_limit,
        request.empty_limit,
        request.executor,
        request.outcome,
        request.settled_at_ms,
        request.settled_step_ms,
        request.output_hash,
        request.output_artifact_dir,
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
        "`control activity-worker-loop` requires the `duckdb` and `valkey` features",
    ))
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
pub(crate) async fn worker_loop_with_hot_state<L, H>(
    ledger: &L,
    hot_state: &H,
    request: ActivityWorkerLoopStoreRequest<'_>,
) -> io::Result<ControlCliOutput>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
    H: xiuxian_qianji_control::HotStateStore + ?Sized,
{
    let output = worker_loop_output_with_hot_state(ledger, hot_state, request).await?;
    render_output(&output, request.json)
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
pub(crate) async fn worker_loop_output_with_hot_state<L, H>(
    ledger: &L,
    hot_state: &H,
    request: ActivityWorkerLoopStoreRequest<'_>,
) -> io::Result<ActivityWorkerLoopOutput>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
    H: xiuxian_qianji_control::HotStateStore + ?Sized,
{
    let mut iterations = Vec::new();
    let mut processed = 0;
    let mut empty_polls = 0;
    let mut empty_streak = 0;
    let mut released = 0;
    let mut heartbeats = 0;
    let mut stopped_reason = ActivityWorkerLoopStopReason::PollLimit;

    for poll_index in 0..request.poll_limit {
        let now_ms = stepped_ms(request.now_ms, request.now_step_ms, poll_index)?;
        let settled_at_ms = stepped_ms(request.settled_at_ms, request.settled_step_ms, poll_index)?;
        let output = worker_once_output_with_hot_state(
            ledger,
            hot_state,
            &ActivityWorkerOnceStoreRequest {
                worker_id: request.worker_id,
                task_queue: request.task_queue,
                now_ms,
                lease_ttl_ms: request.lease_ttl_ms,
                heartbeat_ttl_ms: request.heartbeat_ttl_ms,
                executor: request.executor,
                outcome: request.outcome,
                settled_at_ms,
                output_ref_json: None,
                output_hash: request.output_hash,
                output_artifact_path: None,
                output_artifact_dir: request.output_artifact_dir,
                output_artifact_content: None,
                output_artifact_id: None,
                output_artifact_kind: request.output_artifact_kind,
                openai_compatible_base_url: request.openai_compatible_base_url,
                openai_compatible_api_key: request.openai_compatible_api_key,
                openai_compatible_timeout_ms: request.openai_compatible_timeout_ms,
                error_code: request.error_code,
                message: request.message,
                retryable: request.retryable,
                metadata: request.metadata,
                json: true,
            },
        )
        .await?;
        if output.claimed.is_some() {
            processed += 1;
            empty_streak = 0;
        } else {
            empty_polls += 1;
            empty_streak += 1;
        }
        if output.released {
            released += 1;
        }
        if output.heartbeat.is_some() {
            heartbeats += 1;
        }
        iterations.push(ActivityWorkerLoopIteration {
            poll_index,
            now_ms,
            settled_at_ms,
            output,
        });
        if empty_streak >= request.empty_limit {
            stopped_reason = ActivityWorkerLoopStopReason::EmptyLimit;
            break;
        }
    }

    Ok(ActivityWorkerLoopOutput {
        worker_id: request.worker_id.to_string(),
        task_queue: request.task_queue.map(str::to_string),
        executor: request.executor,
        outcome: request.outcome,
        poll_limit: request.poll_limit,
        empty_limit: request.empty_limit,
        iterations,
        processed,
        empty_polls,
        released,
        heartbeats,
        stopped_reason,
    })
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn render_output(output: &ActivityWorkerLoopOutput, json: bool) -> io::Result<ControlCliOutput> {
    let rendered = if json {
        serde_json::to_string_pretty(output).map_err(io::Error::other)?
    } else {
        render_activity_worker_loop_text(output)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn render_activity_worker_loop_text(output: &ActivityWorkerLoopOutput) -> String {
    let task_queue = output.task_queue.as_deref().unwrap_or("<all>");
    format!(
        concat!(
            "# Qianji Control Activity Worker Loop\n\n",
            "- Worker: `{}`\n",
            "- Task queue: `{}`\n",
            "- Executor: `{:?}`\n",
            "- Outcome: `{:?}`\n",
            "- Poll limit: `{}`\n",
            "- Empty limit: `{}`\n",
            "- Iterations: `{}`\n",
            "- Processed: `{}`\n",
            "- Empty polls: `{}`\n",
            "- Released: `{}`\n",
            "- Heartbeats: `{}`\n",
            "- Stopped reason: `{:?}`\n"
        ),
        output.worker_id,
        task_queue,
        output.executor,
        output.outcome,
        output.poll_limit,
        output.empty_limit,
        output.iterations.len(),
        output.processed,
        output.empty_polls,
        output.released,
        output.heartbeats,
        output.stopped_reason
    )
}

fn parse_outcome(value: &str) -> io::Result<ActivitySettleOutcomeArg> {
    match value {
        "complete" => Ok(ActivitySettleOutcomeArg::Complete),
        "fail" => Ok(ActivitySettleOutcomeArg::Fail),
        other => Err(invalid_input(format!(
            "invalid `--outcome` for `control activity-worker-loop`; expected `complete` or `fail`, got `{other}`"
        ))),
    }
}

fn validate_outcome_args(
    executor: ActivityExecutorKindArg,
    outcome: ActivitySettleOutcomeArg,
    args: &ActivityWorkerLoopArgs,
) -> io::Result<()> {
    if executor == ActivityExecutorKindArg::OpenAiCompatibleLlm {
        return validate_openai_compatible_outcome_args(outcome, args);
    }
    if has_openai_compatible_args(args) {
        return Err(invalid_input(
            "`control activity-worker-loop` OpenAI-compatible flags require `--executor openai-compatible-llm`",
        ));
    }
    match outcome {
        ActivitySettleOutcomeArg::Complete => {
            if args.error_code.is_some() || args.message.is_some() || args.retryable.is_some() {
                return Err(invalid_input(
                    "`control activity-worker-loop --outcome complete` cannot be combined with `--error-code`, `--message`, or `--retryable`",
                ));
            }
        }
        ActivitySettleOutcomeArg::Fail => {
            if args.output_hash.is_some() {
                return Err(invalid_input(
                    "`control activity-worker-loop --outcome fail` cannot be combined with `--output-hash`",
                ));
            }
            if args.error_code.is_none() {
                return Err(invalid_input(
                    "missing `--error-code <code>` for `control activity-worker-loop --outcome fail`",
                ));
            }
            if args.message.is_none() {
                return Err(invalid_input(
                    "missing `--message <text>` for `control activity-worker-loop --outcome fail`",
                ));
            }
            if args.retryable.is_none() {
                return Err(invalid_input(
                    "missing `--retryable <true|false>` for `control activity-worker-loop --outcome fail`",
                ));
            }
        }
    }
    Ok(())
}

fn validate_openai_compatible_outcome_args(
    outcome: ActivitySettleOutcomeArg,
    args: &ActivityWorkerLoopArgs,
) -> io::Result<()> {
    if outcome != ActivitySettleOutcomeArg::Complete {
        return Err(invalid_input(
            "`control activity-worker-loop --executor openai-compatible-llm` uses `--outcome complete`; provider failures are recorded by the executor",
        ));
    }
    if args.error_code.is_some() || args.message.is_some() || args.retryable.is_some() {
        return Err(invalid_input(
            "`control activity-worker-loop --executor openai-compatible-llm` cannot be combined with `--error-code`, `--message`, or `--retryable`",
        ));
    }
    if args.output_hash.is_some() {
        return Err(invalid_input(
            "`control activity-worker-loop --executor openai-compatible-llm` derives output hashes from provider artifacts",
        ));
    }
    if args.output_artifact_dir.is_none() {
        return Err(invalid_input(
            "missing `--output-artifact-dir <dir>` for `control activity-worker-loop --executor openai-compatible-llm`",
        ));
    }
    if args
        .openai_compatible_base_url
        .as_deref()
        .is_none_or(|base_url| base_url.trim().is_empty())
    {
        return Err(invalid_input(
            "missing `--openai-compatible-base-url <url>` for `control activity-worker-loop --executor openai-compatible-llm`",
        ));
    }
    Ok(())
}

fn has_openai_compatible_args(args: &ActivityWorkerLoopArgs) -> bool {
    args.output_artifact_dir.is_some()
        || args.output_artifact_kind.is_some()
        || args.openai_compatible_base_url.is_some()
        || args.openai_compatible_api_key.is_some()
        || args.openai_compatible_timeout_ms.is_some()
}

fn parse_u64(field: &'static str, command: &'static str, value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--{field}` for `{command}`; expected u64: {error}"
        ))
    })
}

fn parse_u32(field: &'static str, command: &'static str, value: &str) -> io::Result<u32> {
    value.parse::<u32>().map_err(|error| {
        invalid_input(format!(
            "invalid `--{field}` for `{command}`; expected u32: {error}"
        ))
    })
}

fn parse_u64_flag(args: &[String], index: &mut usize, field: &'static str) -> io::Result<u64> {
    let flag = format!("--{field}");
    parse_u64(
        field,
        "control activity-worker-loop",
        &parse_flag_value(args, index, &flag)?,
    )
}

fn parse_u32_flag(args: &[String], index: &mut usize, field: &'static str) -> io::Result<u32> {
    let flag = format!("--{field}");
    parse_u32(
        field,
        "control activity-worker-loop",
        &parse_flag_value(args, index, &flag)?,
    )
}

fn parse_bool(value: &str) -> io::Result<bool> {
    value.parse::<bool>().map_err(|error| {
        invalid_input(format!(
            "invalid `--retryable` for `control activity-worker-loop`; expected bool: {error}"
        ))
    })
}

fn positive_u32(field: &'static str, value: u32) -> io::Result<u32> {
    if value == 0 {
        return Err(invalid_input(format!(
            "invalid `--{field}` for `control activity-worker-loop`; expected positive u32"
        )));
    }
    Ok(value)
}

fn positive_u64(field: &'static str, value: u64) -> io::Result<u64> {
    if value == 0 {
        return Err(invalid_input(format!(
            "invalid `--{field}` for `control activity-worker-loop`; expected positive u64"
        )));
    }
    Ok(value)
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn stepped_ms(base_ms: u64, step_ms: u64, index: u32) -> io::Result<u64> {
    let offset = step_ms.checked_mul(u64::from(index)).ok_or_else(|| {
        invalid_input("`control activity-worker-loop` timestamp step overflowed u64")
    })?;
    base_ms.checked_add(offset).ok_or_else(|| {
        invalid_input("`control activity-worker-loop` timestamp value overflowed u64")
    })
}

#[cfg(all(feature = "duckdb", feature = "valkey"))]
fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
