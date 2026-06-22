use std::io;
use std::path::{Path, PathBuf};

use crate::qianji_cli::{invalid_input, parse_flag_value};

use super::activity_args::ActivitySettleOutcomeArg;
use super::types::{ControlCliCommand, ControlCliOutput};

pub(super) fn parse(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut parsed = ActivitySettleArgs::default();
    let mut index = 3;
    while index < args.len() {
        parsed.parse_flag(args, &mut index)?;
        index += 1;
    }
    parsed.into_command()
}

#[derive(Default)]
struct ActivitySettleArgs {
    ledger_path: Option<PathBuf>,
    valkey_url: Option<String>,
    namespace: Option<String>,
    leased_task_json: Option<String>,
    outcome: Option<ActivitySettleOutcomeArg>,
    settled_at_ms: Option<u64>,
    output_hash: Option<String>,
    error_code: Option<String>,
    message: Option<String>,
    retryable: Option<bool>,
    metadata: Option<String>,
    json: bool,
}

impl ActivitySettleArgs {
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
            "--leased-task-json" => {
                self.leased_task_json = Some(parse_flag_value(args, index, "--leased-task-json")?);
            }
            "--outcome" => {
                self.outcome = Some(parse_outcome(&parse_flag_value(args, index, "--outcome")?)?);
            }
            "--settled-at-ms" => {
                self.settled_at_ms = Some(parse_u64(
                    "settled-at-ms",
                    "control activity-settle",
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
                    "`control activity-settle` does not accept argument `{other}`"
                )));
            }
        }
        Ok(())
    }

    fn into_command(self) -> io::Result<ControlCliCommand> {
        let outcome = self.outcome.ok_or_else(|| {
            invalid_input("missing `--outcome <complete|fail>` for `control activity-settle`")
        })?;
        validate_outcome_args(outcome, &self)?;
        Ok(ControlCliCommand::ActivitySettle {
            ledger_path: self.ledger_path.ok_or_else(|| {
                invalid_input("missing `--ledger <path>` for `control activity-settle`")
            })?,
            valkey_url: self.valkey_url.ok_or_else(|| {
                invalid_input("missing `--valkey-url <url>` for `control activity-settle`")
            })?,
            namespace: self.namespace,
            leased_task_json: self.leased_task_json.ok_or_else(|| {
                invalid_input("missing `--leased-task-json <json>` for `control activity-settle`")
            })?,
            outcome,
            settled_at_ms: self.settled_at_ms.ok_or_else(|| {
                invalid_input("missing `--settled-at-ms <ms>` for `control activity-settle`")
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
pub(super) struct WorkerActivitySettleRunRequest<'a> {
    pub(super) ledger_path: &'a Path,
    pub(super) valkey_url: &'a str,
    pub(super) namespace: Option<&'a str>,
    pub(super) leased_task_json: &'a str,
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
pub(crate) struct WorkerActivitySettleStoreRequest<'a> {
    pub(crate) leased_task_json: &'a str,
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
pub(crate) struct WorkerActivitySettleOutput {
    pub(crate) outcome: ActivitySettleOutcomeArg,
    pub(crate) leased: xiuxian_qianji_control::HotStateLeasedActivityTask,
    pub(crate) journal: xiuxian_qianji_control::ActivityJournalWriteOutcome,
    pub(crate) released: bool,
}

#[cfg(all(feature = "duckdb", feature = "valkey"))]
pub(super) fn run(request: WorkerActivitySettleRunRequest<'_>) -> io::Result<ControlCliOutput> {
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
    runtime.block_on(settle_with_hot_state(
        &ledger,
        &store,
        WorkerActivitySettleStoreRequest {
            leased_task_json: request.leased_task_json,
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
pub(super) fn run(request: WorkerActivitySettleRunRequest<'_>) -> io::Result<ControlCliOutput> {
    let _ = (
        request.ledger_path,
        request.valkey_url,
        request.namespace,
        request.leased_task_json,
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
        "`control activity-settle` requires the `duckdb` and `valkey` features",
    ))
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
pub(crate) async fn settle_with_hot_state<L, H>(
    ledger: &L,
    hot_state: &H,
    request: WorkerActivitySettleStoreRequest<'_>,
) -> io::Result<ControlCliOutput>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
    H: xiuxian_qianji_control::HotStateStore + ?Sized,
{
    let leased = parse_leased_task(request.leased_task_json)?;
    validate_lease_matches_task(&leased)?;
    let journal = match request.outcome {
        ActivitySettleOutcomeArg::Complete => record_complete(ledger, &leased, request)?,
        ActivitySettleOutcomeArg::Fail => record_fail(ledger, &leased, request)?,
    };
    let released = hot_state
        .release_activity_task_lease(&leased.lease)
        .await
        .map_err(|error| control_error(&error))?;
    let output = WorkerActivitySettleOutput {
        outcome: request.outcome,
        leased,
        journal,
        released,
    };
    let rendered = if request.json {
        serde_json::to_string_pretty(&output).map_err(io::Error::other)?
    } else {
        render_activity_settle_text(&output)
    };
    Ok(ControlCliOutput { rendered })
}

fn parse_outcome(value: &str) -> io::Result<ActivitySettleOutcomeArg> {
    match value {
        "complete" => Ok(ActivitySettleOutcomeArg::Complete),
        "fail" => Ok(ActivitySettleOutcomeArg::Fail),
        other => Err(invalid_input(format!(
            "invalid `--outcome` for `control activity-settle`; expected `complete` or `fail`, got `{other}`"
        ))),
    }
}

fn validate_outcome_args(
    outcome: ActivitySettleOutcomeArg,
    args: &ActivitySettleArgs,
) -> io::Result<()> {
    match outcome {
        ActivitySettleOutcomeArg::Complete => {
            if args.error_code.is_some() || args.message.is_some() || args.retryable.is_some() {
                return Err(invalid_input(
                    "`control activity-settle --outcome complete` cannot be combined with `--error-code`, `--message`, or `--retryable`",
                ));
            }
        }
        ActivitySettleOutcomeArg::Fail => {
            if args.output_hash.is_some() {
                return Err(invalid_input(
                    "`control activity-settle --outcome fail` cannot be combined with `--output-hash`",
                ));
            }
            if args.error_code.is_none() {
                return Err(invalid_input(
                    "missing `--error-code <code>` for `control activity-settle --outcome fail`",
                ));
            }
            if args.message.is_none() {
                return Err(invalid_input(
                    "missing `--message <text>` for `control activity-settle --outcome fail`",
                ));
            }
            if args.retryable.is_none() {
                return Err(invalid_input(
                    "missing `--retryable <true|false>` for `control activity-settle --outcome fail`",
                ));
            }
        }
    }
    Ok(())
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn parse_leased_task(
    leased_task_json: &str,
) -> io::Result<xiuxian_qianji_control::HotStateLeasedActivityTask> {
    serde_json::from_str(leased_task_json).map_err(|error| {
        invalid_input(format!(
            "invalid `--leased-task-json` for `control activity-settle`: {error}"
        ))
    })
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn validate_lease_matches_task(
    leased: &xiuxian_qianji_control::HotStateLeasedActivityTask,
) -> io::Result<()> {
    let task = &leased.activity_task.task;
    let lease = &leased.lease;
    if task.run_id != lease.run_id
        || task.step_id != lease.step_id
        || task.activity_id != lease.activity_id
    {
        return Err(invalid_input(
            "`control activity-settle` leased task and lease identity do not match",
        ));
    }
    Ok(())
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn record_complete<L>(
    ledger: &L,
    leased: &xiuxian_qianji_control::HotStateLeasedActivityTask,
    request: WorkerActivitySettleStoreRequest<'_>,
) -> io::Result<xiuxian_qianji_control::ActivityJournalWriteOutcome>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
{
    use xiuxian_qianji_control::{
        ActivityResult, WorkerActivityCompletedRecord, record_worker_activity_completed_idempotent,
    };

    let result = ActivityResult {
        output_ref: None,
        output_hash: request.output_hash.map(str::to_owned),
        metadata: parse_metadata(request.metadata)?,
    };
    let complete_record = WorkerActivityCompletedRecord::new(
        leased.activity_task.task.clone(),
        request.settled_at_ms,
        result,
    );
    record_worker_activity_completed_idempotent(ledger, complete_record)
        .map_err(|error| control_error(&error))
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn record_fail<L>(
    ledger: &L,
    leased: &xiuxian_qianji_control::HotStateLeasedActivityTask,
    request: WorkerActivitySettleStoreRequest<'_>,
) -> io::Result<xiuxian_qianji_control::ActivityJournalWriteOutcome>
where
    L: xiuxian_qianji_control::ControlLedger + ?Sized,
{
    use xiuxian_qianji_control::{
        ErrorCode, WorkerActivityFailedRecord, WorkerActivityFailureInput,
        record_worker_activity_failed_idempotent,
    };

    let error_code = request.error_code.ok_or_else(|| {
        invalid_input("missing `--error-code <code>` for `control activity-settle --outcome fail`")
    })?;
    let message = request.message.ok_or_else(|| {
        invalid_input("missing `--message <text>` for `control activity-settle --outcome fail`")
    })?;
    let retryable = request.retryable.ok_or_else(|| {
        invalid_input(
            "missing `--retryable <true|false>` for `control activity-settle --outcome fail`",
        )
    })?;
    let failure_record = WorkerActivityFailedRecord::new(
        WorkerActivityFailureInput::new(
            leased.activity_task.task.clone(),
            ErrorCode::new(error_code).map_err(|error| control_error(&error))?,
            message,
        )
        .with_failed_at_ms(request.settled_at_ms)
        .with_retryable(retryable),
    )
    .with_metadata(parse_metadata(request.metadata)?);
    record_worker_activity_failed_idempotent(ledger, failure_record)
        .map_err(|error| control_error(&error))
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn parse_metadata(metadata: Option<&str>) -> io::Result<serde_json::Value> {
    match metadata {
        Some(value) => serde_json::from_str(value).map_err(|error| {
            invalid_input(format!(
                "invalid `--metadata` JSON for `control activity-settle`: {error}"
            ))
        }),
        None => Ok(serde_json::Value::Null),
    }
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn render_activity_settle_text(output: &WorkerActivitySettleOutput) -> String {
    format!(
        concat!(
            "# Qianji Control Activity Settle\n\n",
            "- Outcome: `{:?}`\n",
            "- Write status: `{:?}`\n",
            "- Event sequence: `{}`\n",
            "- Released: `{}`\n",
            "- Run: `{}`\n",
            "- Activity: `{}`\n",
            "- Lease: `{}`\n"
        ),
        output.outcome,
        output.journal.status,
        output.journal.record.sequence,
        output.released,
        output.leased.lease.run_id.as_str(),
        output.leased.lease.activity_id.as_str(),
        output.leased.lease.lease_id.as_str()
    )
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
            "invalid `--retryable` for `control activity-settle`; expected bool: {error}"
        ))
    })
}

#[cfg(any(all(feature = "duckdb", feature = "valkey"), test))]
fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
