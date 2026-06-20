//! Julia crate build-time project harness gate.

fn main() {
    xiuxian_rust_workspace_harness::assert_member_harness_build_gate_from_env_with_configure(
        |config| {
            config
        .with_verification_profile_hint(
            xiuxian_rust_workspace_harness::RustVerificationProfileHint::new(
                "src/lib.rs",
                [xiuxian_rust_workspace_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_rationale("crate root owns the public package API for build-time verification"),
        )
        .with_verification_profile_hint(
            xiuxian_rust_workspace_harness::RustVerificationProfileHint::new(
                "src/polyglot/",
                [xiuxian_rust_workspace_harness::RustOwnerResponsibility::PublicApi],
            )
            .with_task_kinds([xiuxian_rust_workspace_harness::RustVerificationTaskKind::Regression])
            .with_task_contract(
                xiuxian_rust_workspace_harness::RustVerificationTaskKind::Regression,
                xiuxian_rust_workspace_harness::RustVerificationTaskContract::new(
                    xiuxian_rust_workspace_harness::RustVerificationPhase::AfterUnitTestsPass,
                    "Regression check must exercise the Julia polyglot readiness bridge",
                    [
                        xiuxian_rust_workspace_harness::RustVerificationRequirement::new(
                            "command",
                            "cargo test -p xiuxian-julia-core --lib polyglot",
                        ),
                        xiuxian_rust_workspace_harness::RustVerificationRequirement::new(
                            "target",
                            "lib unit tests mounted from tests/unit/polyglot/",
                        ),
                        xiuxian_rust_workspace_harness::RustVerificationRequirement::new(
                            "coverage",
                            "profile refs, manifest refs, readiness evidence, admission, and snapshots",
                        ),
                    ],
                ),
            )
            .with_rationale(
                "Julia polyglot bridge owns readiness evidence projections for the orchestrator chain",
            ),
        )
        },
    );
}
