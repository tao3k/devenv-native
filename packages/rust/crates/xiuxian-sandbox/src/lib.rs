//! xiuxian-sandbox - NCL-driven sandbox execution layer.
//!
//! This module executes pre-generated sandbox configurations. Configuration is
//! produced by NCL and exported as JSON. Rust reads JSON and executes the
//! sandbox without parsing NCL.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = {
        rust_lang_project_harness::default_rust_harness_config().with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/lib.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_rationale("crate root owns the public package API for cargo-test verification"),
        )
    }
);

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
