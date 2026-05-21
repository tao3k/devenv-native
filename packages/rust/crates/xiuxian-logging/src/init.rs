//! Runtime initialization for `tracing` and `log` subscribers.

use std::io::IsTerminal;

use thiserror::Error;
use tracing_subscriber::EnvFilter;

use crate::cli_args::LogCliArgs;
use crate::types::{LogColor, LogFormat, LogLevel, LogSettings};

/// Logging bootstrap error.
#[derive(Debug, Error)]
pub enum LogInitError {
    /// Filter directive is invalid.
    #[error("invalid log filter directive: {0}")]
    InvalidFilter(String),
}

/// Initialize global logging from CLI settings.
///
/// `RUST_LOG` always takes precedence over CLI defaults.
///
/// # Errors
/// Returns `LogInitError::InvalidFilter` when the filter directive is invalid.
pub fn init_from_cli(default_target: &str, cli: &LogCliArgs) -> Result<(), LogInitError> {
    init(default_target, &LogSettings::from(cli))
}

/// Initialize global logging with runtime settings.
///
/// This function is idempotent in practice:
/// - if `log` or `tracing` global handlers are already initialized, it returns `Ok(())`.
///
/// # Errors
/// Returns `LogInitError::InvalidFilter` when the filter directive is invalid.
pub fn init(default_target: &str, settings: &LogSettings) -> Result<(), LogInitError> {
    let _ = tracing_log::LogTracer::init();

    let filter = build_filter(default_target, settings)?;
    let ansi = use_ansi(settings.color);

    let init_result = match settings.format {
        LogFormat::Pretty => tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_env_filter(filter)
            .with_ansi(ansi)
            .pretty()
            .with_target(true)
            .try_init(),
        LogFormat::Compact => tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_env_filter(filter)
            .with_ansi(ansi)
            .compact()
            .with_target(true)
            .try_init(),
        LogFormat::Json => tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_env_filter(filter)
            .with_ansi(false)
            .json()
            .flatten_event(true)
            .with_current_span(false)
            .with_span_list(false)
            .with_target(true)
            .try_init(),
    };

    if init_result.is_err() {
        return Ok(());
    }

    Ok(())
}

fn use_ansi(color: LogColor) -> bool {
    match color {
        LogColor::Always => true,
        LogColor::Never => false,
        LogColor::Auto => std::io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none(),
    }
}

fn build_filter(default_target: &str, settings: &LogSettings) -> Result<EnvFilter, LogInitError> {
    if let Ok(from_env) = EnvFilter::try_from_default_env() {
        return Ok(from_env);
    }

    let Some(filter_directive) = settings.filter.as_deref() else {
        let level = settings
            .level
            .unwrap_or_else(|| LogLevel::from_verbose(settings.verbose));
        let normalized_target = default_target.replace('-', "_");
        let directive = format!("{normalized_target}={},info", level.as_directive());
        return EnvFilter::try_new(directive)
            .map_err(|error| LogInitError::InvalidFilter(error.to_string()));
    };

    EnvFilter::try_new(filter_directive)
        .map_err(|error| LogInitError::InvalidFilter(error.to_string()))
}
