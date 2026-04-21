//! Scenario preview and validation tests for Flowhub directories.

use std::fs;
use std::path::PathBuf;

#[path = "support/workspace.rs"]
mod workspace;
use tempfile::TempDir;
use xiuxian_qianji::{
    check_flowhub_scenario, render_flowhub_scenario_check_markdown, render_flowhub_scenario_show,
    show_flowhub_scenario,
};

fn assert_common_diagnostic_shape(rendered: &str) {
    assert!(rendered.contains("# Validation Failed"));
    assert!(rendered.contains("Location:"));
    assert!(rendered.contains("Problem:"));
    assert!(rendered.contains("Why it blocks:"));
    assert!(rendered.contains("Fix:"));
}

fn assert_common_show_shape(rendered: &str) {
    assert!(rendered.starts_with("# "));
    assert!(rendered.contains("Location:"));
    assert!(rendered.contains("\n## "));
}

fn repo_root() -> PathBuf {
    workspace::workspace_root()
}

fn flowhub_root() -> PathBuf {
    repo_root().join("qianji-flowhub")
}

fn real_flowhub_fixture_available() -> bool {
    flowhub_root().join("qianji.toml").is_file()
}

fn scenario_fixture_dir(name: &str) -> PathBuf {
    repo_root().join(format!(
        "packages/rust/crates/xiuxian-qianji/tests/fixtures/flowhub/{name}"
    ))
}

fn write_file(path: &std::path::Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("should create {}: {error}", parent.display()));
    }
    fs::write(path, content)
        .unwrap_or_else(|error| panic!("should write {}: {error}", path.display()));
}

fn create_invalid_scenario_fixture(temp_dir: &TempDir) -> PathBuf {
    let scenario_dir = temp_dir.path().join("scenario");
    write_file(
        &scenario_dir.join("qianji.toml"),
        r#"
version = 1

[planning]
name = "broken-scenario"

[template]
use = ["missing-module as missing"]
"#,
    );
    scenario_dir
}

#[test]
fn show_flowhub_scenario_previews_visible_surfaces() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let show = show_flowhub_scenario(
        flowhub_root(),
        scenario_fixture_dir("coding_rust_blueprint_plan"),
    )
    .unwrap_or_else(|error| panic!("scenario preview should render: {error}"));

    assert_eq!(show.plan_name, "coding-rust-blueprint-plan-demo");
    assert_eq!(show.surfaces.len(), 4);
    assert_eq!(show.surfaces[0].alias, "coding");
    assert_eq!(show.surfaces[1].alias, "rust");
    assert_eq!(show.surfaces[2].alias, "blueprint");
    assert_eq!(show.surfaces[3].alias, "plan");
    assert!(
        show.surfaces[2]
            .source_manifest_path
            .ends_with("qianji-flowhub/blueprint/qianji.toml")
    );
    assert!(show.hidden_aliases.is_empty());

    let rendered = render_flowhub_scenario_show(&show);
    assert_common_show_shape(&rendered);
    assert!(rendered.contains("# Scenario Work Surface Preview"));
    assert!(rendered.contains("## flowchart.mmd"));
    assert!(rendered.contains("Source Manifest:"));
    assert!(rendered.contains("## rust"));
    assert!(rendered.contains("blueprint --> plan"));
}

#[test]
fn check_flowhub_scenario_accepts_real_fixture() {
    if !real_flowhub_fixture_available() {
        return;
    }
    let report = check_flowhub_scenario(
        flowhub_root(),
        scenario_fixture_dir("coding_rust_blueprint_plan"),
    );

    assert!(report.is_valid());
    assert_eq!(
        report.visible_aliases,
        vec![
            "coding".to_string(),
            "rust".to_string(),
            "blueprint".to_string(),
            "plan".to_string()
        ]
    );

    let rendered = render_flowhub_scenario_check_markdown(&report);
    assert!(rendered.contains("# Validation Passed"));
    assert!(rendered.contains("Visible surfaces: flowchart.mmd, coding, rust, blueprint, plan"));
}

#[test]
fn check_flowhub_scenario_reports_resolve_failures() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let scenario_dir = create_invalid_scenario_fixture(&temp_dir);

    let report = check_flowhub_scenario(flowhub_root(), scenario_dir);

    assert!(!report.is_valid());
    let rendered = render_flowhub_scenario_check_markdown(&report);
    assert_common_diagnostic_shape(&rendered);
    assert!(rendered.contains("Scenario resolve failed"));
    assert!(rendered.contains("missing-module"));
    assert!(!rendered.contains("## Follow-up Query"));
}

xiuxian_testing::crate_test_policy_harness!();
