use std::io;
use std::path::{Path, PathBuf};

use crate::qianji_cli::{invalid_input, parse_flag_value};

use super::types::{ControlCliCommand, ControlCliOutput};

pub(super) fn parse(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut parsed = RunCreateArgs::default();
    let mut index = 3;
    while index < args.len() {
        parsed.parse_flag(args, &mut index)?;
        index += 1;
    }
    parsed.into_command()
}

#[derive(Default)]
struct RunCreateArgs {
    ledger_path: Option<PathBuf>,
    run_id: Option<String>,
    occurred_at_ms: Option<u64>,
    intent: Option<String>,
    json: bool,
}

impl RunCreateArgs {
    fn parse_flag(&mut self, args: &[String], index: &mut usize) -> io::Result<()> {
        match args[*index].as_str() {
            "--ledger" => {
                self.ledger_path = Some(PathBuf::from(parse_flag_value(args, index, "--ledger")?));
            }
            "--run-id" => {
                self.run_id = Some(parse_flag_value(args, index, "--run-id")?);
            }
            "--occurred-at-ms" => {
                self.occurred_at_ms = Some(parse_occurred_at_ms(&parse_flag_value(
                    args,
                    index,
                    "--occurred-at-ms",
                )?)?);
            }
            "--intent" => {
                self.intent = Some(parse_flag_value(args, index, "--intent")?);
            }
            "--json" => {
                self.json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control run-create` does not accept argument `{other}`"
                )));
            }
        }
        Ok(())
    }

    fn into_command(self) -> io::Result<ControlCliCommand> {
        Ok(ControlCliCommand::RunCreate {
            ledger_path: self.ledger_path.ok_or_else(|| {
                invalid_input("missing `--ledger <path>` for `control run-create`")
            })?,
            run_id: self
                .run_id
                .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control run-create`"))?,
            occurred_at_ms: self.occurred_at_ms.ok_or_else(|| {
                invalid_input("missing `--occurred-at-ms <ms>` for `control run-create`")
            })?,
            intent: self.intent.ok_or_else(|| {
                invalid_input("missing `--intent <text>` for `control run-create`")
            })?,
            json: self.json,
        })
    }
}

#[derive(Clone, Copy)]
pub(super) struct RunCreateRunRequest<'a> {
    pub(super) ledger_path: &'a Path,
    pub(super) run_id: &'a str,
    pub(super) occurred_at_ms: u64,
    pub(super) intent: &'a str,
    pub(super) json: bool,
}

#[cfg(feature = "duckdb")]
pub(super) fn run(request: RunCreateRunRequest<'_>) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{
        DuckDbControlLedger, RunCreatedJournalRecord, RunId, record_run_created,
    };

    let run_id = RunId::new(request.run_id).map_err(|error| control_error(&error))?;
    let ledger =
        DuckDbControlLedger::open(request.ledger_path).map_err(|error| control_error(&error))?;
    let record = record_run_created(
        &ledger,
        RunCreatedJournalRecord::new(run_id, request.intent, request.occurred_at_ms),
    )
    .map_err(|error| control_error(&error))?;
    let rendered = if request.json {
        serde_json::to_string_pretty(&record).map_err(io::Error::other)?
    } else {
        render_run_create_text(&record)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run(request: RunCreateRunRequest<'_>) -> io::Result<ControlCliOutput> {
    let _ = (
        request.ledger_path,
        request.run_id,
        request.occurred_at_ms,
        request.intent,
        request.json,
    );
    Err(invalid_input(
        "`control run-create` requires the `duckdb` feature",
    ))
}

fn parse_occurred_at_ms(value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--occurred-at-ms` value `{value}` for `control run-create`: {error}"
        ))
    })
}

#[cfg(feature = "duckdb")]
fn render_run_create_text(record: &xiuxian_qianji_control::ControlEventRecord) -> String {
    format!(
        "# Qianji Control Run Create\n\n- Run: `{}`\n- Sequence: `{}`\n",
        record.event.run_id.as_str(),
        record.sequence
    )
}

#[cfg(feature = "duckdb")]
fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
