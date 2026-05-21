use std::io;
use std::path::{Path, PathBuf};

use crate::qianji_cli::{invalid_input, parse_flag_value};

use super::types::{ControlCliCommand, ControlCliOutput};

pub(super) fn parse_complete(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut parsed = ActivityCompleteArgs::default();
    let mut index = 3;
    while index < args.len() {
        parsed.parse_flag(args, &mut index)?;
        index += 1;
    }
    parsed.into_command()
}

pub(super) fn parse_fail(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut parsed = ActivityFailArgs::default();
    let mut index = 3;
    while index < args.len() {
        parsed.parse_flag(args, &mut index)?;
        index += 1;
    }
    parsed.into_command()
}

#[derive(Default)]
struct ActivityCompleteArgs {
    ledger_path: Option<PathBuf>,
    run_id: Option<String>,
    step_id: Option<String>,
    activity_id: Option<String>,
    completed_at_ms: Option<u64>,
    output_hash: Option<String>,
    metadata: Option<String>,
    json: bool,
}

impl ActivityCompleteArgs {
    fn parse_flag(&mut self, args: &[String], index: &mut usize) -> io::Result<()> {
        match args[*index].as_str() {
            "--ledger" => {
                self.ledger_path = Some(PathBuf::from(parse_flag_value(args, index, "--ledger")?));
            }
            "--run-id" => {
                self.run_id = Some(parse_flag_value(args, index, "--run-id")?);
            }
            "--step-id" => {
                self.step_id = Some(parse_flag_value(args, index, "--step-id")?);
            }
            "--activity-id" => {
                self.activity_id = Some(parse_flag_value(args, index, "--activity-id")?);
            }
            "--completed-at-ms" => {
                self.completed_at_ms = Some(parse_ms(
                    "completed-at-ms",
                    "control activity-complete",
                    &parse_flag_value(args, index, "--completed-at-ms")?,
                )?);
            }
            "--output-hash" => {
                self.output_hash = Some(parse_flag_value(args, index, "--output-hash")?);
            }
            "--metadata" => {
                self.metadata = Some(parse_flag_value(args, index, "--metadata")?);
            }
            "--json" => {
                self.json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control activity-complete` does not accept argument `{other}`"
                )));
            }
        }
        Ok(())
    }

    fn into_command(self) -> io::Result<ControlCliCommand> {
        Ok(ControlCliCommand::ActivityComplete {
            ledger_path: self.ledger_path.ok_or_else(|| {
                invalid_input("missing `--ledger <path>` for `control activity-complete`")
            })?,
            run_id: self.run_id.ok_or_else(|| {
                invalid_input("missing `--run-id <id>` for `control activity-complete`")
            })?,
            step_id: self.step_id,
            activity_id: self.activity_id.ok_or_else(|| {
                invalid_input("missing `--activity-id <id>` for `control activity-complete`")
            })?,
            completed_at_ms: self.completed_at_ms.ok_or_else(|| {
                invalid_input("missing `--completed-at-ms <ms>` for `control activity-complete`")
            })?,
            output_hash: self.output_hash,
            metadata: self.metadata,
            json: self.json,
        })
    }
}

#[derive(Default)]
struct ActivityFailArgs {
    ledger_path: Option<PathBuf>,
    run_id: Option<String>,
    step_id: Option<String>,
    activity_id: Option<String>,
    failed_at_ms: Option<u64>,
    error_code: Option<String>,
    message: Option<String>,
    retryable: Option<bool>,
    attempt: Option<u32>,
    metadata: Option<String>,
    json: bool,
}

impl ActivityFailArgs {
    fn parse_flag(&mut self, args: &[String], index: &mut usize) -> io::Result<()> {
        match args[*index].as_str() {
            "--ledger" => {
                self.ledger_path = Some(PathBuf::from(parse_flag_value(args, index, "--ledger")?));
            }
            "--run-id" => {
                self.run_id = Some(parse_flag_value(args, index, "--run-id")?);
            }
            "--step-id" => {
                self.step_id = Some(parse_flag_value(args, index, "--step-id")?);
            }
            "--activity-id" => {
                self.activity_id = Some(parse_flag_value(args, index, "--activity-id")?);
            }
            "--failed-at-ms" => {
                self.failed_at_ms = Some(parse_ms(
                    "failed-at-ms",
                    "control activity-fail",
                    &parse_flag_value(args, index, "--failed-at-ms")?,
                )?);
            }
            "--error-code" => {
                self.error_code = Some(parse_flag_value(args, index, "--error-code")?);
            }
            "--message" => {
                self.message = Some(parse_flag_value(args, index, "--message")?);
            }
            "--retryable" => {
                self.retryable = Some(parse_bool(
                    "retryable",
                    "control activity-fail",
                    &parse_flag_value(args, index, "--retryable")?,
                )?);
            }
            "--attempt" => {
                self.attempt = Some(parse_u32(
                    "attempt",
                    "control activity-fail",
                    &parse_flag_value(args, index, "--attempt")?,
                )?);
            }
            "--metadata" => {
                self.metadata = Some(parse_flag_value(args, index, "--metadata")?);
            }
            "--json" => {
                self.json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control activity-fail` does not accept argument `{other}`"
                )));
            }
        }
        Ok(())
    }

    fn into_command(self) -> io::Result<ControlCliCommand> {
        Ok(ControlCliCommand::ActivityFail {
            ledger_path: self.ledger_path.ok_or_else(|| {
                invalid_input("missing `--ledger <path>` for `control activity-fail`")
            })?,
            run_id: self.run_id.ok_or_else(|| {
                invalid_input("missing `--run-id <id>` for `control activity-fail`")
            })?,
            step_id: self.step_id,
            activity_id: self.activity_id.ok_or_else(|| {
                invalid_input("missing `--activity-id <id>` for `control activity-fail`")
            })?,
            failed_at_ms: self.failed_at_ms.ok_or_else(|| {
                invalid_input("missing `--failed-at-ms <ms>` for `control activity-fail`")
            })?,
            error_code: self.error_code.ok_or_else(|| {
                invalid_input("missing `--error-code <code>` for `control activity-fail`")
            })?,
            message: self.message.ok_or_else(|| {
                invalid_input("missing `--message <text>` for `control activity-fail`")
            })?,
            retryable: self.retryable.ok_or_else(|| {
                invalid_input("missing `--retryable <true|false>` for `control activity-fail`")
            })?,
            attempt: self.attempt.ok_or_else(|| {
                invalid_input("missing `--attempt <n>` for `control activity-fail`")
            })?,
            metadata: self.metadata,
            json: self.json,
        })
    }
}

#[derive(Clone, Copy)]
pub(super) struct ActivityCompleteRunRequest<'a> {
    pub(super) ledger_path: &'a Path,
    pub(super) run_id: &'a str,
    pub(super) step_id: Option<&'a str>,
    pub(super) activity_id: &'a str,
    pub(super) completed_at_ms: u64,
    pub(super) output_hash: Option<&'a str>,
    pub(super) metadata: Option<&'a str>,
    pub(super) json: bool,
}

#[derive(Clone, Copy)]
pub(super) struct ActivityFailRunRequest<'a> {
    pub(super) ledger_path: &'a Path,
    pub(super) run_id: &'a str,
    pub(super) step_id: Option<&'a str>,
    pub(super) activity_id: &'a str,
    pub(super) failed_at_ms: u64,
    pub(super) error_code: &'a str,
    pub(super) message: &'a str,
    pub(super) retryable: bool,
    pub(super) attempt: u32,
    pub(super) metadata: Option<&'a str>,
    pub(super) json: bool,
}

#[cfg(feature = "duckdb")]
pub(super) fn run_complete(
    request: ActivityCompleteRunRequest<'_>,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{
        ActivityCompletedJournalRecord, ActivityId, ActivityResult, DuckDbControlLedger,
        record_activity_completed_idempotent,
    };

    let scope = activity_scope(request.run_id, request.step_id)?;
    let result = ActivityResult {
        output_ref: None,
        output_hash: request.output_hash.map(str::to_owned),
        metadata: parse_metadata(request.metadata, "control activity-complete")?,
    };
    let complete_record = ActivityCompletedJournalRecord::new(
        scope,
        request.completed_at_ms,
        ActivityId::new(request.activity_id).map_err(|error| control_error(&error))?,
        result,
    );
    let ledger =
        DuckDbControlLedger::open(request.ledger_path).map_err(|error| control_error(&error))?;
    let outcome = record_activity_completed_idempotent(&ledger, complete_record)
        .map_err(|error| control_error(&error))?;
    let rendered = if request.json {
        serde_json::to_string_pretty(&outcome).map_err(io::Error::other)?
    } else {
        render_activity_complete_text(&outcome)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_complete(
    request: ActivityCompleteRunRequest<'_>,
) -> io::Result<ControlCliOutput> {
    let _ = (
        request.ledger_path,
        request.run_id,
        request.step_id,
        request.activity_id,
        request.completed_at_ms,
        request.output_hash,
        request.metadata,
        request.json,
    );
    Err(invalid_input(
        "`control activity-complete` requires the `duckdb` feature",
    ))
}

#[cfg(feature = "duckdb")]
pub(super) fn run_fail(request: ActivityFailRunRequest<'_>) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{
        ActivityFailedJournalRecord, ActivityFailure, ActivityId, DuckDbControlLedger, ErrorCode,
        record_activity_failed_idempotent,
    };

    let scope = activity_scope(request.run_id, request.step_id)?;
    let failure = ActivityFailure {
        error_code: ErrorCode::new(request.error_code).map_err(|error| control_error(&error))?,
        message: request.message.to_owned(),
        retryable: request.retryable,
        attempt: request.attempt,
        metadata: parse_metadata(request.metadata, "control activity-fail")?,
    };
    let fail_record = ActivityFailedJournalRecord::new(
        scope,
        request.failed_at_ms,
        ActivityId::new(request.activity_id).map_err(|error| control_error(&error))?,
        failure,
    );
    let ledger =
        DuckDbControlLedger::open(request.ledger_path).map_err(|error| control_error(&error))?;
    let outcome = record_activity_failed_idempotent(&ledger, fail_record)
        .map_err(|error| control_error(&error))?;
    let rendered = if request.json {
        serde_json::to_string_pretty(&outcome).map_err(io::Error::other)?
    } else {
        render_activity_fail_text(&outcome)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_fail(request: ActivityFailRunRequest<'_>) -> io::Result<ControlCliOutput> {
    let _ = (
        request.ledger_path,
        request.run_id,
        request.step_id,
        request.activity_id,
        request.failed_at_ms,
        request.error_code,
        request.message,
        request.retryable,
        request.attempt,
        request.metadata,
        request.json,
    );
    Err(invalid_input(
        "`control activity-fail` requires the `duckdb` feature",
    ))
}

fn parse_ms(flag_name: &str, command_name: &str, value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--{flag_name}` value `{value}` for `{command_name}`: {error}"
        ))
    })
}

fn parse_u32(flag_name: &str, command_name: &str, value: &str) -> io::Result<u32> {
    value.parse::<u32>().map_err(|error| {
        invalid_input(format!(
            "invalid `--{flag_name}` value `{value}` for `{command_name}`: {error}"
        ))
    })
}

fn parse_bool(flag_name: &str, command_name: &str, value: &str) -> io::Result<bool> {
    value.parse::<bool>().map_err(|error| {
        invalid_input(format!(
            "invalid `--{flag_name}` value `{value}` for `{command_name}`: {error}"
        ))
    })
}

fn parse_metadata(metadata: Option<&str>, command_name: &str) -> io::Result<serde_json::Value> {
    match metadata {
        Some(value) => serde_json::from_str(value).map_err(|error| {
            invalid_input(format!(
                "invalid `--metadata` JSON for `{command_name}`: {error}"
            ))
        }),
        None => Ok(serde_json::Value::Null),
    }
}

#[cfg(feature = "duckdb")]
fn activity_scope(
    run_id: &str,
    step_id: Option<&str>,
) -> io::Result<xiuxian_qianji_control::ActivityJournalScope> {
    use xiuxian_qianji_control::{ActivityJournalScope, RunId, StepId};

    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    match step_id {
        Some(step_id) => Ok(ActivityJournalScope::step(
            run_id,
            StepId::new(step_id).map_err(|error| control_error(&error))?,
        )),
        None => Ok(ActivityJournalScope::run(run_id)),
    }
}

#[cfg(feature = "duckdb")]
fn render_activity_complete_text(
    outcome: &xiuxian_qianji_control::ActivityJournalWriteOutcome,
) -> String {
    let record = &outcome.record;
    let xiuxian_qianji_control::ControlEventKind::ActivityCompleted {
        activity_id,
        result,
    } = &record.event.kind
    else {
        return "# Qianji Control Activity Complete\n\n- Status: `invalid-event`\n".to_string();
    };
    format!(
        concat!(
            "# Qianji Control Activity Complete\n\n",
            "- Write status: `{:?}`\n",
            "- Event sequence: `{}`\n",
            "- Run: `{}`\n",
            "- Scope: `{}`\n",
            "- Activity: `{}`\n",
            "- Completed at ms: `{}`\n",
            "- Output hash: `{}`\n"
        ),
        outcome.status,
        record.sequence,
        record.event.run_id.as_str(),
        render_scope(record),
        activity_id.as_str(),
        record.event.occurred_at_ms,
        result.output_hash.as_deref().unwrap_or("<none>")
    )
}

#[cfg(feature = "duckdb")]
fn render_activity_fail_text(
    outcome: &xiuxian_qianji_control::ActivityJournalWriteOutcome,
) -> String {
    let record = &outcome.record;
    let xiuxian_qianji_control::ControlEventKind::ActivityFailed {
        activity_id,
        failure,
    } = &record.event.kind
    else {
        return "# Qianji Control Activity Fail\n\n- Status: `invalid-event`\n".to_string();
    };
    format!(
        concat!(
            "# Qianji Control Activity Fail\n\n",
            "- Write status: `{:?}`\n",
            "- Event sequence: `{}`\n",
            "- Run: `{}`\n",
            "- Scope: `{}`\n",
            "- Activity: `{}`\n",
            "- Failed at ms: `{}`\n",
            "- Error code: `{}`\n",
            "- Retryable: `{}`\n",
            "- Attempt: `{}`\n"
        ),
        outcome.status,
        record.sequence,
        record.event.run_id.as_str(),
        render_scope(record),
        activity_id.as_str(),
        record.event.occurred_at_ms,
        failure.error_code.as_str(),
        failure.retryable,
        failure.attempt
    )
}

#[cfg(feature = "duckdb")]
fn render_scope(record: &xiuxian_qianji_control::ControlEventRecord) -> &str {
    record
        .event
        .step_id
        .as_ref()
        .map_or("run", |step_id| step_id.as_str())
}

#[cfg(feature = "duckdb")]
fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
