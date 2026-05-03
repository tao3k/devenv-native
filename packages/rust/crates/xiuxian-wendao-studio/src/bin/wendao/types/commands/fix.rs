//! Fix command arguments for `wendao fix` CLI.

use clap::Args;

/// Apply automated fixes to documents based on semantic audit issues.
///
/// This command uses byte-precise surgical fixes with CAS verification
/// to ensure safe, atomic modifications.
#[derive(Args, Debug)]
pub(crate) struct FixArgs {
    /// Path to the document or directory to fix.
    pub path: String,

    /// Perform a dry run (preview changes without applying).
    #[arg(short = 'n', long = "dry-run", default_value_t = false)]
    pub dry_run: bool,

    /// Minimum confidence threshold for automatic fix application (0.0 - 1.0).
    #[arg(long = "confidence", default_value_t = 0.7)]
    pub confidence_threshold: f32,

    /// Only fix issues of this type (e.g., "`invalid_observation_pattern`").
    #[arg(long = "issue-type")]
    pub issue_type: Option<String>,

    /// Output format for the report.
    #[arg(long = "output-format", default_value = "text")]
    pub output_format: String,

    /// Apply fixes recursively to all markdown files in directory.
    #[arg(long = "recursive", default_value_t = false)]
    pub recursive: bool,
}
