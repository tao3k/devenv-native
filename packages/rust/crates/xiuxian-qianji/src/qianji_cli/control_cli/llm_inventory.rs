use std::io;
use std::path::{Path, PathBuf};

use crate::qianji_cli::control_cli::render::{
    render_llm_activity_inventory_json, render_llm_activity_inventory_text,
};
use crate::qianji_cli::control_cli::{ControlCliCommand, ControlCliOutput};
use crate::qianji_cli::{invalid_input, parse_flag_value};

pub(super) fn parse(args: &[String]) -> io::Result<ControlCliCommand> {
    let mut ledger_path = None;
    let mut run_id = None;
    let mut require_request_audit = false;
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
            "--require-request-audit" => {
                require_request_audit = true;
            }
            other => {
                return Err(invalid_input(format!(
                    "`control llm-activities` does not accept argument `{other}`"
                )));
            }
        }
        index += 1;
    }

    Ok(ControlCliCommand::LlmActivities {
        ledger_path: ledger_path.ok_or_else(|| {
            invalid_input("missing `--ledger <path>` for `control llm-activities`")
        })?,
        run_id: run_id
            .ok_or_else(|| invalid_input("missing `--run-id <id>` for `control llm-activities`"))?,
        require_request_audit,
        json,
    })
}

#[cfg(feature = "duckdb")]
pub(super) fn run(
    ledger_path: &Path,
    run_id: &str,
    require_request_audit: bool,
    json: bool,
) -> io::Result<ControlCliOutput> {
    use xiuxian_qianji_control::{ControlLedger, DuckDbControlLedger, RunId};

    let ledger = DuckDbControlLedger::open(ledger_path).map_err(|error| control_error(&error))?;
    let run_id = RunId::new(run_id).map_err(|error| control_error(&error))?;
    let projection = ledger
        .load_llm_activity_inventory_projection(&run_id)
        .map_err(|error| control_error(&error))?;
    if require_request_audit && projection.summary.missing_request_audit > 0 {
        return Err(invalid_input(format!(
            "`control llm-activities --require-request-audit` found {} LLM activity row(s) without request audit metadata",
            projection.summary.missing_request_audit
        )));
    }
    let rendered = if json {
        render_llm_activity_inventory_json(&projection)?
    } else {
        render_llm_activity_inventory_text(&projection)
    };
    Ok(ControlCliOutput { rendered })
}

#[cfg(feature = "duckdb")]
fn control_error(error: &xiuxian_qianji_control::ControlError) -> io::Error {
    invalid_input(format!("{error}"))
}

#[cfg(not(feature = "duckdb"))]
pub(super) fn run(
    _ledger_path: &Path,
    _run_id: &str,
    _require_request_audit: bool,
    _json: bool,
) -> io::Result<ControlCliOutput> {
    Err(invalid_input(
        "`control llm-activities` requires the `duckdb` feature",
    ))
}
