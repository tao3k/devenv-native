//! Sentinel subcommand for starting the Project Sentinel file observer.
//!
//! This command starts the real-time file watching system that monitors
//! source code changes and emits semantic drift signals.

use clap::{Args, Subcommand};

/// Arguments for the `sentinel` subcommand.
#[derive(Debug, Args, Clone)]
pub(crate) struct SentinelArgs {
    #[command(subcommand)]
    pub(crate) command: SentinelCommand,
}

/// Sentinel subcommands.
#[derive(Debug, Subcommand, Clone)]
pub(crate) enum SentinelCommand {
    /// Start watching paths for file changes.
    Watch(SentinelWatchArgs),
}

/// Arguments for `sentinel watch`.
#[derive(Debug, Args, Clone)]
pub(crate) struct SentinelWatchArgs {
    /// Paths to watch (comma-separated or multiple flags).
    #[arg(short, long)]
    pub(crate) paths: Vec<String>,

    /// Debounce duration in milliseconds (default: 1000).
    #[arg(long, default_value = "1000")]
    pub(crate) debounce_ms: u64,
}

/// Create a Sentinel command from arguments.
#[cfg(test)]
pub(crate) fn sentinel(args: &SentinelArgs) -> super::Command {
    super::Command::Sentinel(args.clone())
}

#[cfg(test)]
#[path = "../../../../../tests/unit/bin/wendao/types/commands/sentinel.rs"]
mod tests;
