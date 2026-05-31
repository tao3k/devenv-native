//! xiuxian-sandbox - NCL-driven sandbox execution layer.
//!
//! This module executes pre-generated sandbox configurations. Configuration is
//! produced by NCL and exported as JSON. Rust reads JSON and executes the
//! sandbox without parsing NCL.

/// Sandbox executor implementations and shared execution types.
pub mod executor;
mod platform;

pub use executor::{
    ExecutionResult, MountConfig, NsJailExecutor, SandboxConfig, SandboxMode, SeatbeltExecutor,
};
pub use platform::{detect_platform, is_nsjail_available, is_seatbelt_available};

#[cfg(test)]
#[path = "../tests/unit/lib.rs"]
mod tests;
