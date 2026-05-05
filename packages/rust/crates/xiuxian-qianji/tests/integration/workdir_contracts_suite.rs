//! Contract tests for compact bounded work-surface show/check behavior.

use std::fs;
use std::path::Path;

use tempfile::TempDir;
use xiuxian_qianji::{
    WorkdirMarkdownSurface, WorkdirSemanticScopeGuardStatus, WorkdirVisibleSurfaceKind,
    build_workdir_check_follow_up_query, check_workdir, load_workdir_manifest,
    parse_workdir_manifest, query_workdir_check_follow_up_payload, query_workdir_markdown_payload,
    render_workdir_check_markdown, render_workdir_semantic_scope_guard_trace, render_workdir_show,
    show_workdir, trace_workdir_semantic_scope_json,
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

fn semantic_workdir_manifest() -> &'static str {
    r#"
version = 1

[plan]
name = "semantic-demo"
surface = ["flowchart.mmd", "blueprint", "plan", "semantic"]

[check]
require = [
  "flowchart.mmd",
  "blueprint",
  "plan",
  "semantic",
  "blueprint/**/*.md",
  "plan/**/*.md",
  "semantic/**/*.md",
  "semantic/objects/component/demo.md",
  "semantic/change-intents/demo-change.md",
]
flowchart = ["blueprint", "plan", "semantic"]
"#
}

fn create_semantic_workdir(temp_dir: &TempDir) -> std::path::PathBuf {
    let workdir = temp_dir.path().join("semantic-demo");
    fs::create_dir_all(&workdir)
        .unwrap_or_else(|error| panic!("should create workdir {}: {error}", workdir.display()));
    write_file(&workdir.join("qianji.toml"), semantic_workdir_manifest());
    write_file(
        &workdir.join("flowchart.mmd"),
        "flowchart LR\n  blueprint --> plan\n  plan --> semantic\n",
    );
    write_file(
        &workdir.join("blueprint/architecture.md"),
        "# Blueprint\n\n## Boundary\n\n- [ ] define boundary\n",
    );
    write_file(
        &workdir.join("plan/tasks.md"),
        "# Plan\n\n## Rust\n\n- [ ] implement\n",
    );
    write_file(
        &workdir.join("semantic/objects/component/demo.md"),
        "# Demo Component\n\n## Authority\n\n- Repo-native semantic object\n",
    );
    write_file(
        &workdir.join("semantic/change-intents/demo-change.md"),
        "# Demo Change Intent\n\n## Required Validations\n\n- cargo test\n",
    );
    workdir
}

fn semantic_scope_metadata_json(projection_staleness: &str, unresolved_ids: &[&str]) -> String {
    serde_json::json!({
        "semanticScopeBundle": {
            "task_id": "task.demo",
            "requested_object_ids": ["component.demo", "task.demo"],
            "objects": [
                {
                    "id": "component.demo",
                    "kind": "component",
                    "title": "Demo Component",
                    "status": "active",
                    "confidence": {
                        "score": 0.95,
                        "source": "verified"
                    },
                    "owners": [
                        {
                            "scope": "xiuxian-qianji",
                            "role": "semantic_scope_consumer"
                        }
                    ],
                    "provenance": {
                        "source": "docs/rfcs/demo.md",
                        "recorded_by": "test",
                        "recorded_at": "2026-05-05"
                    },
                    "verification": {
                        "required": ["cargo test -p xiuxian-qianji workdir_semantic"]
                    },
                    "relations": []
                },
                {
                    "id": "task.demo",
                    "kind": "task",
                    "title": "Candidate Demo Task",
                    "status": "candidate",
                    "confidence": {
                        "score": 0.55,
                        "source": "llm_suggested"
                    },
                    "owners": [
                        {
                            "scope": "xiuxian-qianji",
                            "role": "candidate_scope_consumer"
                        }
                    ],
                    "provenance": {
                        "source": "semantic/change-intents/demo-change.md",
                        "recorded_by": "test",
                        "recorded_at": "2026-05-05"
                    },
                    "verification": {
                        "required": ["cargo test -p xiuxian-qianji workdir_semantic"]
                    },
                    "relations": []
                }
            ],
            "relations": [
                {
                    "source": "component.demo",
                    "kind": "validates",
                    "target": "task.demo"
                }
            ],
            "change_intents": [
                {
                    "type": "semantic_change_intent",
                    "id": "change.demo",
                    "title": "Demo Change",
                    "status": "active",
                    "touched_objects": ["component.demo"],
                    "changed_relations": [],
                    "affected_invariants": ["task.demo"],
                    "required_validations": ["cargo test -p xiuxian-qianji workdir_semantic"],
                    "projections_to_refresh": ["llm_compression"],
                    "candidate_suggestions": ["task.demo"]
                }
            ],
            "affected_invariants": ["task.demo"],
            "required_validations": ["cargo test -p xiuxian-qianji workdir_semantic"],
            "projection_revision": "semantic-scope-demo",
            "projection_source_revision": "blake3:demo",
            "projection_staleness": projection_staleness,
            "provenance": [
                {
                    "object_id": "component.demo",
                    "source_path": "semantic/objects/component/demo.md",
                    "source": "docs/rfcs/demo.md"
                }
            ],
            "unresolved_ids": unresolved_ids
        }
    })
    .to_string()
}

fn semantic_scope_metadata_with_sql_guard_json(
    status: &str,
    failing_row_count: usize,
    message: &str,
) -> String {
    let mut value =
        serde_json::from_str::<serde_json::Value>(&semantic_scope_metadata_json("fresh", &[]))
            .unwrap_or_else(|error| {
                panic!("semantic-scope metadata fixture should decode: {error}")
            });
    value
        .as_object_mut()
        .unwrap_or_else(|| panic!("semantic-scope metadata fixture should be an object"))
        .insert(
            "semanticSqlGuardEvidence".to_string(),
            serde_json::json!({
                "guardId": "semantic_sql.projection_freshness",
                "status": status,
                "failingRowCount": failing_row_count,
                "message": message
            }),
        );
    value.to_string()
}

fn semantic_scope_metadata_with_projection_policy_json(
    status: &str,
    failing_projection_count: usize,
    message: &str,
) -> String {
    let mut value =
        serde_json::from_str::<serde_json::Value>(&semantic_scope_metadata_json("fresh", &[]))
            .unwrap_or_else(|error| {
                panic!("semantic-scope metadata fixture should decode: {error}")
            });
    value
        .as_object_mut()
        .unwrap_or_else(|| panic!("semantic-scope metadata fixture should be an object"))
        .insert(
            "semanticProjectionPolicyEvidence".to_string(),
            serde_json::json!({
                "policyId": "semantic_projection.required_refresh_targets",
                "status": status,
                "failingProjectionCount": failing_projection_count,
                "message": message,
                "projections": []
            }),
        );
    value.to_string()
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
        "select path, surface, surface_kind, heading_path, skeleton \
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
        "select path, surface, surface_kind, heading_path, skeleton \
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
        "select path, surface, surface_kind, heading_path, skeleton \
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

#[test]
fn workdir_semantic_surface_checks_valid() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_semantic_workdir(&temp_dir);

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("semantic work surface should check: {error}"));
    assert!(report.is_valid());

    let show = show_workdir(&workdir)
        .unwrap_or_else(|error| panic!("semantic work surface should show: {error}"));
    assert!(
        show.surfaces
            .iter()
            .any(|surface| surface.surface == "semantic"
                && surface.kind == WorkdirVisibleSurfaceKind::Directory)
    );
}

#[test]
fn workdir_semantic_follow_up_query_targets_semantic_surface() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_semantic_workdir(&temp_dir);

    fs::remove_file(workdir.join("semantic/objects/component/demo.md"))
        .unwrap_or_else(|error| panic!("should remove semantic object: {error}"));

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("invalid semantic surface should still report: {error}"));
    let follow_up = build_workdir_check_follow_up_query(&report)
        .unwrap_or_else(|| panic!("failing semantic report should derive follow-up query"));

    assert_eq!(follow_up.surfaces, vec![WorkdirMarkdownSurface::Semantic]);
    assert_eq!(
        follow_up.query_text,
        "select path, surface, surface_kind, heading_path, skeleton \
from markdown \
where surface = 'semantic' \
order by surface, path, heading_path"
    );
}

#[test]
fn workdir_semantic_change_intent_follow_up_query_targets_semantic_surface() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_semantic_workdir(&temp_dir);

    fs::remove_file(workdir.join("semantic/change-intents/demo-change.md"))
        .unwrap_or_else(|error| panic!("should remove semantic change intent: {error}"));

    let report = check_workdir(&workdir)
        .unwrap_or_else(|error| panic!("invalid semantic surface should still report: {error}"));
    let follow_up = build_workdir_check_follow_up_query(&report)
        .unwrap_or_else(|| panic!("failing semantic report should derive follow-up query"));

    assert_eq!(follow_up.surfaces, vec![WorkdirMarkdownSurface::Semantic]);
    assert_eq!(
        follow_up.query_text,
        "select path, surface, surface_kind, heading_path, skeleton \
from markdown \
where surface = 'semantic' \
order by surface, path, heading_path"
    );
}

#[tokio::test]
async fn workdir_semantic_query_surface_returns_sql_payload() {
    let temp_dir =
        TempDir::new().unwrap_or_else(|error| panic!("temp dir should allocate: {error}"));
    let workdir = create_semantic_workdir(&temp_dir);

    let payload = query_workdir_markdown_payload(
        &workdir,
        "select path, surface, surface_kind, heading_path from markdown where surface = 'semantic' order by path, heading_path",
    )
    .await
    .unwrap_or_else(|error| panic!("semantic workdir SQL payload should resolve: {error}"));

    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(|row| row.get("path").and_then(serde_json::Value::as_str)
                == Some("semantic/objects/component/demo.md"))
    );
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(
                |row| row.get("surface_kind").and_then(serde_json::Value::as_str)
                    == Some("semantic_object")
            )
    );
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(|row| row.get("path").and_then(serde_json::Value::as_str)
                == Some("semantic/change-intents/demo-change.md"))
    );
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(
                |row| row.get("surface_kind").and_then(serde_json::Value::as_str)
                    == Some("semantic_change_intent")
            )
    );
    assert!(
        payload
            .batches
            .iter()
            .flat_map(|batch| batch.rows.iter())
            .any(
                |row| row.get("heading_path").and_then(serde_json::Value::as_str)
                    == Some("Demo Component/Authority")
            )
    );
}

#[test]
fn workdir_semantic_scope_guard_trace_consumes_wendao_metadata_bundle() {
    let trace = trace_workdir_semantic_scope_json(&semantic_scope_metadata_json("fresh", &[]))
        .unwrap_or_else(|error| panic!("semantic-scope metadata should decode: {error}"));

    assert_eq!(trace.status, WorkdirSemanticScopeGuardStatus::Ready);
    assert_eq!(trace.task_id.as_deref(), Some("task.demo"));
    assert_eq!(trace.relation_count, 1);
    assert_eq!(trace.change_intent_ids, vec!["change.demo"]);
    assert!(
        trace
            .objects
            .iter()
            .any(|object| object.id == "task.demo" && object.status == "candidate")
    );
    assert!(
        trace
            .required_validations
            .contains(&"cargo test -p xiuxian-qianji workdir_semantic".to_string())
    );

    let rendered = render_workdir_semantic_scope_guard_trace(&trace);
    assert!(rendered.contains("Status: ready"));
    assert!(rendered.contains("task.demo [task / candidate]"));
    assert!(rendered.contains("change.demo"));
}

#[test]
fn workdir_semantic_scope_guard_trace_marks_stale_projection_for_review() {
    let trace = trace_workdir_semantic_scope_json(&semantic_scope_metadata_json("stale", &[]))
        .unwrap_or_else(|error| panic!("stale semantic-scope metadata should decode: {error}"));

    assert_eq!(
        trace.status,
        WorkdirSemanticScopeGuardStatus::ReviewRequired
    );
    assert!(
        trace
            .issues
            .iter()
            .any(|issue| issue.contains("semantic projection is stale"))
    );
}

#[test]
fn workdir_semantic_scope_guard_trace_consumes_sql_guard_review_evidence() {
    let trace = trace_workdir_semantic_scope_json(&semantic_scope_metadata_with_sql_guard_json(
        "review_required",
        1,
        "semantic projection freshness guard requires review: 1 stale projection row(s)",
    ))
    .unwrap_or_else(|error| panic!("semantic SQL guard evidence should decode: {error}"));

    assert_eq!(
        trace.status,
        WorkdirSemanticScopeGuardStatus::ReviewRequired
    );
    assert_eq!(trace.sql_guard_evidence.len(), 1);
    assert_eq!(
        trace.sql_guard_evidence[0].guard_id,
        "semantic_sql.projection_freshness"
    );
    assert_eq!(trace.sql_guard_evidence[0].failing_row_count, 1);
    assert!(
        trace
            .issues
            .iter()
            .any(|issue| issue.contains("semantic_sql.projection_freshness"))
    );

    let rendered = render_workdir_semantic_scope_guard_trace(&trace);
    assert!(rendered.contains("## SQL Guard Evidence"));
    assert!(rendered.contains("semantic_sql.projection_freshness"));
}

#[test]
fn workdir_semantic_scope_guard_trace_keeps_passed_sql_guard_ready() {
    let trace = trace_workdir_semantic_scope_json(&semantic_scope_metadata_with_sql_guard_json(
        "passed",
        0,
        "semantic projection freshness guard passed: no stale projection rows",
    ))
    .unwrap_or_else(|error| panic!("passed semantic SQL guard evidence should decode: {error}"));

    assert_eq!(trace.status, WorkdirSemanticScopeGuardStatus::Ready);
    assert_eq!(trace.sql_guard_evidence.len(), 1);
    assert!(trace.issues.is_empty());
}

#[test]
fn workdir_semantic_scope_guard_trace_consumes_projection_policy_review_evidence() {
    let trace = trace_workdir_semantic_scope_json(
        &semantic_scope_metadata_with_projection_policy_json(
            "review_required",
            1,
            "active change-intent projection refresh target(s) are stale",
        ),
    )
    .unwrap_or_else(|error| panic!("semantic projection policy evidence should decode: {error}"));

    assert_eq!(
        trace.status,
        WorkdirSemanticScopeGuardStatus::ReviewRequired
    );
    assert_eq!(trace.projection_policy_evidence.len(), 1);
    assert_eq!(
        trace.projection_policy_evidence[0].policy_id,
        "semantic_projection.required_refresh_targets"
    );
    assert_eq!(
        trace.projection_policy_evidence[0].failing_projection_count,
        1
    );
    assert!(
        trace
            .issues
            .iter()
            .any(|issue| issue.contains("semantic_projection.required_refresh_targets"))
    );

    let rendered = render_workdir_semantic_scope_guard_trace(&trace);
    assert!(rendered.contains("## Projection Policy Evidence"));
    assert!(rendered.contains("semantic_projection.required_refresh_targets"));
}

#[test]
fn workdir_semantic_scope_guard_trace_keeps_passed_projection_policy_ready() {
    let trace =
        trace_workdir_semantic_scope_json(&semantic_scope_metadata_with_projection_policy_json(
            "passed",
            0,
            "all active change-intent projection refresh targets are fresh",
        ))
        .unwrap_or_else(|error| panic!("passed projection policy evidence should decode: {error}"));

    assert_eq!(trace.status, WorkdirSemanticScopeGuardStatus::Ready);
    assert_eq!(trace.projection_policy_evidence.len(), 1);
    assert!(trace.issues.is_empty());
}

#[test]
fn workdir_semantic_scope_guard_trace_blocks_unresolved_ids() {
    let trace = trace_workdir_semantic_scope_json(&semantic_scope_metadata_json(
        "fresh",
        &["decision.missing"],
    ))
    .unwrap_or_else(|error| panic!("unresolved semantic-scope metadata should decode: {error}"));

    assert_eq!(trace.status, WorkdirSemanticScopeGuardStatus::Blocked);
    assert_eq!(trace.unresolved_ids, vec!["decision.missing"]);
    assert!(
        trace
            .issues
            .iter()
            .any(|issue| issue.contains("decision.missing"))
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
