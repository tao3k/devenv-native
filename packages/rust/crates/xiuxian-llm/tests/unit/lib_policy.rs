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
    let manifest_dir = llm_manifest_dir();
    let config = llm_rust_harness_config();
    let report = run_rust_project_harness_with_config(&manifest_dir, &config)
        .unwrap_or_else(|error| panic!("{error}"));

    report.assert_clean();
}

#[test]
fn llm_verification_profile_hints_bind_active_skill_tasks() {
    let manifest_dir = llm_manifest_dir();
    let config = llm_rust_harness_config();
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
        "src/llm/providers/openai_like/facade.rs",
        RustVerificationTaskKind::Security,
        "rust-verification-security@http-transport",
    );
    assert_bound_task(
        &plan,
        "src/runtime/bus.rs",
        RustVerificationTaskKind::Performance,
        "rust-verification-performance@cargo-test",
    );
    assert_bound_task(
        &plan,
        "src/embedding/runtime.rs",
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
        contracts.contains("[skill-contract] rust-verification-security@http-transport"),
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

fn llm_manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub(super) fn llm_rust_harness_config() -> RustHarnessConfig {
    default_rust_harness_config()
        .with_verification_profile_hint(openai_transport_hint())
        .with_verification_profile_hint(model_bus_performance_hint())
        .with_verification_profile_hint(embedding_runtime_regression_hint())
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
        .with_verification_responsibility_task_kinds(
            RustOwnerResponsibility::ExternalDependency,
            [RustVerificationTaskKind::Stress],
        )
        .with_verification_skill_binding(
            RustVerificationTaskKind::Security,
            RustVerificationSkillBinding::new("rust-verification-security")
                .with_adapter("http-transport"),
        )
        .with_verification_skill_descriptor(security_skill_descriptor())
        .with_verification_skill_binding(
            RustVerificationTaskKind::Stress,
            RustVerificationSkillBinding::new("rust-verification-stress")
                .with_adapter("cargo-test"),
        )
        .with_verification_skill_descriptor(stress_skill_descriptor())
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

fn openai_transport_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/llm/providers/openai_like/facade.rs",
        [
            RustOwnerResponsibility::AvailabilityCritical,
            RustOwnerResponsibility::ExternalDependency,
            RustOwnerResponsibility::PublicApi,
            RustOwnerResponsibility::SecurityBoundary,
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
            "security skill must report OpenAI-compatible transport auth, payload, and log-sanitization probes",
            [
                RustVerificationRequirement::new(
                    "auth_header_matrix",
                    "API-key header and skip-key behavior matrix",
                ),
                RustVerificationRequirement::new(
                    "payload_sanitization",
                    "request, response, and error log sanitization result",
                ),
            ],
        ),
    )
    .with_task_contract(
        RustVerificationTaskKind::Stress,
        RustVerificationTaskContract::new(
            RustVerificationPhase::BeforeRelease,
            "stress skill must report transient upstream retry and timeout behavior",
            [
                RustVerificationRequirement::new(
                    "retry_matrix",
                    "network, header-timeout, and retryable status matrix",
                ),
                RustVerificationRequirement::new(
                    "timeout_budget",
                    "configured timeout and retry backoff budget",
                ),
            ],
        ),
    )
    .with_rationale("OpenAI-compatible transport crosses external API and secret-bearing boundaries")
}

fn model_bus_performance_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/runtime/bus.rs",
        [
            RustOwnerResponsibility::AvailabilityCritical,
            RustOwnerResponsibility::LatencySensitive,
        ],
    )
    .with_task_kinds([RustVerificationTaskKind::Performance])
    .with_task_contract(
        RustVerificationTaskKind::Performance,
        RustVerificationTaskContract::new(
            RustVerificationPhase::AfterUnitTestsPass,
            "performance skill must report ModelBus activation and executor lifecycle evidence from cargo test -p xiuxian-llm --test unit_test -- --nocapture",
            [
                RustVerificationRequirement::new(
                    "benchmark_command",
                    "cargo test -p xiuxian-llm --test unit_test -- --nocapture",
                ),
                RustVerificationRequirement::new(
                    "baseline",
                    "ModelBus lifecycle baseline name or commit",
                ),
                RustVerificationRequirement::new(
                    "regression_threshold",
                    "accepted activation and memory accounting regression threshold",
                ),
                RustVerificationRequirement::new(
                    "latency_or_throughput",
                    "activation, execution, or memory-accounting result",
                ),
                RustVerificationRequirement::new(
                    "profile_artifact",
                    "unit test output or future benchmark artifact path",
                ),
            ],
        ),
    )
    .with_rationale("ModelBus owns activation, hibernation, and memory pressure behavior")
}

fn embedding_runtime_regression_hint() -> RustVerificationProfileHint {
    RustVerificationProfileHint::new(
        "src/embedding/runtime.rs",
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
            "regression skill must report embedding timeout, cooldown, unavailable, and dimension-repair parity",
            [
                RustVerificationRequirement::new(
                    "snapshot_command",
                    "embedding runtime regression command",
                ),
                RustVerificationRequirement::new(
                    "contract_parity",
                    "timeout, cooldown, unavailable, and repaired-vector parity result",
                ),
            ],
        ),
    )
    .with_rationale("embedding runtime guards semantic memory availability and vector shape")
}

fn security_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-security")
        .with_adapter("http-transport")
        .with_tool("cargo")
        .with_command(
            "cargo test -p xiuxian-llm --test unit_test llm_openai_responses_transport -- --nocapture",
        )
        .with_standard("secret-bearing HTTP transport must preserve auth and sanitized diagnostics")
        .with_required_inputs(["owner", "auth_header_matrix", "payload_sanitization"])
        .with_pass_criteria(["tests=pass", "secret_leak_cases=covered"])
        .with_receipt_fields([
            "auth_header_matrix",
            "payload_sanitization",
            "finding_summary",
            "artifact",
        ])
}

fn stress_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-stress")
        .with_adapter("cargo-test")
        .with_tool("cargo")
        .with_command(
            "cargo test -p xiuxian-llm --test unit_test llm_openai_responses_transport -- --nocapture",
        )
        .with_standard("external provider retry behavior stays bounded under transient failures")
        .with_required_inputs(["retry_matrix", "timeout_budget"])
        .with_pass_criteria(["tests=pass", "retry_budget=bounded"])
        .with_receipt_fields(["retry_matrix", "timeout_budget", "artifact"])
}

fn performance_skill_descriptor() -> RustVerificationSkillDescriptor {
    RustVerificationSkillDescriptor::new("rust-verification-performance")
        .with_adapter("cargo-test")
        .with_tool("cargo")
        .with_command("cargo test -p xiuxian-llm --test unit_test -- --nocapture")
        .with_standard("runtime bus activation and memory accounting stay observable and bounded")
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
        .with_command(
            "cargo test -p xiuxian-llm --test unit_test test_embedding_runtime -- --nocapture",
        )
        .with_standard("embedding runtime guard behavior stays intentional")
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
        .join("xiuxian-llm")
}

fn workspace_root_for_manifest(manifest_dir: &Path) -> PathBuf {
    manifest_dir
        .ancestors()
        .nth(4)
        .map_or_else(|| manifest_dir.to_path_buf(), Path::to_path_buf)
}
