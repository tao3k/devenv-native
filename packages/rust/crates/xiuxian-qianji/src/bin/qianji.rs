//! Qianji binary entry seam.
//!
//! Start with `dispatch` for command routing; `bpmn_cli`, `dir_cli`,
//! `contract_feedback_cli`, `emit_cli`, `lint_cli`, and `template_cli` own the concrete
//! subcommands while `manifest_exec` owns manifest execution.

#[path = "qianji/bpmn_cli/mod.rs"]
mod bpmn_cli;
#[path = "qianji/common.rs"]
mod common;
#[path = "qianji/construct_cli.rs"]
mod construct_cli;
#[path = "qianji/contract_feedback_cli/mod.rs"]
mod contract_feedback_cli;
#[path = "qianji/dir_cli/mod.rs"]
mod dir_cli;
#[path = "qianji/dispatch.rs"]
mod dispatch;
#[path = "qianji/emit_cli.rs"]
mod emit_cli;
#[path = "qianji/graph_export.rs"]
mod graph_export;
#[path = "qianji/json_output.rs"]
mod json_output;
#[path = "qianji/lint_cli/mod.rs"]
mod lint_cli;
#[path = "qianji/manifest_exec.rs"]
mod manifest_exec;
#[path = "qianji/template_cli.rs"]
mod template_cli;
#[cfg(test)]
#[path = "qianji/test_exports.rs"]
mod test_exports;
#[path = "qianji/usage.rs"]
mod usage;
#[path = "qianji/workspace.rs"]
mod workspace;

use common::{invalid_input, parse_flag_value, resolve_cli_path};
/// Main entry point for the Qianji execution engine.
///
/// # Errors
/// Returns an error if environment resolution, compilation, or execution fails.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dispatch::run().await
}

#[cfg(test)]
use test_exports::*;

#[cfg(test)]
#[path = "../../tests/unit/bin/qianji/mod.rs"]
mod tests;
