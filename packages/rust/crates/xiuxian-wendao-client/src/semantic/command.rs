//! Command-line argument model for semantic SSOT operations.

use clap::{Args, Subcommand};
use std::path::PathBuf;

/// Semantic-oriented client subcommands.
#[derive(Subcommand, Debug)]
pub enum SemanticCommand {
    /// Run one explicit semantic projection metadata refresh worker pass.
    RefreshProjections(SemanticRefreshProjectionsArgs),
}

/// CLI arguments for the one-shot semantic projection refresh worker.
#[derive(Args, Debug)]
pub struct SemanticRefreshProjectionsArgs {
    /// Semantic artifact roots to refresh. When omitted, checks
    /// `$PRJ_ROOT/semantic` through the active client root.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}
