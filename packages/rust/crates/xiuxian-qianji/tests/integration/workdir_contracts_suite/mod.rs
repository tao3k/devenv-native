//! Contract tests for compact bounded work-surface show/check behavior.

use std::fs;
use std::path::Path;

use tempfile::TempDir;

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

mod manifest;
mod query;
mod semantic_scope_guard;
mod semantic_surface;
mod show_check;
