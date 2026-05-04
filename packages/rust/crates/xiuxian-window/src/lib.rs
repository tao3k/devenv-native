//! xiuxian-window: high-performance session window for 1k–10k turns.
//!
//! Ring-buffer of turn metadata for context building without holding full history in memory.
//! Python can use this via `PyO3` when feature "pybindings" is enabled.

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

mod turn_slot;
mod window;

pub use turn_slot::TurnSlot;
pub use window::SessionWindow;

#[cfg(feature = "pybindings")]
mod pymodule_impl;

#[cfg(feature = "pybindings")]
pub use pymodule_impl::PySessionWindow;
