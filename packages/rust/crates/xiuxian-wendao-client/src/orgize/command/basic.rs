//! Basic `orgize` format and lint CLI argument DTOs.

use clap::{Args, ValueEnum};
use std::path::PathBuf;

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

    /// Apply safe Org source fixes before rendering lint diagnostics.
    #[arg(long = "fix")]
    pub fix: bool,

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
