use clap::Args;
use std::path::PathBuf;

/// Target-scoped selector shared by reusable `wendao get` subcommands.
#[derive(Args, Debug, Clone)]
pub struct GetScopeArgs {
    /// File or directory target. Relative paths are resolved from the client
    /// root.
    #[arg(value_name = "TARGET", default_value = ".")]
    pub target: PathBuf,

    /// Ignore these directory names during recursive target traversal
    /// (repeatable).
    #[arg(long = "ignore", value_name = "DIR")]
    pub ignore_dirs: Vec<String>,
}
