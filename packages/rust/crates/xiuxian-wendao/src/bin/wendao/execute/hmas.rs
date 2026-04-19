//! HMAS command execution.

use crate::helpers::emit;
use crate::types::{Cli, Command, HmasCommand};
use anyhow::Result;
use xiuxian_wendao::validate_blackboard_file;

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
