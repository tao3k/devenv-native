use crate::{LintCommand, OutputFormat};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use xiuxian_logging::LogCliArgs;

/// Standalone CLI contract for the lightweight Wendao client.
#[derive(Parser, Debug)]
#[command(
    name = "wendao",
    about = "Lightweight Wendao client CLI for local document tooling",
    arg_required_else_help = true
)]
pub struct ClientCli {
    /// Root directory used to resolve relative lint paths.
    #[arg(
        long,
        short = 'r',
        value_name = "DIR",
        default_value = ".",
        global = true
    )]
    pub root: PathBuf,

    /// Output format for lint diagnostics.
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
    /// Lint local repository documents.
    Lint {
        /// Lint-oriented subcommand selection.
        #[command(subcommand)]
        command: LintCommand,
    },
}
