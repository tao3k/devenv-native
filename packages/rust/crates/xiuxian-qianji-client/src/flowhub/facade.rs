//! Public `qianji-client flowhub` command facade.

use std::env;
use std::path::Path;

use super::materialize::{load_registry, run_flowhub_plan};
use super::model::{FlowhubCliOutput, FlowhubScenarioRegistry};
use super::parse::{ClientCommand, parse_client_command};
use crate::QianjiClientError;

/// Run the `qianji-client` CLI with process arguments.
///
/// # Errors
/// Returns [`QianjiClientError`] when parsing, materialization, or validation fails.
pub fn run_xiuxian_qianji_client_cli() -> Result<(), QianjiClientError> {
    let args = env::args().collect::<Vec<_>>();
    let output = run_xiuxian_qianji_client_cli_with_args(&args)?;
    println!("{}", output.rendered);
    if output.passed {
        Ok(())
    } else {
        Err(QianjiClientError::message(format!(
            "qianji-client flowhub {:?} failed validation",
            output.action
        )))
    }
}

/// Run the `qianji-client` CLI with explicit arguments.
///
/// # Errors
/// Returns [`QianjiClientError`] when command parsing, materialization, or
/// report construction fails.
pub fn run_xiuxian_qianji_client_cli_with_args(
    args: &[String],
) -> Result<FlowhubCliOutput, QianjiClientError> {
    match parse_client_command(args)? {
        ClientCommand::Flowhub(command) => run_flowhub_plan(command),
    }
}

/// Load the Flowhub Org+BPMN scenario registry for a Flowhub root.
///
/// # Errors
/// Returns [`QianjiClientError`] when the Flowhub root cannot be scanned or
/// hashed.
pub fn load_flowhub_scenario_registry(
    flowhub_root: &Path,
) -> Result<FlowhubScenarioRegistry, QianjiClientError> {
    load_registry(flowhub_root)
}
