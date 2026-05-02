//! HMAS command execution.

use crate::bin_support::wendao::helpers::emit;
use crate::bin_support::wendao::types::{Cli, Command, HmasCommand};
use crate::validate_blackboard_file;
use anyhow::Result;

pub(super) fn handle(cli: &Cli) -> Result<()> {
    let Command::Hmas { command } = &cli.command else {
        unreachable!("hmas handler must be called with hmas command");
    };

    match command {
        HmasCommand::Validate { file } => {
            let report = validate_blackboard_file(file).map_err(anyhow::Error::msg)?;
            emit(&report, cli.output_or_json())
        }
    }
}
