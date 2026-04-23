//! Materialize tests for Flowhub scenario-to-workdir generation.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[path = "support/workspace.rs"]
mod workspace;
use tempfile::TempDir;
use xiuxian_qianji::{
    advance_workdir_step, check_workdir, materialize_flowhub_anchored_scenario,
    materialize_flowhub_anchored_scenario_at_node, materialize_flowhub_scenario_workdir,
    show_flowhub_graph, show_workdir,
};

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("should create {}: {error}", parent.display()));
    }
    fs::write(path, content)
        .unwrap_or_else(|error| panic!("should write {}: {error}", path.display()));
}

fn create_materialize_fixture(temp_dir: &TempDir) -> (PathBuf, PathBuf) {
    let flowhub_root = temp_dir.path().join("flowhub");
    let scenario_manifest = temp_dir.path().join("scenario/qianji.toml");

    write_file(
        &flowhub_root.join("blueprint/qianji.toml"),
        r#"
version = 1

[module]
name = "blueprint"
tags = ["planning", "blueprint"]

[exports]
entry = "task.blueprint-start"
ready = "task.blueprint-ready"

[contract]
required = ["template", "template/qianji.toml", "template/*.md"]

[[validation]]
scope = "module"
path = "template"
kind = "dir"
required = true

[[validation]]
scope = "module"
path = "template/qianji.toml"
kind = "file"
required = true

[[validation]]
scope = "module"
path = "template/*.md"
kind = "glob"
min_matches = 1
"#,
    );
    write_file(
        &flowhub_root.join("blueprint/template/qianji.toml"),
        "name = \"blueprint\"\n",
    );
    write_file(
        &flowhub_root.join("blueprint/template/01-blueprint.md"),
        "# Blueprint\n",
    );

    write_file(
        &flowhub_root.join("plan/qianji.toml"),
        r#"
version = 1

[module]
name = "plan"
tags = ["planning", "plan"]

[exports]
entry = "task.plan-start"
ready = "task.plan-ready"

[contract]
required = ["template", "template/qianji.toml", "template/*.md"]

[[validation]]
scope = "module"
path = "template"
kind = "dir"
required = true

[[validation]]
scope = "module"
path = "template/qianji.toml"
kind = "file"
required = true

[[validation]]
scope = "module"
path = "template/*.md"
kind = "glob"
min_matches = 1
"#,
    );
    write_file(
        &flowhub_root.join("plan/template/qianji.toml"),
        "name = \"plan\"\n",
    );
    write_file(&flowhub_root.join("plan/template/01-plan.md"), "# Plan\n");

    write_file(
        &scenario_manifest,
        r#"
version = 1

[planning]
name = "blueprint-plan-demo"

[template]
use = [
  "blueprint as blueprint",
  "plan as plan",
]

[[template.link]]
from = "blueprint::task.blueprint-ready"
to = "plan::task.plan-start"
"#,
    );

    (flowhub_root, scenario_manifest)
}

fn create_unlinked_materialize_fixture(temp_dir: &TempDir) -> (PathBuf, PathBuf) {
    let (flowhub_root, scenario_manifest) = create_materialize_fixture(temp_dir);
    write_file(
        &scenario_manifest,
        r#"
version = 1

[planning]
name = "blueprint-plan-demo"

[template]
use = [
  "blueprint as blueprint",
  "plan as plan",
]
"#,
    );

    (flowhub_root, scenario_manifest)
}

fn repo_root() -> PathBuf {
    workspace::workspace_root()
}

fn real_flowhub_paper_anchor() -> PathBuf {
    repo_root().join("qianji-flowhub/research/paper/qianji.toml")
}

fn real_flowhub_paper_anchor_available() -> bool {
    real_flowhub_paper_anchor().is_file()
}

fn real_flowhub_paper_graph() -> PathBuf {
    repo_root().join("qianji-flowhub/research/paper/paper-deep-read.mmd")
}

fn real_flowhub_paper_anchor_supports_localized_contract() -> bool {
    show_flowhub_graph(real_flowhub_paper_graph())
        .is_ok_and(|show| show.declared_check_surface.root.is_some())
}

#[test]
fn materialize_flowhub_scenario_generates_compact_work_surface() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let output_dir = temp_dir.path().join("materialized");
    let (fixture_flowhub_root, scenario_manifest) = create_materialize_fixture(&temp_dir);

    let materialized =
        materialize_flowhub_scenario_workdir(fixture_flowhub_root, scenario_manifest, &output_dir)
            .unwrap_or_else(|error| panic!("scenario should materialize: {error}"));

    assert_eq!(materialized.plan_name, "blueprint-plan-demo");
    assert_eq!(
        materialized.visible_aliases,
        vec!["blueprint".to_string(), "plan".to_string()]
    );
    assert!(output_dir.join("qianji.toml").is_file());
    assert!(output_dir.join("flowchart.mmd").is_file());
    assert!(output_dir.join("blueprint/qianji.toml").is_file());
    assert!(output_dir.join("plan/qianji.toml").is_file());
    assert!(output_dir.join("blueprint/01-blueprint.md").is_file());
    assert!(output_dir.join("plan/01-plan.md").is_file());
    assert!(!output_dir.join("rust").exists());

    let flowchart = fs::read_to_string(output_dir.join("flowchart.mmd"))
        .unwrap_or_else(|error| panic!("should read materialized flowchart: {error}"));
    assert!(flowchart.contains("blueprint --> plan"));

    let show = show_workdir(&output_dir)
        .unwrap_or_else(|error| panic!("materialized workdir should show: {error}"));
    assert_eq!(show.plan_name, "blueprint-plan-demo");

    let report = check_workdir(&output_dir)
        .unwrap_or_else(|error| panic!("materialized workdir should check: {error}"));
    assert!(report.is_valid());
}

#[test]
fn materialize_flowhub_scenario_rejects_non_empty_output_dir() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let output_dir = temp_dir.path().join("materialized");
    let (fixture_flowhub_root, scenario_manifest) = create_materialize_fixture(&temp_dir);
    fs::create_dir_all(&output_dir)
        .unwrap_or_else(|error| panic!("should create output dir: {error}"));
    fs::write(output_dir.join("stale.txt"), "stale")
        .unwrap_or_else(|error| panic!("should write stale file: {error}"));

    let error =
        materialize_flowhub_scenario_workdir(fixture_flowhub_root, scenario_manifest, &output_dir)
            .err()
            .unwrap_or_else(|| panic!("non-empty output dir should fail"));

    assert!(error.to_string().contains("must be empty"));
}

#[test]
fn materialize_flowhub_scenario_reports_follow_up_query_for_invalid_generated_surface() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let output_dir = temp_dir.path().join("materialized");
    let (fixture_flowhub_root, scenario_manifest) = create_unlinked_materialize_fixture(&temp_dir);

    let error =
        materialize_flowhub_scenario_workdir(fixture_flowhub_root, scenario_manifest, &output_dir)
            .err()
            .unwrap_or_else(|| panic!("unlinked materialized work surface should fail"));

    let rendered = error.to_string();
    assert!(rendered.contains("Generated work surface"));
    assert!(rendered.contains("# Validation Failed"));
    assert!(rendered.contains("Missing flowchart backbone"));
    assert!(rendered.contains("## Follow-up Query"));
    assert_eq!(rendered.matches("## Follow-up Query").count(), 1);
    assert!(rendered.contains("Surfaces: blueprint, plan"));
    assert!(rendered.contains(
        "select path, surface, heading_path, skeleton \
from markdown \
where surface in ('blueprint', 'plan') \
order by surface, path, heading_path"
    ));
}

#[test]
fn materialize_flowhub_anchored_scenario_generates_step_aware_run_root() {
    if !real_flowhub_paper_anchor_available()
        || !real_flowhub_paper_anchor_supports_localized_contract()
    {
        return;
    }
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let output_dir = temp_dir.path().join("runs/run_001");
    let anchor = real_flowhub_paper_anchor();

    let materialized =
        materialize_flowhub_anchored_scenario(&anchor, "paper-deep-read", &output_dir)
            .unwrap_or_else(|error| panic!("anchored scenario should materialize: {error}"));

    assert_eq!(materialized.plan_name, "deep_read");
    assert_eq!(materialized.current_node, "research/paper");
    assert_eq!(
        materialized.allowed_next,
        vec!["load_paper_package".to_string()]
    );
    assert!(materialized.current_step_surface.is_empty());
    assert!(output_dir.join("qianji.toml").is_file());
    assert!(output_dir.join("flowchart.mmd").is_file());
    assert!(output_dir.join("refs/paper.json").is_file());
    assert!(output_dir.join("refs/topic.json").is_file());
    assert!(output_dir.join("state/current_node.toml").is_file());
    assert!(output_dir.join("state/allowed_next.json").is_file());

    let report = check_workdir(&output_dir)
        .unwrap_or_else(|error| panic!("anchored materialized workdir should check: {error}"));
    assert!(report.is_valid());
}

#[test]
fn materialize_flowhub_anchored_scenario_scaffolds_selected_current_node() {
    if !real_flowhub_paper_anchor_available()
        || !real_flowhub_paper_anchor_supports_localized_contract()
    {
        return;
    }
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let output_dir = temp_dir.path().join("runs/run_003");
    let anchor = real_flowhub_paper_anchor();

    let materialized = materialize_flowhub_anchored_scenario_at_node(
        &anchor,
        "paper-deep-read",
        &output_dir,
        Some("claim_extract"),
    )
    .unwrap_or_else(|error| {
        panic!("anchored scenario should scaffold selected current node: {error}")
    });

    assert_eq!(materialized.plan_name, "deep_read");
    assert_eq!(materialized.current_node, "claim_extract");
    assert_eq!(
        materialized.allowed_next,
        vec!["diagnostics".to_string(), "evidence_ground".to_string()]
    );
    assert_eq!(
        materialized.current_step_surface,
        vec![
            "checkpoints/claim_extract.json".to_string(),
            "staging/semantics/claim_ledger.patch.jsonl".to_string()
        ]
    );
    assert!(output_dir.join("checkpoints/claim_extract.json").is_file());
    assert!(
        output_dir
            .join("staging/semantics/claim_ledger.patch.jsonl")
            .is_file()
    );
    assert!(!output_dir.join("checkpoints/evidence_ground.json").exists());

    let report = check_workdir(&output_dir).unwrap_or_else(|error| {
        panic!("current-node scaffolded anchored materialized workdir should check: {error}")
    });
    assert!(report.is_valid());
}

#[test]
fn advance_workdir_step_updates_localized_step_state() {
    if !real_flowhub_paper_anchor_available()
        || !real_flowhub_paper_anchor_supports_localized_contract()
    {
        return;
    }
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let output_dir = temp_dir.path().join("runs/run_005");
    let anchor = real_flowhub_paper_anchor();

    materialize_flowhub_anchored_scenario_at_node(
        &anchor,
        "paper-deep-read",
        &output_dir,
        Some("claim_extract"),
    )
    .unwrap_or_else(|error| {
        panic!("anchored scenario should scaffold selected current node: {error}")
    });

    let advanced = advance_workdir_step(&output_dir, "evidence_ground")
        .unwrap_or_else(|error| panic!("adjacent localized advance should succeed: {error}"));

    assert_eq!(advanced.plan_name, "deep_read");
    assert_eq!(advanced.previous_node, "claim_extract");
    assert_eq!(advanced.current_node, "evidence_ground");
    assert_eq!(
        advanced.allowed_next,
        vec!["diagnostics".to_string(), "limitation_extract".to_string()]
    );
    assert_eq!(advanced.trace_path, output_dir.join("state/trace.jsonl"));
    assert_eq!(
        fs::read_to_string(output_dir.join("state/current_node.toml"))
            .unwrap_or_else(|error| panic!("advanced current node should be readable: {error}")),
        "current_node = \"evidence_ground\"\n"
    );
    assert!(
        output_dir
            .join("checkpoints/evidence_ground.json")
            .is_file()
    );
    assert!(
        output_dir
            .join("staging/semantics/evidence_ledger.patch.jsonl")
            .is_file()
    );

    let trace = fs::read_to_string(output_dir.join("state/trace.jsonl"))
        .unwrap_or_else(|error| panic!("advanced trace should be readable: {error}"));
    assert!(trace.contains("\"event\":\"step_advance\""));
    assert!(trace.contains("\"from\":\"claim_extract\""));
    assert!(trace.contains("\"to\":\"evidence_ground\""));

    let report = check_workdir(&output_dir)
        .unwrap_or_else(|error| panic!("advanced anchored workdir should check: {error}"));
    assert!(report.is_valid());
}

xiuxian_testing::crate_test_policy_harness!();
