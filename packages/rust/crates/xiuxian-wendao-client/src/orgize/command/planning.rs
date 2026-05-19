//! Agent-planning `orgize` CLI argument DTOs.

use clap::Args;
use std::path::PathBuf;

/// CLI arguments for agent planning cards.
///
/// Raw DTO boundary: these flags mirror CLI query toggles and are not stored
/// as a long-lived domain model.
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
