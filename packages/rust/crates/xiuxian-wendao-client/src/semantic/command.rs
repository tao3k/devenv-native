//! Command-line argument model for semantic SSOT operations.

use clap::{Args, Subcommand};
use std::num::NonZeroUsize;
use std::path::PathBuf;

/// Semantic-oriented client subcommands.
#[derive(Subcommand, Debug)]
pub enum SemanticCommand {
    /// Check the current advisory semantic read-model snapshot revision.
    CheckReadModelSnapshot(SemanticCheckReadModelSnapshotArgs),
    /// Describe advisory semantic read-model tables and columns.
    DescribeReadModel(SemanticDescribeReadModelArgs),
    /// Execute a read-only SQL query over advisory semantic read-model tables.
    QueryReadModel(SemanticReadModelQueryArgs),
    /// Run the semantic projection metadata refresh worker.
    RefreshProjections(SemanticRefreshProjectionsArgs),
    /// Render deterministic advisory semantic read-model snapshot revisions.
    SnapshotReadModel(SemanticSnapshotReadModelArgs),
}

/// CLI arguments for advisory semantic read-model snapshot verification.
#[derive(Args, Debug)]
pub struct SemanticCheckReadModelSnapshotArgs {
    /// Expected aggregate snapshot revision, including the `blake3:` prefix.
    #[arg(long = "expect", value_name = "SNAPSHOT_REVISION")]
    pub expected_snapshot_revision: String,

    /// Semantic artifact root to check. When omitted, checks
    /// `$PRJ_ROOT/semantic` through the active client root.
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
}

/// CLI arguments for advisory semantic read-model catalog inspection.
#[derive(Args, Debug)]
pub struct SemanticDescribeReadModelArgs {
    /// Semantic artifact root to describe. When omitted, checks
    /// `$PRJ_ROOT/semantic` through the active client root.
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
}

/// CLI arguments for advisory semantic read-model SQL queries.
#[derive(Args, Debug)]
pub struct SemanticReadModelQueryArgs {
    /// SQL statement to execute against `semantic_objects`,
    /// `semantic_relations`, and `semantic_projection_state`.
    #[arg(long = "query", short = 'q', value_name = "SQL")]
    pub query_text: String,

    /// Semantic artifact root to query. When omitted, checks
    /// `$PRJ_ROOT/semantic` through the active client root.
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
}

/// CLI arguments for advisory semantic read-model snapshot inspection.
#[derive(Args, Debug)]
pub struct SemanticSnapshotReadModelArgs {
    /// Semantic artifact root to snapshot. When omitted, checks
    /// `$PRJ_ROOT/semantic` through the active client root.
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
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

    /// Refuse to start the refresh worker unless the root git worktree is
    /// clean.
    #[arg(long = "require-clean-worktree")]
    pub require_clean_worktree: bool,

    /// Semantic artifact roots to refresh. When omitted, checks
    /// `$PRJ_ROOT/semantic` through the active client root.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}
