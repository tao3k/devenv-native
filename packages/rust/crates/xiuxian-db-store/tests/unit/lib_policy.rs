//! Shared policy gate for `cargo test --lib`.

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
    let manifest_dir = db_store_manifest_dir();
    let config = db_store_rust_harness_config();
    let report = run_rust_project_harness_with_config(&manifest_dir, &config)
        .unwrap_or_else(|error| panic!("{error}"));

    report.assert_clean();
}

#[test]
fn db_store_verification_profile_hints_bind_active_skill_tasks() {
    let manifest_dir = db_store_manifest_dir();
    let config = db_store_rust_harness_config();
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
        "src/duckdb/sql.rs",
        RustVerificationTaskKind::Security,
        "rust-verification-security@semgrep",
    );
    assert_bound_task(
        &plan,
        "src/qianji_bpmn/store.rs",
        RustVerificationTaskKind::Performance,
        "rust-verification-performance@criterion",
    );
    assert_bound_task(
        &plan,
        "src/qianji_bpmn/state_log.rs",
        RustVerificationTaskKind::Regression,
        "rust-verification-regression@insta",
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
        contracts.contains("[skill-contract] rust-verification-regression@insta"),
        "{contracts}"
    );
    write_verification_reports_when_requested(&manifest_dir, &plan);
}

fn db_store_manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn db_store_rust_harness_config() -> RustHarnessConfig {
    default_rust_harness_config()
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/duckdb/sql.rs",
                [
                    RustOwnerResponsibility::PublicApi,
                    RustOwnerResponsibility::SecurityBoundary,
                ],
            )
            .with_task_kinds([RustVerificationTaskKind::Security])
            .with_task_contract(
                RustVerificationTaskKind::Security,
                RustVerificationTaskContract::new(
                    RustVerificationPhase::BeforeRelease,
                    "security skill must report DuckDB identifier and SQL-fragment boundary probes",
                    [
                        RustVerificationRequirement::new(
                            "identifier_escape_matrix",
                            "DuckDB identifier escape and rejection matrix",
                        ),
                        RustVerificationRequirement::new(
                            "sql_fragment_surface",
                            "SQL-fragment construction surface under verification",
                        ),
                    ],
                ),
            )
            .with_rationale("DuckDB SQL helpers guard the local storage SQL boundary"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/qianji_bpmn/store.rs",
                [
                    RustOwnerResponsibility::AvailabilityCritical,
                    RustOwnerResponsibility::ExternalDependency,
                    RustOwnerResponsibility::LatencySensitive,
                ],
            )
            .with_task_kinds([RustVerificationTaskKind::Performance])
            .with_task_contract(
                RustVerificationTaskKind::Performance,
                RustVerificationTaskContract::new(
                    RustVerificationPhase::AfterUnitTestsPass,
                    "performance skill must report BPMN DuckDB store latency evidence from cargo bench -p xiuxian-db-store --features qianji-bpmn-workflow-state --bench db_store_performance",
                    [
                        RustVerificationRequirement::new(
                            "benchmark_command",
                            "cargo bench -p xiuxian-db-store --features qianji-bpmn-workflow-state --bench db_store_performance",
                        ),
                        RustVerificationRequirement::new(
                            "baseline",
                            "storage latency baseline name or commit",
                        ),
                        RustVerificationRequirement::new(
                            "regression_threshold",
                            "accepted storage latency regression threshold",
                        ),
                        RustVerificationRequirement::new(
                            "latency_or_throughput",
                            "DuckDB save/load latency or throughput result",
                        ),
                        RustVerificationRequirement::new(
                            "profile_artifact",
                            "test output or benchmark artifact path",
                        ),
                    ],
                ),
            )
            .with_rationale("BPMN DuckDB store owns workflow-state persistence latency"),
        )
        .with_verification_profile_hint(
            RustVerificationProfileHint::new(
                "src/qianji_bpmn/state_log.rs",
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
                    "regression skill must report workflow-state latest/event-log parity",
                    [
                        RustVerificationRequirement::new(
                            "snapshot_command",
                            "workflow-state regression command",
                        ),
                        RustVerificationRequirement::new(
                            "contract_parity",
                            "latest-state and event-log parity result",
                        ),
                    ],
                ),
            )
            .with_rationale("workflow-state log owns replay and latest-state consistency"),
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
                .with_adapter("criterion"),
        )
        .with_verification_skill_descriptor(
            RustVerificationSkillDescriptor::criterion_performance(),
        )
        .with_verification_skill_binding(
            RustVerificationTaskKind::Security,
            RustVerificationSkillBinding::new("rust-verification-security").with_adapter("semgrep"),
        )
        .with_verification_skill_descriptor(security_skill_descriptor())
        .with_verification_skill_binding(
            RustVerificationTaskKind::Regression,
            RustVerificationSkillBinding::new("rust-verification-regression")
                .with_adapter("insta"),
        )
        .with_verification_skill_descriptor(regression_skill_descriptor())
}

fn security_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-security")
        .with_adapter("semgrep")
        .with_tool("semgrep")
        .with_command("semgrep scan --config <policy> <owner>")
        .with_standard("SQL boundary findings must be triaged before release")
        .with_required_inputs(["owner", "policy", "sql_fragment_surface"])
        .with_pass_criteria(["exit=0", "findings=triaged"])
        .with_receipt_fields([
            "identifier_escape_matrix",
            "sql_fragment_surface",
            "finding_summary",
            "artifact",
        ])
}

fn regression_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-regression")
        .with_adapter("insta")
        .with_tool("cargo")
        .with_command(
            "cargo test -p xiuxian-db-store --features qianji-bpmn-workflow-state qianji_bpmn",
        )
        .with_standard("workflow-state replay and latest-state contracts stay intentional")
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
        .join("xiuxian-db-store")
}
