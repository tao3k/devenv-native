use std::io;
use std::path::{Path, PathBuf};

use crate::qianji_cli::{invalid_input, parse_flag_value};

use super::types::{ControlCliCommand, ControlCliOutput};

pub(super) fn parse(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut worker_id = None;
    let mut observed_at_ms = None;
    let mut expires_at_ms = None;
    let mut metadata = None;
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
            "--worker-id" => {
                worker_id = Some(parse_flag_value(args, &mut index, "--worker-id")?);
            }
            "--observed-at-ms" => {
                observed_at_ms = Some(parse_ms(
                    "observed-at-ms",
                    "control heartbeat",
                    &parse_flag_value(args, &mut index, "--observed-at-ms")?,
                )?);
            }
            "--expires-at-ms" => {
                expires_at_ms = Some(parse_ms(
                    "expires-at-ms",
                    "control heartbeat",
                    &parse_flag_value(args, &mut index, "--expires-at-ms")?,
                )?);
            }
            "--metadata" => {
                metadata = Some(parse_flag_value(args, &mut index, "--metadata")?);
            }
            "--json" => {
                json = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control heartbeat` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::Heartbeat {
        ledger_path: ledger_path
            .ok_or_else(|| invalid_input("missing `--ledger <path>` for `control heartbeat`"))?,
        run_id: run_id
            .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control heartbeat`"))?,
        worker_id: worker_id
            .ok_or_else(|| invalid_input("missing `--worker-id <id>` for `control heartbeat`"))?,
        observed_at_ms: observed_at_ms.ok_or_else(|| {
            invalid_input("missing `--observed-at-ms <ms>` for `control heartbeat`")
        })?,
        expires_at_ms: expires_at_ms.ok_or_else(|| {
            invalid_input("missing `--expires-at-ms <ms>` for `control heartbeat`")
        })?,
        metadata,
        json,
    })
}

#[cfg(feature = "duckdb")]
pub(super) fn run(
    ledger_path: &Path,
    run_id: &str,
    worker_id: &str,
    observed_at_ms: u64,
    expires_at_ms: u64,
    metadata: Option<&String>,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{
        ControlEvent, ControlEventKind, ControlLedger, DuckDbControlLedger, RunId, WorkerHeartbeat,
        WorkerId,
    };

    if expires_at_ms <= observed_at_ms {
        return Err(invalid_input(
            "`control heartbeat` requires `--expires-at-ms` to be greater than `--observed-at-ms`",
        ));
    }

    let metadata = parse_metadata(metadata)?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let heartbeat = WorkerHeartbeat {
        worker_id: WorkerId::new(worker_id).map_err(|error| control_error(&error))?,
        observed_at_ms,
        expires_at_ms,
        metadata,
    };
    let event = ControlEvent::run(
        run_id,
        observed_at_ms,
        ControlEventKind::WorkerHeartbeatObserved { heartbeat },
    );
    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let record = ledger
        .append_event(event)
        .map_err(|error| control_error(&error))?;
    let rendered = if json {
        serde_json::to_string_pretty(&record).map_err(io::Error::other)?
    } else {
        render_heartbeat_text(&record)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run(
    _ledger_path: &Path,
    _run_id: &str,
    _worker_id: &str,
    _observed_at_ms: u64,
    _expires_at_ms: u64,
    _metadata: Option<&String>,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control heartbeat` requires the `duckdb` feature",
    ))
}

fn parse_ms(flag_name: &str, command_name: &str, value: &str) -> io::Result<u64> {
    value.parse::<u64>().map_err(|error| {
        invalid_input(format!(
            "invalid `--{flag_name}` value `{value}` for `{command_name}`: {error}"
        ))
    })
}

fn parse_metadata(metadata: Option<&String>) -> io::Result<serde_json::Value> {
    match metadata {
        Some(value) => serde_json::from_str(value).map_err(|error| {
            invalid_input(format!(
                "invalid `--metadata` JSON for `control heartbeat`: {error}"
            ))
        }),
        None => Ok(serde_json::Value::Null),
    }
}

#[cfg(feature = "duckdb")]
fn render_heartbeat_text(record: &xiuxian_qianji_control::ControlEventRecord) -> String {
    let xiuxian_qianji_control::ControlEventKind::WorkerHeartbeatObserved { heartbeat } =
        &record.event.kind
    else {
        return "# Qianji Control Heartbeat\n\n- Status: `invalid-event`\n".to_string();
    };
    format!(
        concat!(
            "# Qianji Control Heartbeat\n\n",
            "- Sequence: `{}`\n",
            "- Run: `{}`\n",
            "- Worker: `{}`\n",
            "- Observed at ms: `{}`\n",
            "- Expires at ms: `{}`\n"
        ),
        record.sequence,
        record.event.run_id.as_str(),
        heartbeat.worker_id.as_str(),
        heartbeat.observed_at_ms,
        heartbeat.expires_at_ms
    )
}

#[cfg(feature = "duckdb")]
fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
