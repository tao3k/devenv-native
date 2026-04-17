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
    #[arg(long, short = 'o', value_enum, default_value_t = OutputFormat::Json, global = true)]
    pub output: OutputFormat,

    /// Global structured logging controls.
    #[command(flatten)]
    pub logging: LogCliArgs,

    #[command(subcommand)]
    pub command: Command,
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use crate::types::Command;
    use clap::Parser;

    #[test]
    fn parses_embedded_client_lint_command() {
        let cli = Cli::parse_from(["wendao", "lint", "markdown", "README.md"]);
        assert!(matches!(cli.command, Command::Client(_)));
    }
}
