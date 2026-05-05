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
    /// Validation-only semantic lint options.
    #[command(flatten)]
    pub validation: SemanticLintValidationArgs,

    /// Explicit semantic metadata or lifecycle writeback options.
    #[command(flatten)]
    pub writeback: SemanticLintWritebackArgs,

    /// Semantic artifact roots to inspect. When omitted, lint checks
    /// `$PRJ_ROOT/semantic` through the active client root.
    #[arg(value_name = "PATH")]
    pub paths: Vec<PathBuf>,
}

/// Validation-only semantic lint options.
#[derive(Args, Debug)]
pub struct SemanticLintValidationArgs {
    /// Also run advisory semantic SQL guard evidence after schema validation succeeds.
    #[arg(long = "semantic-sql-guard")]
    pub semantic_sql_guard: bool,

    /// Render a read-only lifecycle writeback preview for status transitions.
    #[arg(long = "lifecycle-plan")]
    pub lifecycle_plan: bool,

    /// Semantic projection validation and planning options.
    #[command(flatten)]
    pub projection: SemanticLintProjectionValidationArgs,
}

/// Validation-only semantic projection options.
#[derive(Args, Debug)]
pub struct SemanticLintProjectionValidationArgs {
    /// Render a read-only projection metadata refresh plan.
    #[arg(long = "projection-refresh-plan")]
    pub projection_refresh_plan: bool,

    /// Require active change-intent projection refresh targets to be fresh.
    #[arg(long = "require-fresh-projections")]
    pub require_fresh_projections: bool,
}

/// Explicit semantic metadata or lifecycle writeback options.
#[derive(Args, Debug)]
pub struct SemanticLintWritebackArgs {
    /// Refresh semantic projection source revisions before reporting lint results.
    #[arg(long = "refresh-projections")]
    pub refresh_projections: bool,

    /// Apply pending lifecycle status transitions before reporting lint results.
    #[arg(long = "apply-lifecycle-plan")]
    pub apply_lifecycle_plan: bool,
}
