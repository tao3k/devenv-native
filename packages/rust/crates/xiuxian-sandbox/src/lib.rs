//! xiuxian-sandbox - NCL-driven sandbox execution layer
//!
//! # Architecture
//!
//! This module executes pre-generated sandbox configurations.
//! Configuration is produced by NCL and exported as JSON.
//! Rust reads JSON and executes the sandbox - NO configuration parsing in Rust.
//!
//! # Data Flow
//!
//! 1. NCL exports configuration to JSON (nickel export --format json)
//! 2. A host runtime passes the config path to Rust
//! 3. Rust executor reads JSON, spawns nsjail/seatbelt
//! 4. Rust monitors resources and returns results

xiuxian_testing::crate_test_policy_source_harness!("../tests/unit/lib_policy.rs");

pub mod executor;

pub use executor::NsJailExecutor;
pub use executor::SeatbeltExecutor;
pub use executor::{ExecutionResult, MountConfig, SandboxConfig};

/// Platform detection
#[must_use]
pub fn detect_platform() -> String {
    if cfg!(target_os = "linux") {
        "linux".to_string()
    } else if cfg!(target_os = "macos") {
        "macos".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Check if nsjail is available
#[must_use]
pub fn is_nsjail_available() -> bool {
    which::which("nsjail").is_ok()
}

/// Check if sandbox-exec is available (macOS)
#[must_use]
pub fn is_seatbelt_available() -> bool {
    if cfg!(target_os = "macos") {
        which::which("sandbox-exec").is_ok()
    } else {
        false
    }
}
