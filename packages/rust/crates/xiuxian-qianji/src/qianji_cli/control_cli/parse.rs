use std::io;
use std::path::PathBuf;

use crate::qianji_cli::{invalid_input, parse_flag_value};

use super::types::ControlCliCommand;

pub(super) fn parse_control_command_impl(args: &[String]) -> io::Result<Option<ControlCliCommand>> {
    let Some(command_name) = args.get(1).map(String::as_str) else {
        return Ok(None);
    };
    if command_name != "control" {
        return Ok(None);
    }

    match args.get(2).map(String::as_str) {
        Some("activity") => parse_activity(args).map(Some),
        Some("history") => parse_history(args).map(Some),
        Some("recovery-snapshot") => parse_recovery_snapshot(args).map(Some),
        Some("step") => parse_step(args).map(Some),
        Some("view") => parse_view(args).map(Some),
        Some(other) => Err(invalid_input(format!(
            "unsupported `control` subcommand `{other}`"
        ))),
        None => Err(invalid_input(
            "missing `control` subcommand; expected `activity`, `history`, `recovery-snapshot`, `step`, or `view`",
        )),
    }
}

fn parse_activity(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut step_id = None;
    let mut activity_id = None;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--ledger" => {
                ledger_path = Some(PathBuf::from(parse_flag_value(
                    args, &mut index, "--ledger",
                )?));
            }
            "--run-id" => {
                run_id = Some(parse_flag_value(args, &mut index, "--run-id")?);
            }
            "--step-id" => {
                step_id = Some(parse_flag_value(args, &mut index, "--step-id")?);
            }
            "--activity-id" => {
                activity_id = Some(parse_flag_value(args, &mut index, "--activity-id")?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control activity` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::Activity {
        ledger_path: ledger_path
            .ok_or_else(|| invalid_input("missing `--ledger <path>` for `control activity`"))?,
        run_id: run_id
            .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control activity`"))?,
        step_id,
        activity_id: activity_id
            .ok_or_else(|| invalid_input("missing `--activity-id <id>` for `control activity`"))?,
        json,
    })
}

fn parse_history(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--ledger" => {
                ledger_path = Some(PathBuf::from(parse_flag_value(
                    args, &mut index, "--ledger",
                )?));
            }
            "--run-id" => {
                run_id = Some(parse_flag_value(args, &mut index, "--run-id")?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control history` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::History {
        ledger_path: ledger_path
            .ok_or_else(|| invalid_input("missing `--ledger <path>` for `control history`"))?,
        run_id: run_id
            .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control history`"))?,
        json,
    })
}

fn parse_recovery_snapshot(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut now_ms = None;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--ledger" => {
                ledger_path = Some(PathBuf::from(parse_flag_value(
                    args, &mut index, "--ledger",
                )?));
            }
            "--run-id" => {
                run_id = Some(parse_flag_value(args, &mut index, "--run-id")?);
            }
            "--now-ms" => {
                now_ms = Some(parse_now_ms(&parse_flag_value(
                    args, &mut index, "--now-ms",
                )?)?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control recovery-snapshot` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::RecoverySnapshot {
        ledger_path: ledger_path.ok_or_else(|| {
            invalid_input("missing `--ledger <path>` for `control recovery-snapshot`")
        })?,
        run_id: run_id.ok_or_else(|| {
            invalid_input("missing `--run-id <id>` for `control recovery-snapshot`")
        })?,
        now_ms: now_ms.ok_or_else(|| {
            invalid_input("missing `--now-ms <ms>` for `control recovery-snapshot`")
        })?,
        json,
    })
}

fn parse_view(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--ledger" => {
                ledger_path = Some(PathBuf::from(parse_flag_value(
                    args, &mut index, "--ledger",
                )?));
            }
            "--run-id" => {
                run_id = Some(parse_flag_value(args, &mut index, "--run-id")?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control view` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::View {
        ledger_path: ledger_path
            .ok_or_else(|| invalid_input("missing `--ledger <path>` for `control view`"))?,
        run_id: run_id
            .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control view`"))?,
        json,
    })
}

fn parse_step(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut step_id = None;
    let mut json = false;

    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--ledger" => {
                ledger_path = Some(PathBuf::from(parse_flag_value(
                    args, &mut index, "--ledger",
                )?));
            }
            "--run-id" => {
                run_id = Some(parse_flag_value(args, &mut index, "--run-id")?);
            }
            "--step-id" => {
                step_id = Some(parse_flag_value(args, &mut index, "--step-id")?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control step` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::Step {
        ledger_path: ledger_path
            .ok_or_else(|| invalid_input("missing `--ledger <path>` for `control step`"))?,
        run_id: run_id
            .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control step`"))?,
        step_id: step_id
            .ok_or_else(|| invalid_input("missing `--step-id <id>` for `control step`"))?,
        json,
    })
}

fn parse_now_ms(value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--now-ms` value `{value}` for `control recovery-snapshot`: {error}"
        ))
    })
}
