//! Command-line model for Orgize-backed client operations.

use clap::{Args, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Orgize-backed client subcommands.
#[derive(Subcommand, Debug)]
pub enum OrgizeCommand {
    /// Format Org files with the upstream Orgize formatter.
    Fmt(OrgizeFormatArgs),
    /// Lint Org files with the upstream Orgize linter.
    Lint(OrgizeLintArgs),
    /// Render agent planning cards from Org agenda syntax.
    AgentPlanning(OrgizeAgentPlanningArgs),
    /// Materialize the default `DuckDB` read model for agent Org tasks.
    #[cfg(feature = "orgize-agent-read-model")]
    ReadModel(OrgizeReadModelArgs),
    /// Refresh the `DuckDB` read model and list agent Org task rows.
    #[cfg(feature = "orgize-agent-read-model")]
    TaskList(OrgizeTaskListArgs),
    /// Refresh the `DuckDB` read model and summarize agent Org task rows.
    #[cfg(feature = "orgize-agent-read-model")]
    TaskReport(OrgizeTaskReportArgs),
    /// Plan or apply archival for completed agent Org task rows.
    #[cfg(feature = "orgize-agent-read-model")]
    TaskArchive(OrgizeTaskArchiveArgs),
    /// Render sparse-tree cards from Org match/text predicates.
    SparseTree(OrgizeSparseTreeArgs),
}

/// CLI arguments for Org source formatting.
#[derive(Args, Debug)]
pub struct OrgizeFormatArgs {
    /// Check formatting without writing changes.
    #[arg(long = "check")]
    pub check: bool,

    /// Org files or directories to format. When omitted, walks the client root.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

/// CLI arguments for Org source linting.
#[derive(Args, Debug)]
pub struct OrgizeLintArgs {
    /// Rendered lint output format.
    #[arg(
        id = "orgize-lint-format",
        long = "format",
        value_enum,
        default_value_t = OrgizeLintFormatArg::Compact
    )]
    pub format: OrgizeLintFormatArg,

    /// Alias for `--format json`.
    #[arg(long = "json")]
    pub json: bool,

    /// Highest priority bound for Org priority validation.
    #[arg(long = "priority-highest", value_name = "VALUE")]
    pub priority_highest: Option<String>,

    /// Lowest priority bound for Org priority validation.
    #[arg(long = "priority-lowest", value_name = "VALUE")]
    pub priority_lowest: Option<String>,

    /// Default priority value for Org priority validation.
    #[arg(long = "priority-default", value_name = "VALUE")]
    pub priority_default: Option<String>,

    /// Org files or directories to lint. When omitted, walks the client root.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

/// CLI lint output format.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OrgizeLintFormatArg {
    /// Compact diagnostics for agents.
    Compact,
    /// Human-readable text diagnostics.
    Text,
    /// JSON diagnostics.
    Json,
}

/// CLI arguments for agent planning cards.
#[derive(Args, Debug)]
pub struct OrgizeAgentPlanningArgs {
    /// Inclusive start date in `YYYY-MM-DD` form.
    #[arg(long = "date", value_name = "YYYY-MM-DD")]
    pub date: String,

    /// Optional inclusive end date in `YYYY-MM-DD` form.
    #[arg(long = "end", value_name = "YYYY-MM-DD")]
    pub end: Option<String>,

    /// Include DONE-state tasks.
    #[arg(long = "include-done")]
    pub include_done: bool,

    /// Include archived tasks.
    #[arg(long = "include-archived")]
    pub include_archived: bool,

    /// Include COMMENT tasks.
    #[arg(long = "include-comments")]
    pub include_comments: bool,

    /// Optional Org agenda match expression.
    #[arg(long = "match", value_name = "EXPR")]
    pub match_expression: Option<String>,

    /// Org files or directories to inspect. When omitted, walks the client root.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

/// CLI arguments for agent Org read-model materialization.
#[cfg(feature = "orgize-agent-read-model")]
#[derive(Args, Debug)]
pub struct OrgizeReadModelArgs {
    /// Org files or directories to materialize. When omitted, uses `$PRJ_CACHE_HOME/agent/org`.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

/// CLI arguments for listing agent Org task rows.
#[cfg(feature = "orgize-agent-read-model")]
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
#[cfg(feature = "orgize-agent-read-model")]
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
#[cfg(feature = "orgize-agent-read-model")]
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
