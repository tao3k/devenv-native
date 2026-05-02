//! Shared policy gate for `cargo test --test unit_test`.

use std::env;
use std::path::{Path, PathBuf};

use rust_lang_project_harness::{
    RustHarnessConfig, RustOwnerResponsibility, RustVerificationPhase, RustVerificationProfileHint,
    RustVerificationReportWriteConfig, RustVerificationRequirement, RustVerificationSkillBinding,
    RustVerificationSkillDescriptor, RustVerificationTaskContract, RustVerificationTaskKind,
    build_rust_verification_profile_index_with_config, default_rust_harness_config,
    plan_rust_project_verification_with_config, render_rust_verification_skill_contracts,
    run_rust_project_harness_with_config, write_rust_verification_reports,
};

#[test]
fn enforce_rust_project_harness_gate() {
    let manifest_dir = memory_engine_manifest_dir();
    let config = memory_engine_rust_harness_config();
    let report = run_rust_project_harness_with_config(&manifest_dir, &config)
        .unwrap_or_else(|error| panic!("{error}"));

    report.assert_clean();
}

#[test]
fn memory_engine_verification_profile_hints_bind_active_skill_tasks() {
    let manifest_dir = memory_engine_manifest_dir();
    let config = memory_engine_rust_harness_config();
    let index = build_rust_verification_profile_index_with_config(&manifest_dir, &config)
        .unwrap_or_else(|error| panic!("{error}"));
    let plan = plan_rust_project_verification_with_config(&manifest_dir, &config)
        .unwrap_or_else(|error| panic!("{error}"));
    let contracts = render_rust_verification_skill_contracts(&plan);

    assert!(
        index
            .candidates_for_package(&manifest_dir)
            .iter()
            .all(|candidate| !candidate.hint_path.as_os_str().is_empty()),
        "profile index should expose compact parser-owned candidate paths"
    );
    assert_bound_task(
        &plan,
        "src/store.rs",
        RustVerificationTaskKind::Performance,
        "rust-verification-performance@cargo-test",
    );
    assert_bound_task(
        &plan,
        "src/gate.rs",
        RustVerificationTaskKind::Security,
        "rust-verification-security@memory-gate",
    );
    assert_bound_task(
        &plan,
        "src/two_phase.rs",
        RustVerificationTaskKind::Regression,
        "rust-verification-regression@cargo-test",
    );
    assert!(
        plan.tasks
            .iter()
            .all(|task| task.kind != RustVerificationTaskKind::ResponsibilityReview),
        "owner-local verification overrides must carry rationales"
    );
    assert!(
        contracts.contains("[skill-contract] rust-verification-performance@cargo-test"),
        "{contracts}"
    );
    assert!(
        contracts.contains("[skill-contract] rust-verification-security@memory-gate"),
        "{contracts}"
    );
    assert!(
        contracts.contains("[skill-contract] rust-verification-regression@cargo-test"),
        "{contracts}"
    );
    write_verification_reports_when_requested(&manifest_dir, &plan);
}

fn memory_engine_manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn memory_engine_rust_harness_config() -> RustHarnessConfig {
    default_rust_harness_config()
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/store.rs",
                [
                    RustOwnerResponsibility::AvailabilityCritical,
                    RustOwnerResponsibility::LatencySensitive,
                    RustOwnerResponsibility::PublicApi,
                ],
            )
            .with_task_kinds([RustVerificationTaskKind::Performance])
            .with_task_contract(
                RustVerificationTaskKind::Performance,
                RustVerificationTaskContract::new(
                    RustVerificationPhase::AfterUnitTestsPass,
                    "performance skill must report episodic recall, projection, and persistence evidence from cargo test -p xiuxian-memory-engine --test unit_test complex_scenarios::performance -- --nocapture",
                    [
                        RustVerificationRequirement::new(
                            "benchmark_command",
                            "cargo test -p xiuxian-memory-engine --test unit_test complex_scenarios::performance -- --nocapture",
                        ),
                        RustVerificationRequirement::new(
                            "baseline",
                            "memory recall performance baseline name or commit",
                        ),
                        RustVerificationRequirement::new(
                            "regression_threshold",
                            "accepted recall and persistence regression threshold",
                        ),
                        RustVerificationRequirement::new(
                            "latency_or_throughput",
                            "recall, projection, or persistence latency result",
                        ),
                        RustVerificationRequirement::new(
                            "profile_artifact",
                            "unit test output or future benchmark artifact path",
                        ),
                    ],
                ),
            )
            .with_rationale("EpisodeStore owns hot-path recall, projection, and memory state persistence"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/gate.rs",
                [
                    RustOwnerResponsibility::AvailabilityCritical,
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::SecurityBoundary,
                ],
            )
            .with_task_kinds([RustVerificationTaskKind::Security])
            .with_task_contract(
                RustVerificationTaskKind::Security,
                RustVerificationTaskContract::new(
                    RustVerificationPhase::BeforeRelease,
                    "security skill must report memory promotion, purge, and evidence-boundary probes",
                    [
                        RustVerificationRequirement::new(
                            "gate_decision_matrix",
                            "retain, obsolete, promote, confidence, and evidence matrix",
                        ),
                        RustVerificationRequirement::new(
                            "authority_boundary",
                            "memory promotion target and evidence-reference boundary result",
                        ),
                    ],
                ),
            )
            .with_rationale("MemoryGate controls promotion and purge decisions for retained knowledge"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/two_phase.rs",
                [
                    RustOwnerResponsibility::AvailabilityCritical,
                    RustOwnerResponsibility::PublicApi,
                ],
            )
            .with_task_kinds([RustVerificationTaskKind::Regression])
            .with_task_contract(
                RustVerificationTaskKind::Regression,
                RustVerificationTaskContract::new(
                    RustVerificationPhase::ScheduledRegression,
                    "regression skill must report two-phase semantic recall and Q-rerank parity",
                    [
                        RustVerificationRequirement::new(
                            "snapshot_command",
                            "two-phase recall regression command",
                        ),
                        RustVerificationRequirement::new(
                            "contract_parity",
                            "semantic candidate, Q-value, lambda, and ordering parity result",
                        ),
                    ],
                ),
            )
            .with_rationale("TwoPhaseSearch determines memory recall ordering from semantic and Q signals"),
        )
        .with_verification_responsibility_task_kinds(
            RustOwnerResponsibility::LatencySensitive,
            [RustVerificationTaskKind::Performance],
        )
        .with_verification_responsibility_task_kinds(
            RustOwnerResponsibility::SecurityBoundary,
            [RustVerificationTaskKind::Security],
        )
        .with_verification_responsibility_task_kinds(
            RustOwnerResponsibility::AvailabilityCritical,
            [RustVerificationTaskKind::Regression],
        )
        .with_verification_skill_binding(
            RustVerificationTaskKind::Performance,
            RustVerificationSkillBinding::new("rust-verification-performance")
                .with_adapter("cargo-test"),
        )
        .with_verification_skill_descriptor(performance_skill_descriptor())
        .with_verification_skill_binding(
            RustVerificationTaskKind::Security,
            RustVerificationSkillBinding::new("rust-verification-security")
                .with_adapter("memory-gate"),
        )
        .with_verification_skill_descriptor(security_skill_descriptor())
        .with_verification_skill_binding(
            RustVerificationTaskKind::Regression,
            RustVerificationSkillBinding::new("rust-verification-regression")
                .with_adapter("cargo-test"),
        )
        .with_verification_skill_descriptor(regression_skill_descriptor())
}

fn performance_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-performance")
        .with_adapter("cargo-test")
        .with_tool("cargo")
        .with_command(
            "cargo test -p xiuxian-memory-engine --test unit_test complex_scenarios::performance -- --nocapture",
        )
        .with_standard("episodic memory recall and persistence stay within configured guardrails")
        .with_required_inputs(["benchmark_command", "baseline", "regression_threshold"])
        .with_pass_criteria(["tests=pass", "latency_or_throughput=reported"])
        .with_receipt_fields([
            "benchmark_command",
            "baseline",
            "regression_threshold",
            "latency_or_throughput",
            "profile_artifact",
        ])
}

fn security_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-security")
        .with_adapter("memory-gate")
        .with_tool("cargo")
        .with_command("cargo test -p xiuxian-memory-engine --test unit_test gate -- --nocapture")
        .with_standard("memory gate decisions must remain evidence-bound and deterministic")
        .with_required_inputs(["owner", "gate_decision_matrix", "authority_boundary"])
        .with_pass_criteria(["tests=pass", "promotion_boundary=covered"])
        .with_receipt_fields([
            "gate_decision_matrix",
            "authority_boundary",
            "finding_summary",
            "artifact",
        ])
}

fn regression_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-regression")
        .with_adapter("cargo-test")
        .with_tool("cargo")
        .with_command(
            "cargo test -p xiuxian-memory-engine --test unit_test two_phase -- --nocapture",
        )
        .with_standard("two-phase recall ranking stays intentional")
        .with_required_inputs(["snapshot_command", "contract_surface"])
        .with_pass_criteria(["tests=pass", "contract_parity=pass"])
        .with_receipt_fields(["snapshot_command", "contract_parity", "artifact"])
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
        .join("xiuxian-memory-engine")
}
