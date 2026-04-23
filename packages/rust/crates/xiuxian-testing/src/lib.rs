//! Shared testing utilities for xiuxian crates.
//!
//! This crate provides:
//! - **Contract Kernel**: Shared findings, reports, rule-pack interfaces, and advisory audit types
//! - **Scenario Framework**: A reusable framework for scenario-based snapshot testing
//! - **Test Structure Validation**: Utilities to enforce test directory conventions
//! - **External Test Support**: Convention and policy validation for externalized tests via `#[path]`
//! - **Test Utilities**: Common helpers for test setup and assertions
//!
//! # Scenario Framework
//!
//! The scenario framework allows you to define test scenarios as directories with
//! input files and configuration. It uses Insta for snapshot testing and exposes
//! a shared snapshot policy for deterministic ordering, rich metadata, and
//! reusable redactions. The recommended policy also includes shared stability
//! presets for portable paths and common runtime-volatility fields. For fixture
//! roots that should not leak into snapshot headers, use `portable_ci()`. For
//! runtime-heavy suites, use `runtime_heavy()`.
//!
//! ## Directory Structure
//!
//! ```text
//! tests/
//! ├── scenarios/                    # Scenario definitions (you write)
//! │   └── 001_page_index_hierarchy/
//! │       ├── scenario.toml         # Metadata + assertions
//! │       └── input/                # Input files
//! ├── unit/                         # Unit tests (*.rs, snake_case)
//! ├── integration/                  # Integration tests (*.rs, snake_case)
//! └── snapshots/                    # Snapshots (insta manages)
//!     └── scenarios__*.snap
//! ```
//!
//! # External Test Module Pattern
//!
//! Keep tests out of source files using `#[path]` mounting. The shared validation path also treats
//! inline `#[cfg(test)]` modules as policy violations so crates can fail fast when tests drift back
//! into `src/`.
//! For enforcement inside regular crate test targets, prefer
//! [`crate_test_policy_harness!`](macro@crate_test_policy_harness), which makes
//! narrow `cargo test --test <target>` runs execute the policy gate too.
//! For a dedicated root-level `tests/xiuxian-testing-gate.rs` target that should
//! run both the crate test policy and the built-in modularity contract, prefer
//! [`crate_testing_gate!`](macro@crate_testing_gate).
//! For source-backed unit tests that should participate in `cargo test --lib`
//! without keeping test bodies inline in `src/`, mount
//! [`crate_testing_gate!`](macro@crate_testing_gate) through
//! [`crate_test_policy_source_harness!`](macro@crate_test_policy_source_harness).
//!
//! ```ignore
//! // src/foo/bar.rs
//! fn business_logic() { ... }
//!
//! #[cfg(test)]
//! #[path = "../../tests/unit/foo/bar.rs"]
//! mod tests;
//! ```
//!
//! ```ignore
//! // src/lib.rs
//! xiuxian_testing::crate_test_policy_source_harness!("../tests/unit/lib_policy.rs");
//!
//! // tests/unit/lib_policy.rs
//! xiuxian_testing::crate_testing_gate!();
//! ```
//!
//! ## Example
//!
//! ```ignore
//! use xiuxian_testing::{
//!     Scenario, ScenarioFramework, ScenarioRunner, ScenarioSnapshotPolicy,
//!     ScenarioSnapshotRedaction,
//! };
//!
//! struct MyRunner;
//!
//! impl ScenarioRunner for MyRunner {
//!     fn category(&self) -> &str { "my_category" }
//!
//!     fn run(&self, scenario: &Scenario, temp_dir: &Path) -> Result<Value, Box<dyn Error>> {
//!         // Run your test logic here
//!         Ok(serde_json::json!({ "result": "success" }))
//!     }
//! }
//!
//! #[test]
//! fn test_my_scenarios() {
//!     let mut policy = ScenarioSnapshotPolicy::runtime_heavy();
//!     policy.add_redaction(ScenarioSnapshotRedaction::sort(".warnings"));
//!     let mut framework = ScenarioFramework::new().with_snapshot_policy(policy);
//!     framework.register(Box::new(MyRunner));
//!     framework.run_all().unwrap();  // Runs all scenarios with registered runners
//! }
//! ```

pub mod contracts;
pub mod external_test;
pub mod gate;
#[cfg(feature = "performance")]
pub mod performance;
pub mod policy;
pub mod scenario;
pub mod utils;
pub mod validation;

pub use contracts::{
    AdvisoryAuditExecutor, AdvisoryAuditPolicy, AdvisoryAuditRequest, ArtifactKind,
    CollectedArtifact, CollectedArtifacts, CollectionContext, ContractExecutionMode,
    ContractFinding, ContractKnowledgeBatch, ContractKnowledgeDecision, ContractKnowledgeEnvelope,
    ContractReport, ContractRunConfig, ContractStats, ContractSuite, ContractSuiteRunner,
    EvidenceKind, FindingConfidence, FindingEvidence, FindingExamples, FindingMode,
    FindingSeverity, ModularityRulePack, NoopAdvisoryAuditExecutor, NoopRulePack, RestDocsRulePack,
    RoleAuditFinding, RulePack, RulePackDescriptor,
};

pub use external_test::{
    ExternalTestMount, ExternalTestValidationIssue, calculate_test_path, generate_path_attribute,
    validate_external_test_mounts,
};
pub use gate::{
    assert_crate_modularity_gate, collect_crate_modularity_findings,
    format_crate_modularity_gate_report,
};
#[cfg(feature = "performance")]
pub use performance::{
    PerfBudget, PerfQuantiles, PerfReport, PerfRunConfig, PerfSummary, assert_perf_budget,
    default_reports_root, report_output_path, run_async_budget, run_sync_budget,
};

pub use policy::{
    CrateTestPolicyReport, assert_crate_test_policy, assert_crate_test_policy_harness,
    assert_crate_test_policy_with_workspace_config,
    assert_crate_tests_structure_with_workspace_config, format_crate_test_policy_report,
    validate_crate_test_policy, validate_crate_test_policy_with_workspace_config,
    validate_crate_tests_structure_with_workspace_config,
};

pub use scenario::{
    AssertConfig, Scenario, ScenarioConfig, ScenarioFramework, ScenarioMeta, ScenarioRunner,
    ScenarioSnapshotPolicy, ScenarioSnapshotRedaction, ScenarioSnapshotRedactionPreset,
    copy_dir_recursive, discover_scenarios, discover_scenarios_at, find_first_doc_name,
    load_expected_json, scenarios_root, scenarios_root_at,
};

pub use utils::{assert_json_eq, temp_dir_with_prefix};

pub use validation::{
    PathStructureWarning, StructureViolation, TestsStructurePolicy, ViolationKind,
    format_path_structure_warning_report, format_violation_report, validate_crate_path_warnings,
    validate_crate_tests, validate_crate_tests_with_policy, validate_tests_structure,
    validate_tests_structure_with_policy,
};

/// Mount the shared crate test-policy gate directly into a test target.
///
/// This macro is intended for integration-test entry points and other explicit
/// Cargo test targets so `cargo test --test <target>` still runs the crate's
/// shared test-policy gate.
#[macro_export]
macro_rules! crate_test_policy_harness {
    () => {
        #[test]
        fn enforce_crate_test_policy_harness() {
            $crate::assert_crate_test_policy_harness(std::path::Path::new(env!(
                "CARGO_MANIFEST_DIR"
            )));
        }
    };
}

/// Mount a crate test-policy harness from an external source-backed test file.
///
/// This macro is intended for `src/lib.rs`, `src/main.rs`, or another source
/// module that already externalizes unit tests via `#[path]`. It keeps the gate
/// test body out of `src/` while ensuring `cargo test --lib` still runs the
/// shared crate test-policy harness.
#[macro_export]
macro_rules! crate_test_policy_source_harness {
    ($path:literal) => {
        #[cfg(test)]
        #[path = $path]
        mod xiuxian_test_policy_harness;
    };
}

/// Mount the shared crate test-policy gate and the built-in modularity gate
/// into one test target.
///
/// The built-in modularity gate blocks warning-, error-, and
/// critical-severity findings by default.
#[macro_export]
macro_rules! crate_testing_gate {
    () => {
        #[test]
        fn enforce_crate_test_policy_gate() {
            $crate::assert_crate_test_policy_with_workspace_config(std::path::Path::new(env!(
                "CARGO_MANIFEST_DIR"
            )));
        }

        #[test]
        fn enforce_modularity_contract_gate() {
            $crate::assert_crate_modularity_gate(std::path::Path::new(env!("CARGO_MANIFEST_DIR")));
        }
    };
}

/// Mount an external source-backed crate gate entrypoint into a source module
/// such as `src/lib.rs`.
///
/// Keep the full gate body in the mounted file, typically by placing
/// [`crate_testing_gate!`](macro@crate_testing_gate) inside
/// `tests/unit/lib_policy.rs`. This keeps test bodies out of `src/` while
/// ensuring `cargo test --lib` still runs both the shared crate-policy gate
/// and the built-in modularity contract.
#[macro_export]
macro_rules! crate_testing_source_gate {
    ($path:literal) => {
        $crate::crate_test_policy_source_harness!($path);
    };
}

crate_test_policy_source_harness!("../tests/unit/lib_policy.rs");
