//! Command-line argument model for semantic SSOT operations.

use clap::{Args, Subcommand};
use std::num::NonZeroUsize;
use std::path::PathBuf;

/// Semantic-oriented client subcommands.
#[derive(Subcommand, Debug)]
pub enum SemanticCommand {
    /// Run one explicit semantic projection metadata refresh worker pass.
    RefreshProjections(SemanticRefreshProjectionsArgs),
}

/// CLI arguments for the semantic projection refresh worker.
#[derive(Args, Debug)]
pub struct SemanticRefreshProjectionsArgs {
    /// Seconds to wait between worker passes. Defaults to one explicit pass.
    #[arg(long = "interval-secs", default_value_t = 0)]
    pub interval_secs: u64,

    /// Maximum worker passes to run. Without this, a positive interval runs
    /// until interrupted.
    #[arg(long = "max-runs")]
    pub max_runs: Option<NonZeroUsize>,

    /// Semantic artifact roots to refresh. When omitted, checks
    /// `$PRJ_ROOT/semantic` through the active client root.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}
