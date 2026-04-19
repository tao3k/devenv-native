use crate::{GetCommand, LintCommand, OutputFormat};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use xiuxian_logging::LogCliArgs;

/// Standalone CLI contract for the lightweight Wendao client.
#[derive(Parser, Debug)]
#[command(
    name = "wendao-client",
    about = "Lightweight Wendao client CLI for local document tooling",
    arg_required_else_help = true
)]
pub struct ClientCli {
    /// Root directory used to resolve relative client command paths.
    #[arg(
        long,
        short = 'r',
        value_name = "DIR",
        default_value = ".",
        global = true
    )]
    pub root: PathBuf,

    /// Explicit wendao config file path used by config-aware client commands.
    #[arg(long = "conf", short = 'c', value_name = "FILE", global = true)]
    pub config_file: Option<PathBuf>,

    /// Output format for rendered command results.
    #[arg(long, short = 'o', value_enum, default_value_t = OutputFormat::Text, global = true)]
    pub output: OutputFormat,

    /// Global structured logging controls.
    #[command(flatten)]
    pub logging: LogCliArgs,

    /// Reusable client command tree.
    #[command(subcommand)]
    pub command: ClientCommand,
}

/// Reusable client command tree that can be embedded into larger Wendao CLIs.
#[derive(Subcommand, Debug)]
pub enum ClientCommand {
    /// Materialize deterministic docs/page-index collections from one directory scope.
    Get {
        /// Get-oriented subcommand selection.
        #[command(subcommand)]
        command: GetCommand,
    },
    /// Lint local repository documents.
    Lint {
        /// Lint-oriented subcommand selection.
        #[command(subcommand)]
        command: LintCommand,
    },
}
