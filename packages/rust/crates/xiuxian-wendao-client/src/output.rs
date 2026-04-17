use clap::ValueEnum;

/// Output format supported by lightweight client commands.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    /// Plain-text diagnostics suitable for humans and LLM readers.
    Text,
}
