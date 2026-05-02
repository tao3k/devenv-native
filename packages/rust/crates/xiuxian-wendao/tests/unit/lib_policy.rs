use std::env;
use std::path::{Path, PathBuf};

use rust_lang_project_harness::{
    RustHarnessConfig, RustOwnerResponsibility, RustVerificationPhase, RustVerificationProfileHint,
    RustVerificationReportWriteConfig, RustVerificationRequirement, RustVerificationSkillBinding,
    RustVerificationSkillDescriptor, RustVerificationTaskContract, RustVerificationTaskKind,
    default_rust_harness_config, plan_rust_project_verification_with_config,
    render_rust_verification_skill_contracts, run_rust_project_harness_with_config,
    write_rust_verification_reports,
};

#[test]
fn enforce_rust_project_harness_gate() {
    let manifest_dir = wendao_manifest_dir();
    let config = wendao_rust_harness_config();
    let report = run_rust_project_harness_with_config(&manifest_dir, &config)
        .unwrap_or_else(|error| panic!("{error}"));

    report.assert_clean();
}

#[test]
fn wendao_verification_profile_hints_bind_active_skill_tasks() {
    let manifest_dir = wendao_manifest_dir();
    let config = wendao_rust_harness_config();
    let plan = plan_rust_project_verification_with_config(&manifest_dir, &config)
        .unwrap_or_else(|error| panic!("{error}"));
    let contracts = render_rust_verification_skill_contracts(&plan);

    assert_bound_task(
        &plan,
        "src/gateway/studio/mod.rs",
        RustVerificationTaskKind::Security,
        "rust-verification-security@semgrep",
    );
    assert_bound_task(
        &plan,
        "src/query_core/service.rs",
        RustVerificationTaskKind::Performance,
        "rust-verification-performance@criterion",
    );
    assert_bound_task(
        &plan,
        "src/search/perf_support.rs",
        RustVerificationTaskKind::Performance,
        "rust-verification-performance@criterion",
    );
    assert_bound_task(
        &plan,
        "src/gateway/studio/mod.rs",
        RustVerificationTaskKind::Stress,
        "rust-verification-stress@k6",
    );
    assert!(
        plan.tasks
            .iter()
            .all(|task| task.kind != RustVerificationTaskKind::ResponsibilityReview),
        "owner-local verification overrides must carry rationales"
    );
    assert!(
        contracts.contains("[skill-contract] rust-verification-security@semgrep"),
        "{contracts}"
    );
    assert!(
        contracts.contains("[skill-contract] rust-verification-performance@criterion"),
        "{contracts}"
    );
    assert!(
        contracts.contains("[skill-contract] rust-verification-stress@k6"),
        "{contracts}"
    );
    write_verification_reports_when_requested(&manifest_dir, &plan);
}

fn wendao_manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn wendao_rust_harness_config() -> RustHarnessConfig {
    default_rust_harness_config()
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/gateway/studio/mod.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::SecurityBoundary,
                    RustOwnerResponsibility::AvailabilityCritical,
                ],
            )
            .with_task_kinds([
                RustVerificationTaskKind::Security,
                RustVerificationTaskKind::Stress,
            ])
            .with_task_contract(
                RustVerificationTaskKind::Security,
                RustVerificationTaskContract::new(
                    RustVerificationPhase::BeforeRelease,
                    "security skill must report Studio gateway authorization and route probes",
                    [
                        RustVerificationRequirement::new(
                            "gateway_authz_matrix",
                            "Studio gateway authorization matrix",
                        ),
                        RustVerificationRequirement::new(
                            "route_surface",
                            "Studio gateway route surface under verification",
                        ),
                    ],
                ),
            )
            .with_task_contract(
                RustVerificationTaskKind::Stress,
                RustVerificationTaskContract::new(
                    RustVerificationPhase::BeforeRelease,
                    "stress skill must report Studio gateway pressure and SLA evidence",
                    [
                        RustVerificationRequirement::new(
                            "sla_result",
                            "gateway pressure SLA result",
                        ),
                        RustVerificationRequirement::new("load_steps", "gateway load steps"),
                    ],
                ),
            )
            .with_rationale("Studio gateway owns public route and service-boundary behavior"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/query_core/service.rs",
                [
                    RustOwnerResponsibility::ExternalDependency,
                    RustOwnerResponsibility::LatencySensitive,
                ],
            )
            .with_task_kinds([RustVerificationTaskKind::Performance])
            .with_task_contract(
                RustVerificationTaskKind::Performance,
                RustVerificationTaskContract::new(
                    RustVerificationPhase::AfterUnitTestsPass,
                    "Criterion benchmark must report query service latency evidence from cargo bench -p xiuxian-wendao --features performance --bench wendao_performance",
                    [
                        RustVerificationRequirement::new(
                            "benchmark_command",
                            "cargo bench -p xiuxian-wendao --features performance --bench wendao_performance",
                        ),
                        RustVerificationRequirement::new(
                            "baseline",
                            "Criterion baseline name or commit",
                        ),
                        RustVerificationRequirement::new(
                            "regression_threshold",
                            "accepted latency regression threshold",
                        ),
                        RustVerificationRequirement::new(
                            "latency_or_throughput",
                            "Criterion latency or throughput result",
                        ),
                        RustVerificationRequirement::new(
                            "profile_artifact",
                            "target/criterion artifact path for the relevant benchmark group",
                        ),
                    ],
                ),
            )
            .with_rationale("query service is the high-frequency retrieval execution path"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/search/perf_support.rs",
                [RustOwnerResponsibility::LatencySensitive],
            )
            .with_task_kinds([RustVerificationTaskKind::Performance])
            .with_task_contract(
                RustVerificationTaskKind::Performance,
                RustVerificationTaskContract::new(
                    RustVerificationPhase::AfterUnitTestsPass,
                    "Criterion benchmark must report search latency evidence from cargo bench -p xiuxian-wendao --features performance --bench wendao_performance",
                    [
                        RustVerificationRequirement::new(
                            "benchmark_command",
                            "cargo bench -p xiuxian-wendao --features performance --bench wendao_performance",
                        ),
                        RustVerificationRequirement::new(
                            "baseline",
                            "Criterion baseline name or commit",
                        ),
                        RustVerificationRequirement::new(
                            "regression_threshold",
                            "accepted latency regression threshold",
                        ),
                        RustVerificationRequirement::new(
                            "latency_or_throughput",
                            "Criterion latency or throughput result",
                        ),
                        RustVerificationRequirement::new(
                            "profile_artifact",
                            "target/criterion artifact path for the relevant benchmark group",
                        ),
                    ],
                ),
            )
            .with_rationale("search perf support defines latency-sensitive benchmark ownership"),
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
        .with_verification_skill_descriptor(security_skill_descriptor())
}

fn security_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-security")
        .with_adapter("semgrep")
        .with_tool("semgrep")
        .with_command("semgrep scan --config <policy> <owner>")
        .with_standard("authorization and trust-boundary findings must be triaged before release")
        .with_required_inputs(["owner", "policy", "route_surface"])
        .with_pass_criteria(["exit=0", "findings=triaged"])
        .with_receipt_fields([
            "gateway_authz_matrix",
            "route_surface",
            "finding_summary",
            "artifact",
        ])
}

fn assert_bound_task(
    plan: &rust_lang_project_harness::RustVerificationPlan,
    owner_path: &str,
    kind: RustVerificationTaskKind,
    expected_binding: &str,
) {
    let task = plan
        .active_tasks()
        .into_iter()
        .find(|task| task.kind == kind && task.owner_path.ends_with(Path::new(owner_path)))
        .unwrap_or_else(|| panic!("missing {kind:?} task for {owner_path}: {plan:#?}"));
    let binding = task
        .skill_binding
        .as_ref()
        .unwrap_or_else(|| panic!("missing skill binding for {kind:?} task"));
    let binding_label = binding.adapter.as_ref().map_or_else(
        || binding.skill_id.clone(),
        |adapter| format!("{}@{adapter}", binding.skill_id),
    );

    assert_eq!(binding_label, expected_binding);
    assert_eq!(task.skill_contract_ref.as_deref(), Some(expected_binding));
}

fn write_verification_reports_when_requested(
    manifest_dir: &Path,
    plan: &rust_lang_project_harness::RustVerificationPlan,
) {
    if env::var_os("XIUXIAN_WRITE_VERIFICATION_REPORTS").is_none() {
        return;
    }

    let source_dir = verification_source_report_output_dir(manifest_dir);
    let cache_dir = verification_cache_report_output_dir(manifest_dir);
    write_rust_verification_reports(
        plan,
        &RustVerificationReportWriteConfig::new(manifest_dir, source_dir, cache_dir),
    )
    .expect("write verification reports");
}

fn verification_source_report_output_dir(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .join("resources")
        .join("verification")
        .join("reports")
}

fn verification_cache_report_output_dir(manifest_dir: &Path) -> PathBuf {
    let project_root = env::var_os("PRJ_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            manifest_dir
                .ancestors()
                .nth(4)
                .expect("workspace root")
                .to_path_buf()
        });
    let cache_home = env::var_os("PRJ_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".cache"));
    let cache_home = if cache_home.is_absolute() {
        cache_home
    } else {
        project_root.join(cache_home)
    };
    cache_home
        .join("agent")
        .join("verification")
        .join("xiuxian-wendao")
}
