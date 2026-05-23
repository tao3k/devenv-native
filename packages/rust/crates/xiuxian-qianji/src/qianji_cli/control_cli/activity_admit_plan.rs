use std::io;
use std::path::{Path, PathBuf};

use crate::qianji_cli::{invalid_input, parse_flag_value};

use super::types::{ControlCliCommand, ControlCliOutput};

pub(super) fn parse(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut parsed = ActivityAdmitPlanArgs::default();
    let mut index = 3;
    while index < args.len() {
        parsed.parse_flag(args, &mut index)?;
        index += 1;
    }
    parsed.into_command()
}

#[derive(Default)]
struct ActivityAdmitPlanArgs {
    ledger_path: Option<PathBuf>,
    run_id: Option<String>,
    step_id: Option<String>,
    occurred_at_ms: Option<u64>,
    schedule_plan_json_path: Option<PathBuf>,
    json: bool,
}

impl ActivityAdmitPlanArgs {
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
            "--schedule-plan-json" => {
                self.schedule_plan_json_path = Some(PathBuf::from(parse_flag_value(
                    args,
                    index,
                    "--schedule-plan-json",
                )?));
            }
            "--json" => {
                self.json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control activity-admit-plan` does not accept argument `{other}`"
                )));
            }
        }
        Ok(())
    }

    fn into_command(self) -> io::Result<ControlCliCommand> {
        Ok(ControlCliCommand::ActivityAdmitPlan {
            ledger_path: self.ledger_path.ok_or_else(|| {
                invalid_input("missing `--ledger <path>` for `control activity-admit-plan`")
            })?,
            run_id: self.run_id.ok_or_else(|| {
                invalid_input("missing `--run-id <id>` for `control activity-admit-plan`")
            })?,
            step_id: self.step_id,
            occurred_at_ms: self.occurred_at_ms.ok_or_else(|| {
                invalid_input("missing `--occurred-at-ms <ms>` for `control activity-admit-plan`")
            })?,
            schedule_plan_json_path: self.schedule_plan_json_path.ok_or_else(|| {
                invalid_input(
                    "missing `--schedule-plan-json <path>` for `control activity-admit-plan`",
                )
            })?,
            json: self.json,
        })
    }
}

#[derive(Clone, Copy)]
pub(super) struct ActivityAdmitPlanRunRequest<'a> {
    pub(super) ledger_path: &'a Path,
    pub(super) run_id: &'a str,
    pub(super) step_id: Option<&'a str>,
    pub(super) occurred_at_ms: u64,
    pub(super) schedule_plan_json_path: &'a Path,
    pub(super) json: bool,
}

#[cfg(feature = "duckdb")]
pub(super) fn run(request: ActivityAdmitPlanRunRequest<'_>) -> io::Result<ControlCliOutput> {
    use std::fs;

    use xiuxian_qianji_control::{
        ActivitySchedulePlanAdmissionRequest, DuckDbControlLedger, RunId, StepId,
        admit_activity_schedule_plan, parse_activity_schedule_plan_json,
    };

    let content = fs::read_to_string(request.schedule_plan_json_path).map_err(|error| {
        invalid_input(format!(
            "failed to read `--schedule-plan-json` `{}`: {error}",
            request.schedule_plan_json_path.display()
        ))
    })?;
    let items = parse_activity_schedule_plan_json(&content).map_err(|error| {
        invalid_input(format!(
            "invalid `--schedule-plan-json` `{}`: {error}",
            request.schedule_plan_json_path.display()
        ))
    })?;
    let run_id = RunId::new(request.run_id).map_err(|error| control_error(&error))?;
    let admission_request = match request.step_id {
        Some(step_id) => ActivitySchedulePlanAdmissionRequest::step(
            run_id,
            StepId::new(step_id).map_err(|error| control_error(&error))?,
            request.occurred_at_ms,
            items,
        ),
        None => ActivitySchedulePlanAdmissionRequest::run(run_id, request.occurred_at_ms, items),
    };
    let ledger =
        DuckDbControlLedger::open(request.ledger_path).map_err(|error| control_error(&error))?;
    let report = admit_activity_schedule_plan(&ledger, admission_request)
        .map_err(|error| control_error(&error))?;
    let rendered = if request.json {
        serde_json::to_string_pretty(&report).map_err(io::Error::other)?
    } else {
        render_activity_admit_plan_text(&report)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run(request: ActivityAdmitPlanRunRequest<'_>) -> io::Result<ControlCliOutput> {
    let _ = (
        request.ledger_path,
        request.run_id,
        request.step_id,
        request.occurred_at_ms,
        request.schedule_plan_json_path,
        request.json,
    );
    Err(invalid_input(
        "`control activity-admit-plan` requires the `duckdb` feature",
    ))
}

fn parse_occurred_at_ms(value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--occurred-at-ms` value `{value}` for `control activity-admit-plan`: {error}"
        ))
    })
}

#[cfg(feature = "duckdb")]
fn render_activity_admit_plan_text(
    report: &xiuxian_qianji_control::ActivitySchedulePlanAdmissionReport,
) -> String {
    format!(
        "# Qianji Control Activity Schedule Plan Admission\n\n\
         - Run: `{}`\n\
         - Plan rows: `{}`\n\
         - Appended: `{}`\n\
         - Already recorded: `{}`\n",
        report.run_id.as_str(),
        report.plan_item_count,
        report.appended_count,
        report.already_recorded_count
    )
}

#[cfg(feature = "duckdb")]
fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
