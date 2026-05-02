//! Unified structured logging bootstrap for Xiuxian Rust binaries.
//!
//! This crate provides:
//! - one shared CLI logging surface (`-v` / `--log-verbose`, `--log-format`, `--log-filter`),
//! - colorful human-readable output (pretty/compact),
//! - structured JSON output mode,
//! - bridging from `log` macros to `tracing` (`tracing_log::LogTracer`).

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!();

mod argv_split;
mod cli_args;
mod init;
mod types;

pub use argv_split::split_logging_args;
pub use cli_args::LogCliArgs;
pub use init::{LogInitError, init, init_from_cli};
pub use types::{LogColor, LogFormat, LogLevel, LogSettings};
