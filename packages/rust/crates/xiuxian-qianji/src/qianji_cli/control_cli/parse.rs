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
        Some("recovery-snapshot") => parse_recovery_snapshot(args).map(Some),
        Some(other) => Err(invalid_input(format!(
            "unsupported `control` subcommand `{other}`"
        ))),
        None => Err(invalid_input(
            "missing `control` subcommand; expected `recovery-snapshot`",
        )),
    }
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

fn parse_now_ms(value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--now-ms` value `{value}` for `control recovery-snapshot`: {error}"
        ))
    })
}
