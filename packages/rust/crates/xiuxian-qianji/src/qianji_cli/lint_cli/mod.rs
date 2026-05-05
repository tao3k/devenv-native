//! Lint subcommand facade.
//!
//! `command` is the canonical entry owner for lint execution.

mod bpmn_json;
mod command;
mod llm;
mod render;
mod types;
mod workflow_plan;

pub(super) use command::handle_lint_command;
pub(crate) use command::parse_lint_command;
#[cfg(test)]
pub(crate) use command::{LintCliCommand, run_lint_command};
