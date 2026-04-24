//! Unified scenario tests for xiuxian-qianji.
#![cfg(feature = "qianji-full")]
//!
//! Single entry point for all scenario-based tests using `ScenarioFramework`.
//! Scenarios are defined in `tests/scenarios/` with insta-managed snapshots.

use std::path::PathBuf;

use xiuxian_testing::{ScenarioFramework, ScenarioSnapshotPolicy};

mod support;
use support::runners::{AuditFlowRunner, LlmAnalyzerRunner};

/// Get the manifest directory for this crate.
fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn run_all_scenarios() {
    let manifest = manifest_dir();
    let scenarios_root = manifest.join("tests").join("scenarios");
    let snapshot_path = manifest.join("tests").join("snapshots");

    let mut framework = ScenarioFramework::with_snapshot_path(&snapshot_path)
        .with_snapshot_policy(ScenarioSnapshotPolicy::recommended());
    // Register all runners
    framework.register(Box::new(LlmAnalyzerRunner));
    framework.register(Box::new(AuditFlowRunner));
    // Run all scenarios with registered runners at this crate's scenarios root
    let count = framework
        .run_all_at(&scenarios_root)
        .unwrap_or_else(|error| panic!("scenario tests should pass: {error}"));
    assert!(count > 0, "should run at least one scenario");
}

xiuxian_testing::crate_test_policy_harness!();
