use std::io;
use std::path::{Path, PathBuf};

use crate::qianji_cli::{invalid_input, parse_flag_value};

use super::activity_args::ActivitySettleOutcomeArg;
use super::activity_executor::ActivityExecutorKindArg;
#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
use super::activity_executor::{
    ActivityExecutionRequest, ActivityExecutorOutcome, ActivityExecutorRegistry,
};
use super::types::{ControlCliCommand, ControlCliOutput};

pub(super) fn parse(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut parsed = ActivityWorkerOnceArgs::default();
    let mut index = 3;
    while index < args.len() {
        parsed.parse_flag(args, &mut index)?;
        index += 1;
    }
    parsed.into_command()
}

#[derive(Default)]
struct ActivityWorkerOnceArgs {
    ledger_path: Option<PathBuf>,
    valkey_url: Option<String>,
    namespace: Option<String>,
    worker_id: Option<String>,
    task_queue: Option<String>,
    now_ms: Option<u64>,
    lease_ttl_ms: Option<u64>,
    executor: Option<ActivityExecutorKindArg>,
    outcome: Option<ActivitySettleOutcomeArg>,
    settled_at_ms: Option<u64>,
    output_hash: Option<String>,
    error_code: Option<String>,
    message: Option<String>,
    retryable: Option<bool>,
    metadata: Option<String>,
    json: bool,
}

impl ActivityWorkerOnceArgs {
    fn parse_flag(&mut self, args: &[String], index: &mut usize) -> io::Result<()> {
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
            "--now-ms" => {
                self.now_ms = Some(parse_u64(
                    "now-ms",
                    "control activity-worker-once",
                    &parse_flag_value(args, index, "--now-ms")?,
                )?);
            }
            "--lease-ttl-ms" => {
                self.lease_ttl_ms = Some(parse_u64(
                    "lease-ttl-ms",
                    "control activity-worker-once",
                    &parse_flag_value(args, index, "--lease-ttl-ms")?,
                )?);
            }
            "--executor" => {
                self.executor = Some(super::activity_executor::parse_executor(
                    &parse_flag_value(args, index, "--executor")?,
                )?);
            }
            "--outcome" => {
                self.outcome = Some(parse_outcome(&parse_flag_value(args, index, "--outcome")?)?);
            }
            "--settled-at-ms" => {
                self.settled_at_ms = Some(parse_u64(
                    "settled-at-ms",
                    "control activity-worker-once",
                    &parse_flag_value(args, index, "--settled-at-ms")?,
                )?);
            }
            "--output-hash" => {
                self.output_hash = Some(parse_flag_value(args, index, "--output-hash")?);
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
                    "`control activity-worker-once` does not accept argument `{other}`"
                )));
            }
        }
        Ok(())
    }

    fn into_command(self) -> io::Result<ControlCliCommand> {
        let executor = self.executor.ok_or_else(|| {
            invalid_input("missing `--executor fixture` for `control activity-worker-once`")
        })?;
        let outcome = self.outcome.ok_or_else(|| {
            invalid_input("missing `--outcome <complete|fail>` for `control activity-worker-once`")
        })?;
        validate_outcome_args(outcome, &self)?;
        Ok(ControlCliCommand::ActivityWorkerOnce {
            ledger_path: self.ledger_path.ok_or_else(|| {
                invalid_input("missing `--ledger <path>` for `control activity-worker-once`")
            })?,
            valkey_url: self.valkey_url.ok_or_else(|| {
                invalid_input("missing `--valkey-url <url>` for `control activity-worker-once`")
            })?,
            namespace: self.namespace,
            worker_id: self.worker_id.ok_or_else(|| {
                invalid_input("missing `--worker-id <id>` for `control activity-worker-once`")
            })?,
            task_queue: self.task_queue,
            now_ms: self.now_ms.ok_or_else(|| {
                invalid_input("missing `--now-ms <ms>` for `control activity-worker-once`")
            })?,
            lease_ttl_ms: self.lease_ttl_ms.ok_or_else(|| {
                invalid_input("missing `--lease-ttl-ms <ms>` for `control activity-worker-once`")
            })?,
            executor,
            outcome,
            settled_at_ms: self.settled_at_ms.ok_or_else(|| {
                invalid_input("missing `--settled-at-ms <ms>` for `control activity-worker-once`")
            })?,
            output_hash: self.output_hash,
            error_code: self.error_code,
            message: self.message,
            retryable: self.retryable,
            metadata: self.metadata,
            json: self.json,
        })
    }
}

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
    pub(super) output_hash: Option<&'a str>,
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
    pub(crate) output_hash: Option<&'a str>,
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
pub(super) fn run(request: ActivityWorkerOnceRunRequest<'_>) -> io::Result<ControlCliOutput> {
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
        ActivityWorkerOnceStoreRequest {
            worker_id: request.worker_id,
            task_queue: request.task_queue,
            now_ms: request.now_ms,
            lease_ttl_ms: request.lease_ttl_ms,
            heartbeat_ttl_ms: None,
            executor: request.executor,
            outcome: request.outcome,
            settled_at_ms: request.settled_at_ms,
            output_hash: request.output_hash,
            error_code: request.error_code,
            message: request.message,
            retryable: request.retryable,
            metadata: request.metadata,
            json: request.json,
        },
    ))
}

#[cfg(not(all(feature = "duckdb", feature = "valkey")))]
pub(super) fn run(request: ActivityWorkerOnceRunRequest<'_>) -> io::Result<ControlCliOutput> {
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
        request.output_hash,
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
    request: ActivityWorkerOnceStoreRequest<'_>,
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
    request: ActivityWorkerOnceStoreRequest<'_>,
) -> io::Result<ActivityWorkerOnceOutput>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
    H: xiuxian_qianji_control::HotStateStore + ?Sized,
{
    use xiuxian_qianji_control::{
        TaskQueue, WorkerActivityStartRecord, WorkerId, WorkerRef,
        record_worker_activity_started_idempotent,
    };

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
        .claim_activity_task(
            worker,
            task_queue.as_ref(),
            request.now_ms,
            request.lease_ttl_ms,
        )
        .await
        .map_err(|error| control_error(&error))?;
    let Some(claimed_task) = claimed.clone() else {
        let output = ActivityWorkerOnceOutput {
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
        };
        return Ok(output);
    };
    let executor_registry = ActivityExecutorRegistry::fixture_only();
    let executor_contract = executor_registry
        .validate_task(request.executor, Some(&claimed_task.activity_task.task))?;
    let start = record_worker_activity_started_idempotent(
        ledger,
        WorkerActivityStartRecord::new(
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
    let executor_outcome = executor_registry.execute(ActivityExecutionRequest {
        task: Some(&claimed_task.activity_task.task),
        executor: request.executor,
        outcome: request.outcome,
        output_hash: request.output_hash,
        error_code: request.error_code,
        message: request.message,
        retryable: request.retryable,
        metadata: request.metadata,
    })?;
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
        claimed,
        start: Some(start),
        heartbeat,
        terminal: Some(terminal),
        released,
    };
    Ok(output)
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

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn heartbeat_metadata(
    claimed_task: &xiuxian_qianji_control::HotStateLeasedActivityTask,
) -> serde_json::Value {
    serde_json::json!({
        "source": "qianji-control-activity-worker",
        "activity_id": claimed_task.activity_task.task.activity_id.as_str(),
        "task_queue": claimed_task.activity_task.task.task_queue.as_str(),
        "next_attempt": claimed_task.activity_task.task.next_attempt,
    })
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

fn parse_outcome(value: &str) -> io::Result<ActivitySettleOutcomeArg> {
    match value {
        "complete" => Ok(ActivitySettleOutcomeArg::Complete),
        "fail" => Ok(ActivitySettleOutcomeArg::Fail),
        other => Err(invalid_input(format!(
            "invalid `--outcome` for `control activity-worker-once`; expected `complete` or `fail`, got `{other}`"
        ))),
    }
}

fn validate_outcome_args(
    outcome: ActivitySettleOutcomeArg,
    args: &ActivityWorkerOnceArgs,
) -> io::Result<()> {
    match outcome {
        ActivitySettleOutcomeArg::Complete => {
            if args.error_code.is_some() || args.message.is_some() || args.retryable.is_some() {
                return Err(invalid_input(
                    "`control activity-worker-once --outcome complete` cannot be combined with `--error-code`, `--message`, or `--retryable`",
                ));
            }
        }
        ActivitySettleOutcomeArg::Fail => {
            if args.output_hash.is_some() {
                return Err(invalid_input(
                    "`control activity-worker-once --outcome fail` cannot be combined with `--output-hash`",
                ));
            }
            if args.error_code.is_none() {
                return Err(invalid_input(
                    "missing `--error-code <code>` for `control activity-worker-once --outcome fail`",
                ));
            }
            if args.message.is_none() {
                return Err(invalid_input(
                    "missing `--message <text>` for `control activity-worker-once --outcome fail`",
                ));
            }
            if args.retryable.is_none() {
                return Err(invalid_input(
                    "missing `--retryable <true|false>` for `control activity-worker-once --outcome fail`",
                ));
            }
        }
    }
    Ok(())
}

fn parse_u64(field: &'static str, command: &'static str, value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--{field}` for `{command}`; expected u64: {error}"
        ))
    })
}

fn parse_bool(value: &str) -> io::Result<bool> {
    value.parse::<bool>().map_err(|error| {
        invalid_input(format!(
            "invalid `--retryable` for `control activity-worker-once`; expected bool: {error}"
        ))
    })
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
