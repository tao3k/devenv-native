//! Attachment parsing, audit, and artifact helpers for Wendao document surfaces.

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

#[cfg(feature = "archive-audit")]
/// Archive attachment manifest auditing for routing and cache planning.
pub mod archive_audit;

/// Image attachment preflight auditing for routing and cache planning.
pub mod image_audit;

#[doc(hidden)]
pub mod pdf;
