//! Contract tests for compact bounded work-surface show/check behavior.

use std::fs;
use std::path::Path;

use tempfile::TempDir;
use xiuxian_qianji::{
    WorkdirMarkdownSurface, WorkdirVisibleSurfaceKind, build_workdir_check_follow_up_query,
    check_workdir, load_workdir_manifest, parse_workdir_manifest,
    query_workdir_check_follow_up_payload, query_workdir_markdown_payload,
    render_workdir_check_markdown, render_workdir_show, show_workdir,
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

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("should create {}: {error}", parent.display()));
    }
    fs::write(path, content)
        .unwrap_or_else(|error| panic!("should write {}: {error}", path.display()));
}

fn valid_workdir_manifest() -> &'static str {
    r#"
version = 1

[plan]
name = "demo-plan"
surface = ["flowchart.mmd", "blueprint", "plan"]

[check]
require = ["flowchart.mmd", "blueprint", "plan", "blueprint/**/*.md", "plan/**/*.md"]
flowchart = ["blueprint", "plan"]
"#
}

fn create_valid_workdir(temp_dir: &TempDir) -> std::path::PathBuf {
    let workdir = temp_dir.path().join("demo-plan");
    fs::create_dir_all(&workdir)
        .unwrap_or_else(|error| panic!("should create workdir {}: {error}", workdir.display()));
    write_file(&workdir.join("qianji.toml"), valid_workdir_manifest());
    write_file(
        &workdir.join("flowchart.mmd"),
        "flowchart LR\n  blueprint --> plan\n",
    );
    write_file(
        &workdir.join("blueprint/architecture.md"),
        "# Blueprint\n\n## Boundary\n\n- [ ] define boundary\n",
    );
    write_file(
        &workdir.join("plan/tasks.md"),
        "# Plan\n\n## Rust\n\n- [ ] implement\n",
    );
    workdir
}

fn step_aware_workdir_manifest() -> &'static str {
    r#"
version = 1

[plan]
name = "paper-step-demo"
surface = ["flowchart.mmd", "refs", "state", "checkpoints", "staging", "diagnostics", "outputs"]

[check]
require = [
  "qianji.toml",
  "flowchart.mmd",
  "refs/paper.json",
  "refs/topic.json",
  "state/current_node.toml",
  "state/trace.jsonl",
  "state/allowed_next.json",
  "checkpoints/methods_extract.json",
  "checkpoints/results_extract.json",
  "staging/semantics/method_card.patch.json",
  "staging/semantics/result_sheet.patch.json",
  "diagnostics/latest_check.md",
  "diagnostics/blocked.json",
  "diagnostics/failed.json",
  "outputs/response_preview.md",
]
flowchart = ["state", "checkpoints", "staging"]
"#
}

fn create_step_aware_workdir(temp_dir: &TempDir) -> std::path::PathBuf {
    let workdir = temp_dir.path().join("paper-step-demo");
    fs::create_dir_all(&workdir)
        .unwrap_or_else(|error| panic!("should create workdir {}: {error}", workdir.display()));
    write_file(&workdir.join("qianji.toml"), step_aware_workdir_manifest());
    write_file(
        &workdir.join("flowchart.mmd"),
        r#"
%% qianji.scenario.id: deep_read
%% qianji.scenario.name: PAPER_STEP_DEMO
%% qianji.scenario.workdir_root: runs/<run_id>
%% qianji.scenario.requires:
%%   - refs/paper.json
%%   - refs/topic.json
%% qianji.scenario.target_root: papers/<paper_id>
%% qianji.scenario.target_paths:
%%   - semantics/method_card.json
%%   - semantics/result_sheet.json
flowchart LR
  paper_package["research/paper"] --> methods_extract["methods_extract"]
  methods_extract --> results_extract["results_extract"]
  results_extract --> done_gate["done gate"]

%% qianji.node.paper_package.kind: artifact
%% qianji.node.methods_extract.kind: process
%% qianji.node.methods_extract.checkpoint: checkpoints/methods_extract.json
%% qianji.node.methods_extract.writes:
%%   - staging/semantics/method_card.patch.json
%% qianji.node.methods_extract.merge_target:
%%   - semantics/method_card.json
%% qianji.node.results_extract.kind: process
%% qianji.node.results_extract.checkpoint: checkpoints/results_extract.json
%% qianji.node.results_extract.writes:
%%   - staging/semantics/result_sheet.patch.json
%% qianji.node.results_extract.merge_target:
%%   - semantics/result_sheet.json
%% qianji.node.done_gate.kind: gate
%% qianji.done_gate.require:
%%   - semantics/method_card.json
%%   - semantics/result_sheet.json
"#,
    );
    write_file(&workdir.join("refs/paper.json"), "{}\n");
    write_file(&workdir.join("refs/topic.json"), "{}\n");
    write_file(
        &workdir.join("state/current_node.toml"),
        "current_node = \"methods_extract\"\n",
    );
    write_file(&workdir.join("state/trace.jsonl"), "{}\n");
    write_file(
        &workdir.join("state/allowed_next.json"),
        "[\"results_extract\"]\n",
    );
    write_file(&workdir.join("checkpoints/methods_extract.json"), "{}\n");
    write_file(
        &workdir.join("staging/semantics/method_card.patch.json"),
        "{}\n",
    );
    write_file(&workdir.join("diagnostics/latest_check.md"), "# Check\n");
    write_file(&workdir.join("diagnostics/blocked.json"), "[]\n");
    write_file(&workdir.join("diagnostics/failed.json"), "[]\n");
    write_file(&workdir.join("outputs/response_preview.md"), "# Preview\n");
    workdir
}

#[test]
fn bounded_workdir_manifest_parses_compact_contract() {
    let manifest = parse_workdir_manifest(valid_workdir_manifest())
        .unwrap_or_else(|error| panic!("compact work-surface manifest should parse: {error}"));

    assert_eq!(manifest.version, 1);
    assert_eq!(manifest.plan.name, "demo-plan");
    assert_eq!(
        manifest.plan.surface,
        vec![
            "flowchart.mmd".to_string(),
            "blueprint".to_string(),
            "plan".to_string()
        ]
    );
    assert_eq!(
        manifest.check.flowchart,
        vec!["blueprint".to_string(), "plan".to_string()]
    );
}

#[test]
fn bounded_workdir_manifest_rejects_missing_flowchart_surface() {
    let error = parse_workdir_manifest(
        r#"
version = 1

[plan]
name = "broken"
surface = ["blueprint", "plan"]

[check]
require = ["flowchart.mmd", "blueprint", "plan"]
flowchart = ["blueprint", "plan"]
"#,
    )
    .err()
    .unwrap_or_else(|| panic!("missing flowchart surface should fail"));

    assert!(
        error
            .to_string()
            .contains("`plan.surface` must include `flowchart.mmd`")
    );
}

#[test]
fn load_workdir_manifest_reads_real_file() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    let manifest = load_workdir_manifest(workdir.join("qianji.toml"))
        .unwrap_or_else(|error| panic!("root manifest file should load: {error}"));

    assert_eq!(manifest.plan.name, "demo-plan");
    assert_eq!(manifest.check.require.len(), 5);
}

#[test]
fn show_workdir_reports_top_level_surface_state() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    fs::remove_dir_all(workdir.join("plan"))
        .unwrap_or_else(|error| panic!("should remove plan dir for show test: {error}"));

    let show = show_workdir(&workdir)
        .unwrap_or_else(|error| panic!("show surface should still render: {error}"));

    assert_eq!(show.plan_name, "demo-plan");
    assert_eq!(show.surfaces[0].surface, "flowchart.mmd");
    assert_eq!(show.surfaces[0].kind, WorkdirVisibleSurfaceKind::File);
    assert_eq!(show.surfaces[1].kind, WorkdirVisibleSurfaceKind::Directory);
    assert_eq!(show.surfaces[2].kind, WorkdirVisibleSurfaceKind::Missing);

    let rendered = render_workdir_show(&show);
    assert_common_show_shape(&rendered);
    assert!(rendered.contains("# Work Surface"));
    assert!(rendered.contains("## flowchart.mmd"));
    assert!(rendered.contains("Status: missing"));
}

#[test]
fn check_workdir_accepts_valid_surface() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("valid work surface should check: {error}"));

    assert!(report.is_valid());
    let rendered = render_workdir_check_markdown(&report);
    assert!(rendered.contains("# Validation Passed"));
}

#[test]
fn check_workdir_accepts_step_aware_current_node_without_future_outputs() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_step_aware_workdir(&temp_dir);

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("step-aware work surface should check: {error}"));

    assert!(report.is_valid());
    let rendered = render_workdir_check_markdown(&report);
    assert!(rendered.contains("# Validation Passed"));
}

#[test]
fn check_workdir_blocks_allowed_next_drift_for_step_aware_surface() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_step_aware_workdir(&temp_dir);
    write_file(
        &workdir.join("state/allowed_next.json"),
        "[\"done_gate\"]\n",
    );

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("step-aware drift should still report: {error}"));

    assert!(!report.is_valid());
    let rendered = render_workdir_check_markdown(&report);
    assert_common_diagnostic_shape(&rendered);
    assert!(rendered.contains("Allowed-next drift"));
    assert!(rendered.contains("current node `methods_extract`"));
    assert!(rendered.contains("`results_extract`"));
    assert!(rendered.contains("`done gate`"));
}

#[test]
fn check_workdir_reports_missing_glob_matches_and_backbone_conflicts() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    fs::remove_file(workdir.join("plan/tasks.md"))
        .unwrap_or_else(|error| panic!("should remove plan markdown: {error}"));
    write_file(
        &workdir.join("flowchart.mmd"),
        "flowchart LR\n  plan --> blueprint\n",
    );

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("invalid work surface should still report: {error}"));

    assert!(!report.is_valid());
    let rendered = render_workdir_check_markdown(&report);
    assert_common_diagnostic_shape(&rendered);
    assert!(rendered.contains("Missing required glob matches"));
    assert!(rendered.contains("Missing flowchart backbone"));
    assert!(rendered.contains("blueprint --> plan"));
    assert!(rendered.contains("## Follow-up Query"));
    assert!(rendered.contains("Surfaces: blueprint, plan"));
    assert!(rendered.contains(
        "select path, surface, heading_path, skeleton \
from markdown \
where surface in ('blueprint', 'plan') \
order by surface, path, heading_path"
    ));
}

#[test]
fn check_workdir_render_includes_follow_up_query_on_failure() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    fs::remove_file(workdir.join("plan/tasks.md"))
        .unwrap_or_else(|error| panic!("should remove plan markdown: {error}"));

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("invalid work surface should still report: {error}"));
    let rendered = render_workdir_check_markdown(&report);

    assert!(rendered.contains("## Follow-up Query"));
    assert!(rendered.contains("Surfaces: plan"));
    assert!(rendered.contains(
        "select path, surface, heading_path, skeleton \
from markdown \
where surface = 'plan' \
order by surface, path, heading_path"
    ));
}

#[test]
fn workdir_check_follow_up_query_stays_surface_bounded() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    fs::remove_file(workdir.join("plan/tasks.md"))
        .unwrap_or_else(|error| panic!("should remove plan markdown: {error}"));

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("invalid work surface should still report: {error}"));
    let follow_up = build_workdir_check_follow_up_query(&report)
        .unwrap_or_else(|| panic!("failing report should derive follow-up query"));

    assert_eq!(follow_up.workdir, workdir);
    assert_eq!(follow_up.surfaces, vec![WorkdirMarkdownSurface::Plan]);
    assert_eq!(
        follow_up.query_text,
        "select path, surface, heading_path, skeleton \
from markdown \
where surface = 'plan' \
order by surface, path, heading_path"
    );
}

#[tokio::test]
async fn workdir_query_surface_returns_sql_payload() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    let payload = query_workdir_markdown_payload(
        &workdir,
        "select path, heading_path from markdown where surface = 'plan' order by path, heading_path",
    )
    .await
    .unwrap_or_else(|error| panic!("workdir SQL payload should resolve: {error}"));

    assert_eq!(
        payload.metadata.registered_tables,
        vec!["markdown".to_string()]
    );
    assert_eq!(payload.metadata.registered_table_count, 1);
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(|row| row.get("path").and_then(serde_json::Value::as_str) == Some("plan/tasks.md"))
    );
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(
                |row| row.get("heading_path").and_then(serde_json::Value::as_str)
                    == Some("Plan/Rust")
            )
    );
}

#[tokio::test]
async fn workdir_check_follow_up_query_returns_surface_bounded_payload() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    write_file(
        &workdir.join("flowchart.mmd"),
        "flowchart LR\n  plan --> blueprint\n",
    );

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("invalid work surface should still report: {error}"));
    let follow_up_payload = query_workdir_check_follow_up_payload(&report)
        .await
        .unwrap_or_else(|error| panic!("follow-up payload should resolve: {error}"))
        .unwrap_or_else(|| panic!("failing report should emit follow-up payload"));

    assert_eq!(
        follow_up_payload.metadata.registered_tables,
        vec!["markdown".to_string()]
    );
    assert!(
        follow_up_payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .all(|row| {
                row.get("surface")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|surface| matches!(surface, "blueprint" | "plan"))
            })
    );
    assert!(
        follow_up_payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(|row| row.get("path").and_then(serde_json::Value::as_str)
                == Some("blueprint/architecture.md"))
    );
    assert!(
        follow_up_payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(|row| row.get("path").and_then(serde_json::Value::as_str)
                == Some("plan/tasks.md"))
    );
}

#[tokio::test]
async fn valid_workdir_has_no_follow_up_payload() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_valid_workdir(&temp_dir);

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("valid work surface should check: {error}"));
    let follow_up_payload = query_workdir_check_follow_up_payload(&report)
        .await
        .unwrap_or_else(|error| panic!("valid follow-up lookup should not fail: {error}"));

    assert!(follow_up_payload.is_none());
}
