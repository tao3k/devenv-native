//! Sparse-tree `orgize` CLI argument DTOs.

use clap::Args;
use std::path::PathBuf;

/// CLI arguments for sparse-tree projections.
#[derive(Args, Debug)]
pub struct OrgizeSparseTreeArgs {
    /// Text predicate over titles, body slices, metadata, links, and targets.
    #[arg(long = "text", value_name = "TEXT")]
    pub text: Option<String>,

    /// Optional Org agenda match expression.
    #[arg(long = "match", value_name = "EXPR")]
    pub match_expression: Option<String>,

    /// Visibility controls.
    #[command(flatten)]
    pub visibility: OrgizeSparseTreeVisibilityArgs,

    /// Rendering controls.
    #[command(flatten)]
    pub render: OrgizeSparseTreeRenderArgs,

    /// Org files or directories to inspect. When omitted, walks the client root.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

/// CLI visibility arguments for sparse-tree projections.
#[derive(Args, Debug)]
pub struct OrgizeSparseTreeVisibilityArgs {
    /// Exclude DONE-state tasks.
    #[arg(long = "exclude-done")]
    pub exclude_done: bool,

    /// Exclude archived tasks.
    #[arg(long = "exclude-archived")]
    pub exclude_archived: bool,

    /// Include COMMENT tasks.
    #[arg(long = "include-comments")]
    pub include_comments: bool,
}

/// CLI rendering arguments for sparse-tree projections.
#[derive(Args, Debug)]
pub struct OrgizeSparseTreeRenderArgs {
    /// Render skipped section receipts.
    #[arg(long = "explain-skips")]
    pub explain_skips: bool,
}
