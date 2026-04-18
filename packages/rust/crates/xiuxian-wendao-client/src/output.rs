use clap::ValueEnum;

/// Output format supported by lightweight client commands.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    /// Plain-text diagnostics suitable for humans and LLM readers.
    Text,
    /// Compact JSON diagnostics suitable for machine parsing.
    Json,
    /// Pretty-printed JSON diagnostics for human-readable structured output.
    Pretty,
}
