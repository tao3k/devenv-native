pub(super) fn wendao_server_harness_config() -> rust_lang_project_harness::RustHarnessConfig {
    use rust_lang_project_harness::{
        RustOwnerResponsibility, RustVerificationPhase, RustVerificationProfileHint,
        RustVerificationRequirement, RustVerificationSkillBinding, RustVerificationSkillDescriptor,
        RustVerificationTaskContract, RustVerificationTaskKind, default_rust_harness_config,
    };

    default_rust_harness_config()
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/transport/mod.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::LatencySensitive,
                    RustOwnerResponsibility::AvailabilityCritical,
                    RustOwnerResponsibility::SecurityBoundary,
                ],
            )
            .with_task_kinds([
                RustVerificationTaskKind::Performance,
                RustVerificationTaskKind::Stress,
                RustVerificationTaskKind::Security,
                RustVerificationTaskKind::Regression,
            ])
            .with_task_contract(
                RustVerificationTaskKind::Performance,
                RustVerificationTaskContract::new(
                    RustVerificationPhase::AfterUnitTestsPass,
                    "transport verification must report Flight/gRPC route dispatch latency evidence",
                    [
                        RustVerificationRequirement::new(
                            "benchmark_command",
                            "cargo bench -p xiuxian-wendao-server --features performance --bench wendao_transport_performance",
                        ),
                        RustVerificationRequirement::new(
                            "baseline",
                            "transport latency baseline name or commit",
                        ),
                        RustVerificationRequirement::new(
                            "regression_threshold",
                            "accepted Flight/gRPC route dispatch regression threshold",
                        ),
                        RustVerificationRequirement::new(
                            "latency_or_throughput",
                            "Flight/gRPC route dispatch latency or throughput result",
                        ),
                    ],
                ),
            )
            .with_rationale("server owns only the high-throughput Flight/gRPC transport boundary"),
        )
        .with_verification_responsibility_task_kinds(
            RustOwnerResponsibility::LatencySensitive,
            [RustVerificationTaskKind::Performance],
        )
        .with_verification_responsibility_task_kinds(
            RustOwnerResponsibility::AvailabilityCritical,
            [RustVerificationTaskKind::Stress],
        )
        .with_verification_responsibility_task_kinds(
            RustOwnerResponsibility::SecurityBoundary,
            [RustVerificationTaskKind::Security],
        )
        .with_verification_skill_binding(
            RustVerificationTaskKind::Performance,
            RustVerificationSkillBinding::new("rust-verification-performance")
                .with_adapter("criterion"),
        )
        .with_verification_skill_descriptor(
            RustVerificationSkillDescriptor::criterion_performance(),
        )
        .with_verification_skill_binding(
            RustVerificationTaskKind::Stress,
            RustVerificationSkillBinding::new("rust-verification-stress").with_adapter("k6"),
        )
        .with_verification_skill_descriptor(RustVerificationSkillDescriptor::k6_stress())
        .with_verification_skill_binding(
            RustVerificationTaskKind::Security,
            RustVerificationSkillBinding::new("rust-verification-security").with_adapter("semgrep"),
        )
}

rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = wendao_server_harness_config()
);
