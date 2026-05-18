use std::io;

use crate::qianji_cli::invalid_input;

use super::render::{
    render_control_history_json, render_control_history_text, render_recovery_snapshot_json,
    render_recovery_snapshot_text,
};
use super::types::{ControlCliCommand, ControlCliOutput};

pub(super) fn run_control_command_impl(
    command: &ControlCliCommand,
) -> io::Result<ControlCliOutput> {
    match command {
        ControlCliCommand::History {
            ledger_path,
            run_id,
            json,
        } => run_history_command(ledger_path, run_id, *json),
        ControlCliCommand::RecoverySnapshot {
            ledger_path,
            run_id,
            now_ms,
            json,
        } => run_recovery_snapshot_command(ledger_path, run_id, *now_ms, *json),
    }
}

#[cfg(feature = "duckdb")]
fn run_history_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, RunId};

    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let records = ledger
        .load_events(&run_id)
        .map_err(|error| control_error(&error))?;
    let rendered = if json {
        render_control_history_json(&records)?
    } else {
        render_control_history_text(&run_id, &records)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
fn run_history_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control history` requires the `duckdb` feature",
    ))
}

#[cfg(feature = "duckdb")]
fn run_recovery_snapshot_command(
    ledger_path: &std::path::Path,
    run_id: &str,
    now_ms: u64,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, RunId};

    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let snapshot = ledger
        .load_recovery_snapshot(&run_id, now_ms)
        .map_err(|error| control_error(&error))?;
    let rendered = if json {
        render_recovery_snapshot_json(&snapshot)?
    } else {
        render_recovery_snapshot_text(&snapshot)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(not(feature = "duckdb"))]
fn run_recovery_snapshot_command(
    _ledger_path: &std::path::Path,
    _run_id: &str,
    _now_ms: u64,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control recovery-snapshot` requires the `duckdb` feature",
    ))
}

fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}
