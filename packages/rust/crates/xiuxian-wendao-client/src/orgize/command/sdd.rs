//! Org-native SDD CLI argument DTOs.

use clap::{Args, Subcommand};
use std::path::PathBuf;

/// Org-native SDD subcommands.
#[derive(Subcommand, Debug)]
pub enum OrgizeSddCommand {
    /// Render Org-native SDD status cards.
    Status(OrgizeSddStatusArgs),
    /// Compare SDD parent edges with Org outline nesting.
    GraphDiff(OrgizeSddGraphDiffArgs),
}

/// CLI arguments for Org-native SDD status cards.
#[derive(Args, Debug)]
pub struct OrgizeSddStatusArgs {
    /// Render a machine-readable JSON status payload.
    #[arg(long = "json")]
    pub json: bool,

    /// Render only SDD files that currently have diagnostics.
    #[arg(long = "issues-only")]
    pub issues_only: bool,

    /// Return exit code 1 when SDD diagnostics are present.
    #[arg(long = "fail-on-issues")]
    pub fail_on_issues: bool,

    /// Org files or directories to inspect. When omitted, uses `$PRJ_CACHE_HOME/agent/sdd`.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

/// CLI arguments for Org-native SDD graph diff cards.
#[derive(Args, Debug)]
pub struct OrgizeSddGraphDiffArgs {
    /// Return exit code 1 when graph drift is present.
    #[arg(long = "fail-on-drift")]
    pub fail_on_drift: bool,

    /// Org files or directories to inspect. When omitted, uses `$PRJ_CACHE_HOME/agent/sdd`.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}
