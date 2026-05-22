//! Agent Org read-model `DuckDB` CLI argument DTOs.

use clap::{Args, ValueEnum};
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
    /// Reuse the existing `DuckDB` snapshot when available instead of refreshing first.
    #[arg(long = "cached")]
    pub cached: bool,

    /// Named task-list view. When omitted, default active recovery filtering is used.
    #[arg(long = "view", value_enum)]
    pub view: Option<OrgizeTaskListView>,

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

/// Named `task-list` views for agent recovery and archive control.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OrgizeTaskListView {
    /// Open, non-archived work items.
    Active,
    /// DONE-state rows that are not archived.
    Done,
    /// Archived rows.
    Archived,
    /// Rows tagged as achievements.
    Achievement,
    /// DONE, non-repeating rows ready for archive planning.
    ArchiveCandidate,
    /// Open rows with complete direct progress cookies that still need closure.
    ClosureNeeded,
    /// Rows with scheduled or deadline repeaters.
    Repeating,
}

/// CLI arguments for summarizing agent Org task rows.
#[derive(Args, Debug)]
pub struct OrgizeTaskReportArgs {
    /// Reuse the existing `DuckDB` snapshot when available instead of refreshing first.
    #[arg(long = "cached")]
    pub cached: bool,

    /// Named task-report view. When omitted, renders the full summary report.
    #[arg(long = "view", value_enum)]
    pub view: Option<OrgizeTaskListView>,

    /// Text predicate over task title, source path, outline, tags, properties, and repeaters.
    #[arg(long = "text", value_name = "TEXT")]
    pub text: Option<String>,

    /// Require this Org tag. May be repeated.
    #[arg(long = "tag", value_name = "TAG")]
    pub tags: Vec<String>,

    /// Include archived rows in report sections.
    #[arg(long = "include-archived")]
    pub include_archived: bool,

    /// Render only counters and tag counts, omitting detailed task sections.
    #[arg(long = "summary-only")]
    pub summary_only: bool,

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

    /// Keep only rows whose resolved archive target contains this text.
    #[arg(long = "target", value_name = "PATH_TEXT")]
    pub target: Option<String>,

    /// Keep only rows closed before this date, formatted as YYYY-MM-DD.
    #[arg(long = "closed-before", value_name = "YYYY-MM-DD")]
    pub closed_before: Option<String>,

    /// Maximum number of rows to plan or apply.
    #[arg(long = "limit", default_value_t = 20)]
    pub limit: usize,

    /// Require the selected row count to match before applying or reporting a plan.
    #[arg(long = "expect-selected", value_name = "COUNT")]
    pub expect_selected: Option<usize>,

    /// Org files or directories to materialize before archiving. When omitted, uses `$PRJ_CACHE_HOME/agent/org`.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}
