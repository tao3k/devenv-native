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
    let manifest_dir = client_manifest_dir();
    let config = client_rust_harness_config();
    let report = run_rust_project_harness_with_config(&manifest_dir, &config)
        .unwrap_or_else(|error| panic!("{error}"));

    report.assert_clean();
}

#[test]
fn client_verification_profile_hints_bind_active_skill_tasks() {
    let manifest_dir = client_manifest_dir();
    let config = client_rust_harness_config();
    let plan = plan_rust_project_verification_with_config(&manifest_dir, &config)
        .unwrap_or_else(|error| panic!("{error}"));
    let contracts = render_rust_verification_skill_contracts(&plan);

    assert_bound_task(
        &plan,
        "src/cli.rs",
        RustVerificationTaskKind::Security,
        "rust-verification-security@semgrep",
    );
    assert_bound_task(
        &plan,
        "src/get/run/facade.rs",
        RustVerificationTaskKind::Performance,
        "rust-verification-performance@criterion",
    );
    assert_bound_task(
        &plan,
        "src/orgize/read_model/store/materialize.rs",
        RustVerificationTaskKind::Performance,
        "rust-verification-performance@criterion",
    );
    assert_bound_task(
        &plan,
        "src/lint/contract/validation.rs",
        RustVerificationTaskKind::Regression,
        "rust-verification-regression@insta",
    );
    assert!(
        plan.tasks
            .iter()
            .all(|task| task.kind != RustVerificationTaskKind::ResponsibilityReview),
        "owner-local verification overrides must carry rationales: {:#?}",
        plan.tasks
            .iter()
            .filter(|task| task.kind == RustVerificationTaskKind::ResponsibilityReview)
            .collect::<Vec<_>>()
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

fn client_manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(super) fn client_rust_harness_config() -> RustHarnessConfig {
    default_rust_harness_config()
        .with_verification_profile_hint(cli_security_hint())
        .with_verification_profile_hint(get_runtime_performance_hint())
        .with_verification_profile_hint(orgize_read_model_performance_hint())
        .with_verification_profile_hint(lint_contract_regression_hint())
        .with_verification_responsibility_task_kinds(
            RustOwnerResponsibility::LatencySensitive,
            [RustVerificationTaskKind::Performance],
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

fn cli_security_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/cli.rs",
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
            "security skill must report CLI argument and output-boundary probes",
            [
                RustVerificationRequirement::new(
                    "cli_surface",
                    "client CLI command surface under verification",
                ),
                RustVerificationRequirement::new(
                    "output_boundary",
                    "client output boundary review result",
                ),
            ],
        ),
    )
    .with_rationale("client CLI exposes user-facing command and output boundaries")
}

fn get_runtime_performance_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/get/run/facade.rs",
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
            "performance skill must report get-runtime latency evidence from cargo bench -p xiuxian-wendao-client --features performance --bench wendao_client_get",
            [
                RustVerificationRequirement::new(
                    "benchmark_command",
                    "cargo bench -p xiuxian-wendao-client --features performance --bench wendao_client_get",
                ),
                RustVerificationRequirement::new("baseline", "Criterion baseline name or commit"),
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
    .with_rationale("get command execution crosses runtime and document-fetch boundaries")
}

fn orgize_read_model_performance_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/orgize/read_model/store/materialize.rs",
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
            "performance skill must report Org agent read-model refresh/query evidence from cargo bench -p xiuxian-wendao-client --features performance --bench wendao_client_orgize",
            [
                RustVerificationRequirement::new(
                    "benchmark_command",
                    "cargo bench -p xiuxian-wendao-client --features performance --bench wendao_client_orgize",
                ),
                RustVerificationRequirement::new("baseline", "Criterion baseline name or commit"),
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
    .with_rationale(
        "Org agent read-model refresh crosses parser, DuckDB write, and cached query boundaries",
    )
}

fn lint_contract_regression_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/lint/contract/validation.rs",
        [
            RustOwnerResponsibility::ExternalDependency,
            RustOwnerResponsibility::PublicApi,
        ],
    )
    .with_task_kinds([RustVerificationTaskKind::Regression])
    .with_task_contract(
        RustVerificationTaskKind::Regression,
        RustVerificationTaskContract::new(
            RustVerificationPhase::ScheduledRegression,
            "regression skill must report markdown-lint contract snapshot parity",
            [
                RustVerificationRequirement::new(
                    "snapshot_command",
                    "markdown-lint contract snapshot command",
                ),
                RustVerificationRequirement::new(
                    "contract_parity",
                    "contract snapshot parity result",
                ),
            ],
        ),
    )
    .with_rationale("lint contract validation owns stable client-facing diagnostics")
}

fn security_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-security")
        .with_adapter("semgrep")
        .with_tool("semgrep")
        .with_command("semgrep scan --config <policy> <owner>")
        .with_standard("user-facing command and output-boundary findings must be triaged")
        .with_required_inputs(["owner", "policy", "cli_surface"])
        .with_pass_criteria(["exit=0", "findings=triaged"])
        .with_receipt_fields([
            "cli_surface",
            "output_boundary",
            "finding_summary",
            "artifact",
        ])
}

fn regression_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-regression")
        .with_adapter("insta")
        .with_tool("cargo-insta")
        .with_command("cargo insta test -p xiuxian-wendao-client")
        .with_standard("contract snapshots stay intentional and reviewed")
        .with_required_inputs(["snapshot_command", "contract_surface"])
        .with_pass_criteria(["snapshots=accepted", "contract_parity=pass"])
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
        .join("xiuxian-wendao-client")
}

fn workspace_root_for_manifest(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .ancestors()
        .nth(4)
        .map_or_else(|| manifest_dir.to_path_buf(), Path::to_path_buf)
}
