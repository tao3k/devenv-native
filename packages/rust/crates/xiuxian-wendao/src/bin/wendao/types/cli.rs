use super::commands::Command;
use super::enums::OutputFormat;
use clap::Parser;
use std::path::PathBuf;
use xiuxian_logging::LogCliArgs;

#[derive(Parser, Debug)]
#[command(
    name = "wendao",
    about = "Wendao link-graph CLI for local note search and traversal",
    arg_required_else_help = true
)]
pub(crate) struct Cli {
    /// Notebook root directory.
    #[arg(
        long,
        short = 'r',
        value_name = "DIR",
        default_value = ".",
        global = true
    )]
    pub root: PathBuf,

    /// Explicit wendao config file path (for example: `.config/xiuxian-artisan-workshop/wendao.yaml`).
    ///
    /// This overrides the default user settings path resolution.
    #[arg(long = "conf", short = 'c', value_name = "FILE", global = true)]
    pub config_file: Option<PathBuf>,

    /// Include only these relative directories (repeatable).
    #[arg(long = "include-dir", value_name = "DIR", global = true)]
    pub include_dirs: Vec<String>,

    /// Exclude these directory names globally (repeatable).
    #[arg(long = "exclude-dir", value_name = "DIR", global = true)]
    pub exclude_dirs: Vec<String>,

    /// Output format.
    #[arg(long, short = 'o', value_enum, global = true)]
    pub output: Option<OutputFormat>,

    /// Global structured logging controls.
    #[command(flatten)]
    pub logging: LogCliArgs,

    #[command(subcommand)]
    pub command: Command,
}

#[cfg(test)]
#[path = "../../../../tests/unit/bin/wendao/types/cli.rs"]
mod tests;

impl Cli {
    #[must_use]
    pub(crate) fn output_or_json(&self) -> OutputFormat {
        self.output.unwrap_or(OutputFormat::Json)
    }

    #[must_use]
    pub(crate) fn embedded_client_output(&self) -> OutputFormat {
        self.output.unwrap_or(OutputFormat::Text)
    }
}
