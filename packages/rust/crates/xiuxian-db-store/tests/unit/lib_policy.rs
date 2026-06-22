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
    assert_bound_verification_tasks(&plan);
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
    assert_verification_task_requirements(&plan);
    write_verification_reports_when_requested(&manifest_dir, &plan);
}

fn assert_bound_verification_tasks(plan: &rust_lang_project_harness::RustVerificationPlan) {
    for (owner_path, kind, expected_binding) in [
        (
            "src/duckdb/sql.rs",
            RustVerificationTaskKind::Security,
            "rust-verification-security@semgrep",
        ),
        (
            "src/duckdb/ducklake/mod.rs",
            RustVerificationTaskKind::Regression,
            "rust-verification-regression@insta",
        ),
        (
            "src/duckdb/ducklake/mod.rs",
            RustVerificationTaskKind::Performance,
            "rust-verification-performance@criterion",
        ),
        (
            "src/qianji_bpmn/store.rs",
            RustVerificationTaskKind::Performance,
            "rust-verification-performance@criterion",
        ),
        (
            "src/qianji_bpmn/state_log.rs",
            RustVerificationTaskKind::Regression,
            "rust-verification-regression@insta",
        ),
        (
            "src/valkey/mod.rs",
            RustVerificationTaskKind::Regression,
            "rust-verification-regression@insta",
        ),
        (
            "src/valkey/mod.rs",
            RustVerificationTaskKind::Performance,
            "rust-verification-performance@criterion",
        ),
    ] {
        assert_bound_task(plan, owner_path, kind, expected_binding);
    }
}

fn assert_verification_task_requirements(plan: &rust_lang_project_harness::RustVerificationPlan) {
    for (owner_path, kind, expected_key, expected_fragment) in [
        (
            "src/duckdb/ducklake/mod.rs",
            RustVerificationTaskKind::Regression,
            "local_live_smoke_command",
            "ducklake_live_attach_smoke",
        ),
        (
            "src/duckdb/ducklake/mod.rs",
            RustVerificationTaskKind::Performance,
            "benchmark_command",
            "db_store_ducklake_arrow_appender",
        ),
        (
            "src/valkey/mod.rs",
            RustVerificationTaskKind::Regression,
            "default_test_command",
            "cargo test -p xiuxian-db-store --features valkey valkey",
        ),
        (
            "src/valkey/mod.rs",
            RustVerificationTaskKind::Performance,
            "benchmark_command",
            "db_store_valkey_hot_queue",
        ),
    ] {
        assert_task_requirement(plan, owner_path, kind, expected_key, expected_fragment);
    }
}

fn db_store_manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(super) fn db_store_rust_harness_config() -> RustHarnessConfig {
    default_rust_harness_config()
        .with_verification_profile_hint(duckdb_sql_security_hint())
        .with_verification_profile_hint(ducklake_chain_regression_hint())
        .with_verification_profile_hint(bpmn_store_performance_hint())
        .with_verification_profile_hint(state_log_regression_hint())
        .with_verification_profile_hint(valkey_hot_queue_regression_hint())
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

fn duckdb_sql_security_hint() -> RustVerificationProfileHint {
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
    .with_rationale("DuckDB SQL helpers guard the local storage SQL boundary")
}

fn ducklake_chain_regression_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/duckdb/ducklake/mod.rs",
        [
            RustOwnerResponsibility::AvailabilityCritical,
            RustOwnerResponsibility::ExternalDependency,
            RustOwnerResponsibility::PublicApi,
        ],
    )
    .with_task_kinds([
        RustVerificationTaskKind::Regression,
        RustVerificationTaskKind::Performance,
    ])
    .with_task_contract(
        RustVerificationTaskKind::Regression,
        RustVerificationTaskContract::new(
            RustVerificationPhase::AfterUnitTestsPass,
            "regression skill must report the DuckLake attach, catalog, secret, appender, local-live, and external-probe chain",
            [
                RustVerificationRequirement::new(
                    "default_test_command",
                    "cargo test -p xiuxian-db-store --features duckdb -- --nocapture",
                ),
                RustVerificationRequirement::new(
                    "local_live_smoke_command",
                    "cargo test -p xiuxian-db-store --features duckdb ducklake_live_attach_smoke -- --ignored --nocapture",
                ),
                RustVerificationRequirement::new(
                    "external_probe_command",
                    "cargo test -p xiuxian-db-store --features duckdb ducklake_external -- --ignored --nocapture",
                ),
                RustVerificationRequirement::new(
                    "chain_contract",
                    "local metadata, typed local or remote data path, S3 secret SQL, Arrow appender, and skip/live probe semantics",
                ),
                RustVerificationRequirement::new(
                    "profile_artifact",
                    "test output or verification report path",
                ),
            ],
        ),
    )
    .with_task_contract(
        RustVerificationTaskKind::Performance,
        RustVerificationTaskContract::new(
            RustVerificationPhase::AfterUnitTestsPass,
            "performance skill must report the reusable DuckLake Arrow appender benchmark",
            [
                RustVerificationRequirement::new(
                    "benchmark_command",
                    "cargo bench -p xiuxian-db-store --features duckdb --bench db_store_performance db_store_ducklake_arrow_appender",
                ),
                RustVerificationRequirement::new(
                    "baseline",
                    "DuckLake Arrow appender baseline name or commit",
                ),
                RustVerificationRequirement::new(
                    "regression_threshold",
                    "accepted reusable appender throughput regression threshold",
                ),
                RustVerificationRequirement::new(
                    "latency_or_throughput",
                    "rows-per-second or batch append latency from Criterion output",
                ),
                RustVerificationRequirement::new(
                    "profile_artifact",
                    "Criterion output or verification report path",
                ),
            ],
        ),
    )
    .with_rationale(
        "DuckLake substrate owns the embedded attach/appender/external-probe chain and reusable appender throughput used by downstream event lakes",
    )
}

fn bpmn_store_performance_hint() -> RustVerificationProfileHint {
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
    .with_rationale("BPMN DuckDB store owns workflow-state persistence latency")
}

fn state_log_regression_hint() -> RustVerificationProfileHint {
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
    .with_rationale("workflow-state log owns replay and latest-state consistency")
}

fn valkey_hot_queue_regression_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/valkey/mod.rs",
        [
            RustOwnerResponsibility::AvailabilityCritical,
            RustOwnerResponsibility::ExternalDependency,
            RustOwnerResponsibility::LatencySensitive,
        ],
    )
    .with_task_kinds([
        RustVerificationTaskKind::Regression,
        RustVerificationTaskKind::Performance,
    ])
    .with_task_contract(
        RustVerificationTaskKind::Regression,
        RustVerificationTaskContract::new(
            RustVerificationPhase::AfterUnitTestsPass,
            "regression skill must report structured Valkey queue key, filter, TTL, lease ownership, and no-domain-key-parsing contracts",
            [
                RustVerificationRequirement::new(
                    "default_test_command",
                    "cargo test -p xiuxian-db-store --features valkey valkey",
                ),
                RustVerificationRequirement::new(
                    "structured_queue_contract",
                    "explicit queue fields, typed payloads, atomic claim/renew/release/reclaim, and TTL validation before connect",
                ),
                RustVerificationRequirement::new(
                    "domain_boundary",
                    "domain crates own payload schema and durable truth; db-store owns Valkey command mechanics",
                ),
            ],
        ),
    )
    .with_task_contract(
        RustVerificationTaskKind::Performance,
        RustVerificationTaskContract::new(
            RustVerificationPhase::AfterUnitTestsPass,
            "performance skill must report Valkey hot queue lease throughput once live Valkey benchmark coverage is enabled",
            [
                RustVerificationRequirement::new(
                    "benchmark_command",
                    "cargo bench -p xiuxian-db-store --features valkey --bench db_store_performance db_store_valkey_hot_queue",
                ),
                RustVerificationRequirement::new(
                    "baseline",
                    "Valkey hot queue throughput baseline name or commit",
                ),
                RustVerificationRequirement::new(
                    "regression_threshold",
                    "accepted enqueue/claim/release throughput regression threshold",
                ),
                RustVerificationRequirement::new(
                    "latency_or_throughput",
                    "operations per second or p95 lease claim latency",
                ),
            ],
        ),
    )
    .with_rationale("Valkey hot queues own live scheduling latency for Qianji server workers")
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
        .with_command("cargo test -p xiuxian-db-store <profile-specific regression filter>")
        .with_standard("db-store regression contracts stay intentional")
        .with_required_inputs(["snapshot_command", "contract_surface"])
        .with_pass_criteria(["tests=pass", "regression_evidence=present"])
        .with_receipt_fields(["snapshot_command", "regression_evidence", "artifact"])
}

fn assert_bound_task(
    plan: &rust_lang_project_harness::RustVerificationPlan,
    owner_path: &str,
    kind: RustVerificationTaskKind,
    expected_binding: &str,
) {
    let task = find_active_task(plan, owner_path, kind);
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

fn assert_task_requirement(
    plan: &rust_lang_project_harness::RustVerificationPlan,
    owner_path: &str,
    kind: RustVerificationTaskKind,
    expected_key: &str,
    expected_description_fragment: &str,
) {
    let (task, requirement) = find_active_task_requirement(plan, owner_path, kind, expected_key);

    assert!(
        requirement
            .description
            .contains(expected_description_fragment),
        "requirement `{expected_key}` should include `{expected_description_fragment}`: {requirement:#?}"
    );
    assert!(
        task.owner_path.ends_with(Path::new(owner_path)),
        "matched task should belong to {owner_path}: {task:#?}"
    );
}

fn find_active_task_requirement<'a>(
    plan: &'a rust_lang_project_harness::RustVerificationPlan,
    owner_path: &str,
    kind: RustVerificationTaskKind,
    expected_key: &str,
) -> (
    &'a rust_lang_project_harness::RustVerificationTask,
    &'a rust_lang_project_harness::RustVerificationRequirement,
) {
    plan.active_tasks()
        .into_iter()
        .filter(|task| task.kind == kind && task.owner_path.ends_with(Path::new(owner_path)))
        .find_map(|task| {
            task.required_evidence
                .iter()
                .find(|requirement| requirement.key == expected_key)
                .map(|requirement| (task, requirement))
        })
        .unwrap_or_else(|| {
            panic!("missing requirement {expected_key} for {kind:?} task at {owner_path}")
        })
}

fn find_active_task<'a>(
    plan: &'a rust_lang_project_harness::RustVerificationPlan,
    owner_path: &str,
    kind: RustVerificationTaskKind,
) -> &'a rust_lang_project_harness::RustVerificationTask {
    plan.active_tasks()
        .into_iter()
        .find(|task| task.kind == kind && task.owner_path.ends_with(Path::new(owner_path)))
        .unwrap_or_else(|| panic!("missing {kind:?} task for {owner_path}: {plan:#?}"))
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
    .unwrap_or_else(|error| panic!("write verification reports: {error}"));
}

fn verification_source_report_output_dir(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .join("resources")
        .join("verification")
        .join("reports")
}

fn verification_cache_report_output_dir(manifest_dir: &Path) -> PathBuf {
    let project_root = env::var_os("PRJ_ROOT")
        .map_or_else(|| workspace_root_for_manifest(manifest_dir), PathBuf::from);
    let cache_home =
        env::var_os("PRJ_CACHE_HOME").map_or_else(|| PathBuf::from(".cache"), PathBuf::from);
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

fn workspace_root_for_manifest(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .ancestors()
        .nth(4)
        .map_or_else(|| manifest_dir.to_path_buf(), Path::to_path_buf)
}
