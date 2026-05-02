//! xiuxian-sandbox - NCL-driven sandbox execution layer.
//!
//! This module executes pre-generated sandbox configurations. Configuration is
//! produced by NCL and exported as JSON. Rust reads JSON and executes the
//! sandbox without parsing NCL.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!();

/// Sandbox executor implementations and shared execution types.
pub mod executor;
mod platform;

pub use executor::{ExecutionResult, MountConfig, NsJailExecutor, SandboxConfig, SeatbeltExecutor};
pub use platform::{detect_platform, is_nsjail_available, is_seatbelt_available};

#[cfg(test)]
#[path = "../tests/unit/lib.rs"]
mod tests;
