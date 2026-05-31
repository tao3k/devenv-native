//! Julia crate build-time project harness gate.

fn main() {
    let config = rust_lang_project_harness::default_rust_harness_config()
        .with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/lib.rs",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_rationale("crate root owns the public package API for build-time verification"),
        )
        .with_verification_profile_hint(
            rust_lang_project_harness::RustVerificationProfileHint::new(
                "src/polyglot/",
                [rust_lang_project_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_task_kinds([rust_lang_project_harness::RustVerificationTaskKind::Regression])
            .with_task_contract(
                rust_lang_project_harness::RustVerificationTaskKind::Regression,
                rust_lang_project_harness::RustVerificationTaskContract::new(
                    rust_lang_project_harness::RustVerificationPhase::AfterUnitTestsPass,
                    "Regression check must exercise the Julia polyglot readiness bridge",
                    [
                        rust_lang_project_harness::RustVerificationRequirement::new(
                            "command",
                            "cargo test -p xiuxian-julia-core --lib polyglot",
                        ),
                        rust_lang_project_harness::RustVerificationRequirement::new(
                            "target",
                            "lib unit tests mounted from tests/unit/polyglot/",
                        ),
                        rust_lang_project_harness::RustVerificationRequirement::new(
                            "coverage",
                            "profile refs, manifest refs, readiness evidence, admission, and snapshots",
                        ),
                    ],
                ),
            )
            .with_rationale(
                "Julia polyglot bridge owns readiness evidence projections for the orchestrator chain",
            ),
        );
    rust_lang_project_harness::assert_rust_project_harness_build_clean_from_env_with_config(
        &config,
    );
}
