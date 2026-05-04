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
    let manifest_dir = vector_manifest_dir();
    let config = vector_rust_harness_config();
    let report = run_rust_project_harness_with_config(&manifest_dir, &config)
        .unwrap_or_else(|error| panic!("{error}"));

    report.assert_clean();
}

#[test]
fn vector_verification_profile_hints_bind_active_skill_tasks() {
    let manifest_dir = vector_manifest_dir();
    let config = vector_rust_harness_config();
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
        "src/search/search_impl/filter.rs",
        RustVerificationTaskKind::Security,
        "rust-verification-security@filter-sql",
    );
    assert_bound_task(
        &plan,
        "src/search/search_impl/mod.rs",
        RustVerificationTaskKind::Performance,
        "rust-verification-performance@cargo-test",
    );
    assert_bound_task(
        &plan,
        "src/ops/migration.rs",
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
        contracts.contains("[skill-contract] rust-verification-security@filter-sql"),
        "{contracts}"
    );
    assert!(
        contracts.contains("[skill-contract] rust-verification-performance@cargo-test"),
        "{contracts}"
    );
    assert!(
        contracts.contains("[skill-contract] rust-verification-regression@cargo-test"),
        "{contracts}"
    );
    write_verification_reports_when_requested(&manifest_dir, &plan);
}

fn vector_manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(super) fn vector_rust_harness_config() -> RustHarnessConfig {
    default_rust_harness_config()
        .with_verification_profile_hint(filter_security_hint())
        .with_verification_profile_hint(search_performance_hint())
        .with_verification_profile_hint(migration_regression_hint())
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
            RustVerificationTaskKind::Security,
            RustVerificationSkillBinding::new("rust-verification-security")
                .with_adapter("filter-sql"),
        )
        .with_verification_skill_descriptor(security_skill_descriptor())
        .with_verification_skill_binding(
            RustVerificationTaskKind::Performance,
            RustVerificationSkillBinding::new("rust-verification-performance")
                .with_adapter("cargo-test"),
        )
        .with_verification_skill_descriptor(performance_skill_descriptor())
        .with_verification_skill_binding(
            RustVerificationTaskKind::Regression,
            RustVerificationSkillBinding::new("rust-verification-regression")
                .with_adapter("cargo-test"),
        )
        .with_verification_skill_descriptor(regression_skill_descriptor())
}

fn filter_security_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/search/search_impl/filter.rs",
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
            "security skill must report JSON-to-Lance filter escaping and predicate-surface probes",
            [
                RustVerificationRequirement::new(
                    "filter_escape_matrix",
                    "JSON string, identifier, and comparator escaping matrix",
                ),
                RustVerificationRequirement::new(
                    "metadata_predicate_surface",
                    "metadata predicate conversion surface under verification",
                ),
            ],
        ),
    )
    .with_rationale("JSON filter conversion builds Lance SQL-like predicates")
}

fn search_performance_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/search/search_impl/mod.rs",
        [
            RustOwnerResponsibility::LatencySensitive,
            RustOwnerResponsibility::PublicApi,
        ],
    )
    .with_task_kinds([RustVerificationTaskKind::Performance])
    .with_task_contract(
        RustVerificationTaskKind::Performance,
        RustVerificationTaskContract::new(
            RustVerificationPhase::AfterUnitTestsPass,
            "performance skill must report vector search latency evidence from cargo test -p xiuxian-vector --test performance_test -- --nocapture",
            [
                RustVerificationRequirement::new(
                    "benchmark_command",
                    "cargo test -p xiuxian-vector --test performance_test -- --nocapture",
                ),
                RustVerificationRequirement::new(
                    "baseline",
                    "vector search latency baseline name or commit",
                ),
                RustVerificationRequirement::new(
                    "regression_threshold",
                    "accepted vector search latency regression threshold",
                ),
                RustVerificationRequirement::new(
                    "latency_or_throughput",
                    "search_optimized p50, p95, average latency, or throughput result",
                ),
                RustVerificationRequirement::new(
                    "profile_artifact",
                    "test output or benchmark artifact path",
                ),
            ],
        ),
    )
    .with_rationale("search execution owns vector retrieval latency")
}

fn migration_regression_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/ops/migration.rs",
        [
            RustOwnerResponsibility::AvailabilityCritical,
            RustOwnerResponsibility::ExternalDependency,
        ],
    )
    .with_task_kinds([RustVerificationTaskKind::Regression])
    .with_task_contract(
        RustVerificationTaskKind::Regression,
        RustVerificationTaskContract::new(
            RustVerificationPhase::ScheduledRegression,
            "regression skill must report Lance schema migration and table compatibility evidence",
            [
                RustVerificationRequirement::new(
                    "snapshot_command",
                    "schema migration regression command",
                ),
                RustVerificationRequirement::new(
                    "contract_parity",
                    "schema version and migrated-row parity result",
                ),
            ],
        ),
    )
    .with_rationale("Lance schema migration preserves persisted vector tables")
}

fn security_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-security")
        .with_adapter("filter-sql")
        .with_tool("cargo")
        .with_command("cargo test -p xiuxian-vector --test unit_test filter_expr -- --nocapture")
        .with_standard("filter predicates must preserve intended metadata boundaries")
        .with_required_inputs([
            "owner",
            "filter_escape_matrix",
            "metadata_predicate_surface",
        ])
        .with_pass_criteria(["tests=pass", "injection_cases=covered"])
        .with_receipt_fields([
            "filter_escape_matrix",
            "metadata_predicate_surface",
            "finding_summary",
            "artifact",
        ])
}

fn performance_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-performance")
        .with_adapter("cargo-test")
        .with_tool("cargo")
        .with_command("cargo test -p xiuxian-vector --test performance_test -- --nocapture")
        .with_standard("vector retrieval latency remains within configured guardrails")
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

fn regression_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-regression")
        .with_adapter("cargo-test")
        .with_tool("cargo")
        .with_command("cargo test -p xiuxian-vector --test unit_test -- --nocapture")
        .with_standard("schema migration and search conversion contracts stay intentional")
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
        .join("xiuxian-vector")
}

fn workspace_root_for_manifest(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .ancestors()
        .nth(4)
        .map_or_else(|| manifest_dir.to_path_buf(), Path::to_path_buf)
}
