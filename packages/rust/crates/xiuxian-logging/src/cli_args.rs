//! `clap` argument definitions for shared `xiuxian-logging` settings.

use clap::{ArgAction, Args};

use crate::types::{LogColor, LogFormat, LogLevel, LogSettings};

/// Shared CLI logging arguments.
#[derive(Debug, Clone, Args)]
pub struct LogCliArgs {
    /// Increase log verbosity (`-v` => debug, `-vv` => trace).
    #[arg(short = 'v', long = "log-verbose", action = ArgAction::Count, global = true)]
    pub log_verbose: u8,

    /// Explicit log level override.
    ///
    /// This has higher priority than `-v`.
    #[arg(long = "log-level", value_enum, global = true)]
    pub level: Option<LogLevel>,

    /// Log output format.
    #[arg(long = "log-format", value_enum, default_value_t = LogFormat::Pretty, global = true)]
    pub format: LogFormat,

    /// ANSI color behavior.
    #[arg(long = "log-color", value_enum, default_value_t = LogColor::Auto, global = true)]
    pub color: LogColor,

    /// Explicit tracing filter directive (for example: `wendao=debug,hyper=warn`).
    #[arg(long = "log-filter", value_name = "DIRECTIVE", global = true)]
    pub filter: Option<String>,
}

impl Default for LogCliArgs {
    fn default() -> Self {
        Self {
            log_verbose: 0,
            level: None,
            format: LogFormat::Pretty,
            color: LogColor::Auto,
            filter: None,
        }
    }
}

impl From<&LogCliArgs> for LogSettings {
    fn from(value: &LogCliArgs) -> Self {
        Self {
            verbose: value.log_verbose,
            level: value.level,
            format: value.format,
            color: value.color,
            filter: value.filter.clone(),
        }
    }
}
