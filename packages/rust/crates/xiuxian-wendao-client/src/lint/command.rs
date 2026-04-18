use clap::{Args, Subcommand};
use std::path::PathBuf;

/// Lint-oriented client subcommands.
#[derive(Subcommand, Debug)]
pub enum LintCommand {
    /// Lint Markdown files for syntax-oriented failures.
    Markdown(MarkdownLintArgs),
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
