//! Public logging option types for `xiuxian-logging` callers.

use clap::ValueEnum;

/// Log rendering format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LogFormat {
    /// Colorful multi-line rendering for humans.
    Pretty,
    /// Compact single-line rendering for humans.
    Compact,
    /// Structured JSON rendering for machines.
    Json,
}

impl std::str::FromStr for LogFormat {
    type Err = ();

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.to_ascii_lowercase().as_str() {
            "pretty" => Ok(Self::Pretty),
            "compact" => Ok(Self::Compact),
            "json" => Ok(Self::Json),
            _ => Err(()),
        }
    }
}

/// ANSI color choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LogColor {
    /// Use ANSI colors when stderr is a terminal and `NO_COLOR` is unset.
    Auto,
    /// Always use ANSI colors.
    Always,
    /// Never use ANSI colors.
    Never,
}

impl std::str::FromStr for LogColor {
    type Err = ();

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            _ => Err(()),
        }
    }
}

/// Explicit log level override.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LogLevel {
    /// Error only.
    Error,
    /// Warn and above.
    Warn,
    /// Info and above.
    Info,
    /// Debug and above.
    Debug,
    /// Trace and above.
    Trace,
}

impl LogLevel {
    pub(crate) fn as_directive(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }

    pub(crate) const fn from_verbose(verbose: u8) -> Self {
        match verbose {
            0 => Self::Info,
            1 => Self::Debug,
            _ => Self::Trace,
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = ();

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.to_ascii_lowercase().as_str() {
            "error" => Ok(Self::Error),
            "warn" | "warning" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            "trace" => Ok(Self::Trace),
            _ => Err(()),
        }
    }
}

/// Runtime logging settings.
#[derive(Debug, Clone)]
pub struct LogSettings {
    /// Verbosity count from CLI (`-v`, `-vv`).
    pub verbose: u8,
    /// Optional explicit level.
    pub level: Option<LogLevel>,
    /// Rendering format.
    pub format: LogFormat,
    /// Color policy.
    pub color: LogColor,
    /// Optional tracing filter directive.
    pub filter: Option<String>,
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            verbose: 0,
            level: None,
            format: LogFormat::Pretty,
            color: LogColor::Auto,
            filter: None,
        }
    }
}
