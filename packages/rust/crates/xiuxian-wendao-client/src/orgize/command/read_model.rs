//! Agent Org read-model `DuckDB` CLI argument DTOs.

use clap::Args;
use std::path::PathBuf;

/// CLI arguments for agent Org read-model materialization.
#[derive(Args, Debug)]
pub struct OrgizeReadModelArgs {
    /// Org files or directories to materialize. When omitted, uses `$PRJ_CACHE_HOME/agent/org`.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

/// CLI arguments for listing agent Org task rows.
#[derive(Args, Debug)]
pub struct OrgizeTaskListArgs {
    /// Text predicate over task title, source path, outline, tags, properties, and repeaters.
    #[arg(long = "text", value_name = "TEXT")]
    pub text: Option<String>,

    /// Require this Org tag. May be repeated.
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,

    /// Include DONE-state tasks.
    #[arg(long = "include-done")]
    pub include_done: bool,

    /// Include archived tasks.
    #[arg(long = "include-archived")]
    pub include_archived: bool,

    /// Maximum number of rows to render.
    #[arg(long = "limit", default_value_t = 20)]
    pub limit: usize,

    /// Org files or directories to materialize before listing. When omitted, uses `$PRJ_CACHE_HOME/agent/org`.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

/// CLI arguments for summarizing agent Org task rows.
#[derive(Args, Debug)]
pub struct OrgizeTaskReportArgs {
    /// Text predicate over task title, source path, outline, tags, properties, and repeaters.
    #[arg(long = "text", value_name = "TEXT")]
    pub text: Option<String>,

    /// Require this Org tag. May be repeated.
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,

    /// Include archived rows in report sections.
    #[arg(long = "include-archived")]
    pub include_archived: bool,

    /// Maximum number of rows to render per section.
    #[arg(long = "limit", default_value_t = 10)]
    pub limit: usize,

    /// Org files or directories to materialize before reporting. When omitted, uses `$PRJ_CACHE_HOME/agent/org`.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

/// CLI arguments for planning or applying completed agent task archival.
#[derive(Args, Debug)]
pub struct OrgizeTaskArchiveArgs {
    /// Apply the archive plan. Omit this flag for a read-only plan.
    #[arg(long = "apply")]
    pub apply: bool,

    /// Text predicate over task title, source path, outline, tags, properties, and repeaters.
    #[arg(long = "text", value_name = "TEXT")]
    pub text: Option<String>,

    /// Require this Org tag. May be repeated.
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,

    /// Maximum number of rows to plan or apply.
    #[arg(long = "limit", default_value_t = 20)]
    pub limit: usize,

    /// Org files or directories to materialize before archiving. When omitted, uses `$PRJ_CACHE_HOME/agent/org`.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}
