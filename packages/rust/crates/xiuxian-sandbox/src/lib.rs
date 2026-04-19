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

use std::{env, ffi::OsStr, path::Path};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

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
    is_command_available("nsjail")
}

/// Check if sandbox-exec is available (macOS)
#[must_use]
pub fn is_seatbelt_available() -> bool {
    if cfg!(target_os = "macos") {
        is_command_available("sandbox-exec")
    } else {
        false
    }
}

fn is_command_available(command: &str) -> bool {
    command_in_path(command, env::var_os("PATH").as_deref())
}

fn command_in_path(command: &str, path_env: Option<&OsStr>) -> bool {
    path_env.is_some_and(|paths| {
        env::split_paths(paths).any(|path_dir| is_executable_file(&path_dir.join(command)))
    })
}

fn is_executable_file(path: &Path) -> bool {
    match path.metadata() {
        Ok(metadata) if metadata.is_file() => {
            #[cfg(unix)]
            {
                metadata.permissions().mode() & 0o111 != 0
            }
            #[cfg(not(unix))]
            {
                true
            }
        }
        _ => false,
    }
}

#[cfg(test)]
#[path = "../tests/unit/lib.rs"]
mod tests;
