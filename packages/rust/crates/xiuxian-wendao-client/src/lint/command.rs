//! Command-line argument model for markdown lint operations.

use clap::{Args, Subcommand};
use std::path::PathBuf;

/// Lint-oriented client subcommands.
#[derive(Subcommand, Debug)]
pub enum LintCommand {
    /// Lint Markdown files for syntax-oriented failures.
    Markdown(MarkdownLintArgs),
    /// Lint repo-native semantic SSOT artifacts.
    Semantic(SemanticLintArgs),
}

/// CLI arguments for repo-local Markdown linting.
#[derive(Args, Debug)]
pub struct MarkdownLintArgs {
    /// File or directory roots to inspect. When omitted, lint walks
    /// `link_graph.projects.*.root` from `wendao.toml` before falling back to
    /// the configured `--root`.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,

    /// Extra directory names to skip while walking recursive paths.
    #[arg(long = "skip-dir", value_name = "NAME")]
    pub skip_dirs: Vec<String>,
}

/// CLI arguments for repo-native semantic SSOT linting.
#[derive(Args, Debug)]
pub struct SemanticLintArgs {
    /// Also run advisory semantic SQL guard evidence after schema validation succeeds.
    #[arg(long = "semantic-sql-guard")]
    pub semantic_sql_guard: bool,

    /// Refresh semantic projection source revisions before reporting lint results.
    #[arg(long = "refresh-projections")]
    pub refresh_projections: bool,

    /// Render a read-only lifecycle writeback preview for status transitions.
    #[arg(long = "lifecycle-plan")]
    pub lifecycle_plan: bool,

    /// Semantic artifact roots to inspect. When omitted, lint checks
    /// `$PRJ_ROOT/semantic` through the active client root.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}
