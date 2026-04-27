use clap::Args;

/// Audit documents for structural and logical consistency.
///
/// This command performs multi-pass auditing (Project Sentinel) to identify
/// dead links, invalid code observations, and content drift.
#[derive(Args, Debug, Clone)]
pub(crate) struct AuditArgs {
    /// Document file or directory to audit. Defaults to current directory.
    #[arg(default_value = ".")]
    pub target: String,

    /// Optional source file directory to verify code observations.
    #[arg(short, long)]
    pub source: Option<String>,

    /// Minimum confidence threshold for fuzzy pattern suggestions (0.0-1.0).
    #[arg(short, long, default_value = "0.7")]
    pub threshold: f32,

    /// Output format.
    #[arg(long, default_value = "xml")]
    pub output_format: String,

    /// Episteme repository directory or episteme.toml file to load.
    #[arg(long, value_name = "EPISTEME")]
    pub load: Option<String>,

    /// Print an episteme authoring template for a framework before creating content.
    #[arg(long, value_name = "FRAMEWORK")]
    pub template: Option<String>,
}
