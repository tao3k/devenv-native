//! Julia runtime contracts and feature-scoped adapters.
//!
//! Wendao integration lives behind the `wendao` feature and consumes inert
//! Julia fact catalogs from `xiuxian-polyglot-orchestrator`.

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
                    "Wendao Julia runtime adapters consume feature-scoped polyglot facts",
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
