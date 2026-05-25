//! Julia runtime contracts and feature-scoped adapters.
//!
//! This crate is the successor boundary for Julia runtime facts that should not
//! be owned by a Wendao-specific plugin crate. Wendao integration lives behind
//! the `wendao` feature.

#[cfg(test)]
rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = {
        rust_lang_project_harness::default_rust_harness_config()
            .with_verification_profile_hint(
                rust_lang_project_harness::RustVerificationProfileHint::new(
                    "src/lib.rs",
                    [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
                )
                .with_task_kinds([rust_lang_project_harness::RustVerificationTaskKind::Regression])
                .with_rationale("crate root owns the Julia runtime public contract boundary"),
            )
            .with_verification_profile_hint(
                rust_lang_project_harness::RustVerificationProfileHint::new(
                    "src/wendao/",
                    [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
                )
                .with_task_kinds([rust_lang_project_harness::RustVerificationTaskKind::Regression])
                .with_rationale(
                    "Wendao Julia profile facts are feature-scoped under the runtime crate",
                ),
            )
    }
);

#[cfg(feature = "wendao")]
/// Wendao-facing Julia runtime facts and contract identities.
pub mod wendao;

#[cfg(test)]
#[path = "../tests/unit/lib/mod.rs"]
mod tests;
