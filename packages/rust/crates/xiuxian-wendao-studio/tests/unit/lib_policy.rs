use rust_lang_project_harness::{
    RustHarnessConfig, RustOwnerResponsibility, RustVerificationPhase, RustVerificationProfileHint,
    RustVerificationRequirement, RustVerificationSkillBinding, RustVerificationSkillDescriptor,
    RustVerificationTaskContract, RustVerificationTaskKind, default_rust_harness_config,
};

const STUDIO_BENCH_COMMAND: &str =
    "cargo bench -p xiuxian-wendao-studio --features performance --bench wendao_studio_performance";

pub(super) fn wendao_studio_harness_config() -> RustHarnessConfig {
    if contracts_only_surface() {
        return contracts_harness_config();
    }

    default_rust_harness_config()
        .with_verification_profile_hint(studio_contracts_hint())
        .with_verification_profile_hint(studio_gateway_hint())
        .with_verification_profile_hint(studio_router_hint())
        .with_verification_profile_hint(studio_perf_support_hint())
        .with_verification_profile_hint(studio_openapi_hint())
        .with_verification_responsibility_task_kinds(
            RustOwnerResponsibility::LatencySensitive,
            [RustVerificationTaskKind::Performance],
        )
        .with_verification_responsibility_task_kinds(
            RustOwnerResponsibility::AvailabilityCritical,
            [RustVerificationTaskKind::Chaos],
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
        .with_verification_skill_descriptor(security_skill_descriptor())
}

fn contracts_only_surface() -> bool {
    cfg!(all(
        feature = "contracts",
        not(any(
            feature = "studio",
            feature = "http-router",
            feature = "flight-transport",
            feature = "local-runtime",
            feature = "cli-bin-support"
        ))
    ))
}

fn contracts_harness_config() -> RustHarnessConfig {
    let mut config =
        default_rust_harness_config().with_verification_profile_hint(studio_contracts_hint());
    config.include_tests = false;
    config.source_dir_names = vec!["src".to_string()];
    config.ignored_dir_names.insert("benches".to_string());
    config.ignored_dir_names.insert("bin".to_string());
    config.ignored_dir_names.insert("bin_support".to_string());
    config.ignored_dir_names.insert("examples".to_string());
    config.ignored_dir_names.insert("studio".to_string());
    config
}

fn studio_contracts_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/contracts/routes.rs",
        [RustOwnerResponsibility::PublicApi],
    )
    .with_task_kinds([RustVerificationTaskKind::Regression])
    .with_rationale("Studio contracts own the lightweight HTTP route inventory")
}

fn studio_gateway_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/studio/mod.rs",
        [
            RustOwnerResponsibility::PublicApi,
            RustOwnerResponsibility::SecurityBoundary,
            RustOwnerResponsibility::AvailabilityCritical,
        ],
    )
    .with_task_kinds([
        RustVerificationTaskKind::Security,
        RustVerificationTaskKind::Stress,
        RustVerificationTaskKind::Chaos,
        RustVerificationTaskKind::Regression,
    ])
    .with_rationale("Studio owns the public HTTP gateway and service-boundary behavior")
}

fn studio_router_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/studio/router/routes.rs",
        [
            RustOwnerResponsibility::PublicApi,
            RustOwnerResponsibility::LatencySensitive,
        ],
    )
    .with_task_kinds([
        RustVerificationTaskKind::Performance,
        RustVerificationTaskKind::Stress,
        RustVerificationTaskKind::Regression,
    ])
    .with_task_contract(
        RustVerificationTaskKind::Performance,
        studio_benchmark_contract(
            "Criterion benchmark must report Studio router construction latency evidence",
            "accepted router construction latency regression threshold",
            "target/criterion artifact path for the Studio router benchmark",
        ),
    )
    .with_rationale("router composition is a public hot path for Studio API startup")
}

fn studio_perf_support_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/studio/perf_support/mod.rs",
        [RustOwnerResponsibility::LatencySensitive],
    )
    .with_task_kinds([RustVerificationTaskKind::Performance])
    .with_task_contract(
        RustVerificationTaskKind::Performance,
        studio_benchmark_contract(
            "Criterion benchmark must report Studio state bootstrap latency evidence",
            "accepted state bootstrap latency regression threshold",
            "target/criterion artifact path for the Studio state benchmark",
        ),
    )
    .with_rationale("perf support owns the Studio gateway benchmark fixtures")
}

fn studio_openapi_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new("src/openapi.rs", [RustOwnerResponsibility::PublicApi])
        .with_task_kinds([RustVerificationTaskKind::Regression])
        .with_rationale("OpenAPI exports preserve the Studio HTTP route contract")
}

fn studio_benchmark_contract(
    description: &'static str,
    threshold_detail: &'static str,
    artifact_detail: &'static str,
) -> RustVerificationTaskContract {
    RustVerificationTaskContract::new(
        RustVerificationPhase::AfterUnitTestsPass,
        description,
        [
            RustVerificationRequirement::new("benchmark_command", STUDIO_BENCH_COMMAND),
            RustVerificationRequirement::new("baseline", "Criterion baseline name or commit"),
            RustVerificationRequirement::new("regression_threshold", threshold_detail),
            RustVerificationRequirement::new("latency_or_throughput", "Criterion latency result"),
            RustVerificationRequirement::new("profile_artifact", artifact_detail),
        ],
    )
}

fn security_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-security")
        .with_adapter("semgrep")
        .with_tool("semgrep")
        .with_command("semgrep scan --config <policy> <owner>")
        .with_standard("authorization and trust-boundary findings must be triaged")
        .with_required_inputs(["owner", "policy", "route_surface"])
        .with_pass_criteria(["exit=0", "findings=triaged"])
        .with_receipt_fields(["route_surface", "finding_summary", "artifact"])
}

rust_lang_project_harness::rust_project_harness_cargo_test_gate!(
    config = wendao_studio_harness_config()
);
