use std::io;
use std::path::{Path, PathBuf};

use crate::qianji_cli::{invalid_input, parse_flag_value};

use super::types::{ControlCliCommand, ControlCliOutput};

pub(super) fn parse(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut parsed = ActivityStartArgs::default();
    let mut index = 3;
    while index < args.len() {
        parsed.parse_flag(args, &mut index)?;
        index += 1;
    }
    parsed.into_command()
}

#[derive(Default)]
struct ActivityStartArgs {
    ledger_path: Option<PathBuf>,
    worker_task_json: Option<String>,
    run_id: Option<String>,
    step_id: Option<String>,
    activity_id: Option<String>,
    worker_id: Option<String>,
    started_at_ms: Option<u64>,
    attempt: Option<u32>,
    json: bool,
}

impl ActivityStartArgs {
    fn parse_flag(&mut self, args: &[String], index: &mut usize) -> io::Result<()> {
        match args[*index].as_str() {
            "--ledger" => {
                self.ledger_path = Some(PathBuf::from(parse_flag_value(args, index, "--ledger")?));
            }
            "--worker-task-json" => {
                self.worker_task_json = Some(parse_flag_value(args, index, "--worker-task-json")?);
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
            "--worker-id" => {
                self.worker_id = Some(parse_flag_value(args, index, "--worker-id")?);
            }
            "--started-at-ms" => {
                self.started_at_ms = Some(parse_started_at_ms(&parse_flag_value(
                    args,
                    index,
                    "--started-at-ms",
                )?)?);
            }
            "--attempt" => {
                self.attempt = Some(parse_attempt(&parse_flag_value(args, index, "--attempt")?)?);
            }
            "--json" => {
                self.json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control activity-start` does not accept argument `{other}`"
                )));
            }
        }
        Ok(())
    }

    fn into_command(self) -> io::Result<ControlCliCommand> {
        let ledger_path = self.ledger_path.ok_or_else(|| {
            invalid_input("missing `--ledger <path>` for `control activity-start`")
        })?;
        if let Some(worker_task_json) = self.worker_task_json {
            reject_worker_task_conflict(
                "control activity-start",
                self.run_id.as_ref(),
                self.step_id.as_ref(),
                self.activity_id.as_ref(),
                self.attempt,
            )?;
            return Ok(ControlCliCommand::ActivityStartWorkerTask {
                ledger_path,
                worker_task_json,
                worker_id: self.worker_id.ok_or_else(|| {
                    invalid_input("missing `--worker-id <id>` for `control activity-start`")
                })?,
                started_at_ms: self.started_at_ms.ok_or_else(|| {
                    invalid_input("missing `--started-at-ms <ms>` for `control activity-start`")
                })?,
                json: self.json,
            });
        }
        Ok(ControlCliCommand::ActivityStart {
            ledger_path,
            run_id: self.run_id.ok_or_else(|| {
                invalid_input("missing `--run-id <id>` for `control activity-start`")
            })?,
            step_id: self.step_id,
            activity_id: self.activity_id.ok_or_else(|| {
                invalid_input("missing `--activity-id <id>` for `control activity-start`")
            })?,
            worker_id: self.worker_id.ok_or_else(|| {
                invalid_input("missing `--worker-id <id>` for `control activity-start`")
            })?,
            started_at_ms: self.started_at_ms.ok_or_else(|| {
                invalid_input("missing `--started-at-ms <ms>` for `control activity-start`")
            })?,
            attempt: self.attempt.ok_or_else(|| {
                invalid_input("missing `--attempt <n>` for `control activity-start`")
            })?,
            json: self.json,
        })
    }
}

#[derive(Clone, Copy)]
pub(super) struct ActivityStartRunRequest<'a> {
    pub(super) ledger_path: &'a Path,
    pub(super) run_id: &'a str,
    pub(super) step_id: Option<&'a str>,
    pub(super) activity_id: &'a str,
    pub(super) worker_id: &'a str,
    pub(super) started_at_ms: u64,
    pub(super) attempt: u32,
    pub(super) json: bool,
}

#[derive(Clone, Copy)]
pub(super) struct WorkerActivityStartRunRequest<'a> {
    pub(super) ledger_path: &'a Path,
    pub(super) worker_task_json: &'a str,
    pub(super) worker_id: &'a str,
    pub(super) started_at_ms: u64,
    pub(super) json: bool,
}

#[cfg(feature = "duckdb")]
pub(super) fn run(request: ActivityStartRunRequest<'_>) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{
        ActivityId, ActivityJournalScope, ActivityStartedJournalRecord, DuckDbControlLedger, RunId,
        StepId, WorkerId, record_activity_started_idempotent,
    };

    let run_id = RunId::new(request.run_id).map_err(|error| control_error(&error))?;
    let scope = match request.step_id {
        Some(step_id) => ActivityJournalScope::step(
            run_id,
            StepId::new(step_id).map_err(|error| control_error(&error))?,
        ),
        None => ActivityJournalScope::run(run_id),
    };
    let start_record = ActivityStartedJournalRecord::new(
        scope,
        request.started_at_ms,
        ActivityId::new(request.activity_id).map_err(|error| control_error(&error))?,
        request.attempt,
    )
    .with_worker_id(WorkerId::new(request.worker_id).map_err(|error| control_error(&error))?);
    let ledger =
        DuckDbControlLedger::open(request.ledger_path).map_err(|error| control_error(&error))?;
    let outcome = record_activity_started_idempotent(&ledger, start_record)
        .map_err(|error| control_error(&error))?;
    let rendered = if request.json {
        serde_json::to_string_pretty(&outcome).map_err(io::Error::other)?
    } else {
        render_activity_start_text(&outcome)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(feature = "duckdb")]
pub(super) fn run_worker_task(
    request: WorkerActivityStartRunRequest<'_>,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{
        DuckDbControlLedger, WorkerActivityStartRecord, WorkerId,
        record_worker_activity_started_idempotent,
    };

    let worker_task = parse_worker_task(request.worker_task_json)?;
    let start_record = WorkerActivityStartRecord::new(
        worker_task,
        WorkerId::new(request.worker_id).map_err(|error| control_error(&error))?,
        request.started_at_ms,
    );
    let ledger =
        DuckDbControlLedger::open(request.ledger_path).map_err(|error| control_error(&error))?;
    let outcome = record_worker_activity_started_idempotent(&ledger, start_record)
        .map_err(|error| control_error(&error))?;
    let rendered = if request.json {
        serde_json::to_string_pretty(&outcome).map_err(io::Error::other)?
    } else {
        render_activity_start_text(&outcome)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run(request: ActivityStartRunRequest<'_>) -> io::Result<ControlCliOutput> {
    let _ = (
        request.ledger_path,
        request.run_id,
        request.step_id,
        request.activity_id,
        request.worker_id,
        request.started_at_ms,
        request.attempt,
        request.json,
    );
    Err(invalid_input(
        "`control activity-start` requires the `duckdb` feature",
    ))
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run_worker_task(
    request: WorkerActivityStartRunRequest<'_>,
) -> io::Result<ControlCliOutput> {
    let _ = (
        request.ledger_path,
        request.worker_task_json,
        request.worker_id,
        request.started_at_ms,
        request.json,
    );
    Err(invalid_input(
        "`control activity-start --worker-task-json` requires the `duckdb` feature",
    ))
}

fn parse_started_at_ms(value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--started-at-ms` value `{value}` for `control activity-start`: {error}"
        ))
    })
}

fn parse_attempt(value: &str) -> io::Result<u32> {
    value.parse::<u32>().map_err(|error| {
        invalid_input(format!(
            "invalid `--attempt` value `{value}` for `control activity-start`: {error}"
        ))
    })
}

fn reject_worker_task_conflict(
    command_name: &str,
    run_id: Option<&String>,
    step_id: Option<&String>,
    activity_id: Option<&String>,
    attempt: Option<u32>,
) -> io::Result<()> {
    if run_id.is_some() || step_id.is_some() || activity_id.is_some() || attempt.is_some() {
        return Err(invalid_input(format!(
            "`{command_name} --worker-task-json` cannot be combined with `--run-id`, `--step-id`, `--activity-id`, or `--attempt`"
        )));
    }
    Ok(())
}

#[cfg(feature = "duckdb")]
fn parse_worker_task(json: &str) -> io::Result<xiuxian_qianji_control::WorkerActivityTask> {
    serde_json::from_str(json).map_err(|error| {
        invalid_input(format!(
            "invalid `--worker-task-json` for `control activity-start`: {error}"
        ))
    })
}

#[cfg(feature = "duckdb")]
fn render_activity_start_text(
    outcome: &xiuxian_qianji_control::ActivityJournalWriteOutcome,
) -> String {
    let record = &outcome.record;
    let xiuxian_qianji_control::ControlEventKind::ActivityStarted {
        activity_id,
        worker_id,
        attempt,
    } = &record.event.kind
    else {
        return "# Qianji Control Activity Start\n\n- Status: `invalid-event`\n".to_string();
    };
    let scope = record
        .event
        .step_id
        .as_ref()
        .map_or("run", |step_id| step_id.as_str());
    let worker = worker_id
        .as_ref()
        .map_or("<none>", |worker| worker.as_str());
    format!(
        concat!(
            "# Qianji Control Activity Start\n\n",
            "- Write status: `{:?}`\n",
            "- Event sequence: `{}`\n",
            "- Run: `{}`\n",
            "- Scope: `{}`\n",
            "- Activity: `{}`\n",
            "- Worker: `{}`\n",
            "- Started at ms: `{}`\n",
            "- Attempt: `{}`\n"
        ),
        outcome.status,
        record.sequence,
        record.event.run_id.as_str(),
        scope,
        activity_id.as_str(),
        worker,
        record.event.occurred_at_ms,
        attempt
    )
}

#[cfg(feature = "duckdb")]
fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
