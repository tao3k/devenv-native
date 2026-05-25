//! Org Babel eval-contract CLI argument DTOs.

use std::path::PathBuf;

use clap::{Args, Subcommand};

/// Org Babel eval-contract subcommands.
#[derive(Subcommand, Debug)]
pub enum OrgizeEvalCommand {
    /// Render the parser-owned eval contract for one named source block.
    Plan(OrgizeEvalPlanArgs),
    /// Render or apply host-supplied output as an Org `#+RESULTS:` patch.
    Patch(OrgizeEvalPatchArgs),
}

/// CLI arguments for an Org Babel eval plan.
#[derive(Args, Debug)]
pub struct OrgizeEvalPlanArgs {
    /// Render machine-readable JSON instead of compact text.
    #[arg(long = "json")]
    pub json: bool,

    /// Named `#+NAME:` source block to resolve.
    #[arg(value_name = "NAME")]
    pub name: String,

    /// Org file containing the named source block.
    #[arg(value_name = "PATH")]
    pub path: PathBuf,
}

/// CLI arguments for an Org Babel eval result patch.
#[derive(Args, Debug)]
pub struct OrgizeEvalPatchArgs {
    /// Render machine-readable JSON instead of compact text.
    #[arg(long = "json")]
    pub json: bool,

    /// Write the rendered `#+RESULTS:` patch to the Org file.
    #[arg(long = "write")]
    pub write: bool,

    /// Host-supplied stdout text.
    #[arg(long = "stdout", value_name = "TEXT", conflicts_with = "stdout_file")]
    pub stdout: Option<String>,

    /// Read host-supplied stdout text from a file.
    #[arg(long = "stdout-file", value_name = "PATH")]
    pub stdout_file: Option<PathBuf>,

    /// Host-supplied stderr text.
    #[arg(long = "stderr", value_name = "TEXT", conflicts_with = "stderr_file")]
    pub stderr: Option<String>,

    /// Read host-supplied stderr text from a file.
    #[arg(long = "stderr-file", value_name = "PATH")]
    pub stderr_file: Option<PathBuf>,

    /// Host-supplied process exit code.
    #[arg(long = "exit-code", value_name = "CODE")]
    pub exit_code: Option<i32>,

    /// Named `#+NAME:` source block to resolve.
    #[arg(value_name = "NAME")]
    pub name: String,

    /// Org file containing the named source block.
    #[arg(value_name = "PATH")]
    pub path: PathBuf,
}
