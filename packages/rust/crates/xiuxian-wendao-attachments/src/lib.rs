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
        .with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/polyglot.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_task_kinds([rust_lang_project_harness::RustVerificationTaskKind::Regression])
            .with_task_contract(
                rust_lang_project_harness::RustVerificationTaskKind::Regression,
                rust_lang_project_harness::RustVerificationTaskContract::new(
                    rust_lang_project_harness::RustVerificationPhase::AfterUnitTestsPass,
                    "Regression check must exercise the feature-gated OCR polyglot bridge",
                    [
                        rust_lang_project_harness::RustVerificationRequirement::new(
                            "command",
                            "cargo test -p xiuxian-wendao-attachments --features pdf-source-range --lib polyglot",
                        ),
                        rust_lang_project_harness::RustVerificationRequirement::new(
                            "feature",
                            "pdf-source-range",
                        ),
                        rust_lang_project_harness::RustVerificationRequirement::new(
                            "coverage",
                            "OCR route refs, pressure evidence, snapshots, and schedule-plan projections",
                        ),
                    ],
                ),
            )
            .with_rationale(
                "attachment polyglot bridge owns OCR shard evidence and schedule-plan projections",
            ),
        )
        .with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/polyglot.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_task_kinds([rust_lang_project_harness::RustVerificationTaskKind::Regression])
            .with_task_contract(
                rust_lang_project_harness::RustVerificationTaskKind::Regression,
                rust_lang_project_harness::RustVerificationTaskContract::new(
                    rust_lang_project_harness::RustVerificationPhase::AfterUnitTestsPass,
                    "Regression check must exercise the feature-gated audio polyglot bridge",
                    [
                        rust_lang_project_harness::RustVerificationRequirement::new(
                            "command",
                            "cargo test -p xiuxian-wendao-attachments --features audio-shard-arrow --lib polyglot",
                        ),
                        rust_lang_project_harness::RustVerificationRequirement::new(
                            "feature",
                            "audio-shard-arrow",
                        ),
                        rust_lang_project_harness::RustVerificationRequirement::new(
                            "coverage",
                            "audio pressure evidence and schedule-plan projections",
                        ),
                    ],
                ),
            )
            .with_rationale(
                "attachment polyglot bridge owns audio shard evidence and schedule-plan projections",
            ),
        )
    }
);

/// Model-agnostic audio shard planning and admission identity contracts.
pub mod audio;

#[cfg(feature = "archive-audit")]
/// Archive attachment manifest auditing for routing and cache planning.
pub mod archive_audit;

/// Image attachment preflight auditing for routing and cache planning.
pub mod image_audit;

#[cfg(feature = "image-shards")]
/// Standalone image attachment shard planning and materialization.
pub mod image_shards;

#[doc(hidden)]
pub mod pdf;

/// Read-only projections from attachment-owned contracts into polyglot contracts.
pub mod polyglot;
