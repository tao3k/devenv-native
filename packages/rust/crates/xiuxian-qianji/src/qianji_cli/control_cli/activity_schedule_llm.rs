use std::io;
use std::path::{Path, PathBuf};

use crate::qianji_cli::{invalid_input, parse_flag_value};

use super::types::{ControlCliCommand, ControlCliOutput};

pub(super) fn parse(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut parsed = ActivityScheduleLlmArgs::default();
    let mut index = 3;
    while index < args.len() {
        parsed.parse_flag(args, &mut index)?;
        index += 1;
    }
    parsed.into_command()
}

#[derive(Default)]
struct ActivityScheduleLlmArgs {
    ledger_path: Option<PathBuf>,
    run_id: Option<String>,
    step_id: Option<String>,
    occurred_at_ms: Option<u64>,
    llm_activity_json: Option<String>,
    json: bool,
}

impl ActivityScheduleLlmArgs {
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
            "--occurred-at-ms" => {
                self.occurred_at_ms = Some(parse_occurred_at_ms(&parse_flag_value(
                    args,
                    index,
                    "--occurred-at-ms",
                )?)?);
            }
            "--llm-activity-json" => {
                self.llm_activity_json =
                    Some(parse_flag_value(args, index, "--llm-activity-json")?);
            }
            "--json" => {
                self.json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control activity-schedule-llm` does not accept argument `{other}`"
                )));
            }
        }
        Ok(())
    }

    fn into_command(self) -> io::Result<ControlCliCommand> {
        Ok(ControlCliCommand::ActivityScheduleLlm {
            ledger_path: self.ledger_path.ok_or_else(|| {
                invalid_input("missing `--ledger <path>` for `control activity-schedule-llm`")
            })?,
            run_id: self.run_id.ok_or_else(|| {
                invalid_input("missing `--run-id <id>` for `control activity-schedule-llm`")
            })?,
            step_id: self.step_id,
            occurred_at_ms: self.occurred_at_ms.ok_or_else(|| {
                invalid_input("missing `--occurred-at-ms <ms>` for `control activity-schedule-llm`")
            })?,
            llm_activity_json: self.llm_activity_json.ok_or_else(|| {
                invalid_input(
                    "missing `--llm-activity-json <json>` for `control activity-schedule-llm`",
                )
            })?,
            json: self.json,
        })
    }
}

#[derive(Clone, Copy)]
pub(super) struct ActivityScheduleLlmRunRequest<'a> {
    pub(super) ledger_path: &'a Path,
    pub(super) run_id: &'a str,
    pub(super) step_id: Option<&'a str>,
    pub(super) occurred_at_ms: u64,
    pub(super) llm_activity_json: &'a str,
    pub(super) json: bool,
}

#[cfg(feature = "duckdb")]
pub(super) fn run(request: ActivityScheduleLlmRunRequest<'_>) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{
        AdmittedLlmActivityScheduleRecord, DuckDbControlLedger, LlmActivityAdmission,
        LlmActivityTask, RunId, StepId, record_admitted_llm_activity_schedule_idempotent,
    };

    let activity: LlmActivityTask =
        serde_json::from_str(request.llm_activity_json).map_err(|error| {
            invalid_input(format!(
                "invalid `--llm-activity-json` for `control activity-schedule-llm`: {error}"
            ))
        })?;
    let admission =
        LlmActivityAdmission::from_activity(activity).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(request.run_id).map_err(|error| control_error(&error))?;
    let schedule_request = match request.step_id {
        Some(step_id) => AdmittedLlmActivityScheduleRecord::step(
            run_id,
            StepId::new(step_id).map_err(|error| control_error(&error))?,
            request.occurred_at_ms,
            admission,
        ),
        None => AdmittedLlmActivityScheduleRecord::run(run_id, request.occurred_at_ms, admission),
    };
    let ledger =
        DuckDbControlLedger::open(request.ledger_path).map_err(|error| control_error(&error))?;
    let outcome = record_admitted_llm_activity_schedule_idempotent(&ledger, schedule_request)
        .map_err(|error| control_error(&error))?;
    let rendered = if request.json {
        serde_json::to_string_pretty(&outcome).map_err(io::Error::other)?
    } else {
        render_activity_schedule_llm_text(&outcome)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run(request: ActivityScheduleLlmRunRequest<'_>) -> io::Result<ControlCliOutput> {
    let _ = (
        request.ledger_path,
        request.run_id,
        request.step_id,
        request.occurred_at_ms,
        request.llm_activity_json,
        request.json,
    );
    Err(invalid_input(
        "`control activity-schedule-llm` requires the `duckdb` feature",
    ))
}

fn parse_occurred_at_ms(value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--occurred-at-ms` value `{value}` for `control activity-schedule-llm`: {error}"
        ))
    })
}

#[cfg(feature = "duckdb")]
fn render_activity_schedule_llm_text(
    outcome: &xiuxian_qianji_control::ActivityJournalWriteOutcome,
) -> String {
    let xiuxian_qianji_control::ControlEventKind::ActivityScheduled { task } =
        &outcome.record.event.kind
    else {
        return "# Qianji Control LLM Activity Schedule\n\n- Status: `<invalid>`\n".to_owned();
    };
    format!(
        "# Qianji Control LLM Activity Schedule\n\n\
         - Status: `{:?}`\n\
         - Activity: `{}`\n\
         - Activity type: `{}`\n\
         - Task queue: `{}`\n\
         - Sequence: `{}`\n",
        outcome.status,
        task.activity_id.as_str(),
        task.activity_type.as_str(),
        task.task_queue.as_str(),
        outcome.record.sequence
    )
}

#[cfg(feature = "duckdb")]
fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
