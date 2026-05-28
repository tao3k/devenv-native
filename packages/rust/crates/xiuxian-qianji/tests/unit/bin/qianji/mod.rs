#[cfg(feature = "wendao-integration")]
use super::resolve_workspace_root;
use super::{
    BpmnCliCheckpointBackend, BpmnCliCommand, BpmnHostSessionCliCommand, BpmnRunCliCommand,
    BpmnStartAtCliCommand, BpmnStartCliCommand, BpmnStatusCliCommand, BpmnTaskClaimCliCommand,
    BpmnTaskCompleteCliCommand, BpmnTaskCompleteCliKind, BpmnTaskReleaseCliCommand,
    BpmnTaskWorklistCliCommand, ConstructCliCommand, DirCliCommand, EmitCliCommand, LintCliCommand,
    ShowCliTarget, TemplateCliCommand, parse_bpmn_command, parse_construct_command,
    parse_dir_command, parse_emit_command, parse_lint_command, parse_template_command,
    resolve_bpmn_checkpoint_store_with_env, run_bpmn_command,
    run_bpmn_run_command_with_runtime_env, run_bpmn_start_at_command_with_runtime_env,
    run_bpmn_status_command_with_runtime_env, run_bpmn_task_claim_command_with_runtime_env,
    run_bpmn_task_complete_command_with_runtime_env,
    run_bpmn_task_release_command_with_runtime_env,
    run_bpmn_task_worklist_command_with_runtime_env, run_construct_command, run_dir_command,
    run_emit_command, run_lint_command, run_template_command,
};
#[cfg(feature = "wendao-integration")]
use super::{
    ContractFeedbackCliCommand, DEFAULT_CONTRACT_FEEDBACK_TABLE_NAME, REST_DOCS_PACK_ID,
    RestDocsCliCommand, build_contract_feedback_config, build_rest_docs_collection_context,
    normalize_prj_data_home, parse_contract_feedback_command,
    run_deterministic_rest_docs_contract_feedback, run_scaffold_rest_docs_contract_feedback,
};
use crate::runtime_config::QianjiRuntimeEnv;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
#[cfg(feature = "wendao-integration")]
use xiuxian_config_core::resolve_path_from_value;

mod bpmn;
#[cfg(feature = "wendao-integration")]
mod cache_paths;
mod construct_cli;
mod control_cli;
mod dir_parsing;
mod dir_runtime;
mod emit;
mod lint;
#[cfg(feature = "wendao-integration")]
mod rest_docs;
mod template_cli;

fn to_args(values: &[&str]) -> Vec<String> {
    values.iter().map(ToString::to_string).collect()
}

fn must_ok<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

fn must_some<T>(value: Option<T>, context: &str) -> T {
    value.unwrap_or_else(|| panic!("{context}"))
}

#[cfg(feature = "wendao-integration")]
fn write_openapi_fixture(temp_dir: &TempDir) -> PathBuf {
    let path = temp_dir.path().join("openapi.yaml");
    let content = r#"
openapi: 3.1.0
paths:
  /api/search:
    get:
      responses:
        "200":
          description: ok
"#;
    must_ok(
        fs::write(&path, content),
        "should write temporary OpenAPI fixture",
    );
    path
}

#[cfg(feature = "wendao-integration")]
fn rest_docs_command(openapi_path: &Path, workspace_root: &Path) -> RestDocsCliCommand {
    RestDocsCliCommand {
        openapi_path: openapi_path.to_path_buf(),
        workspace_root: Some(workspace_root.to_path_buf()),
        storage_path: None,
        table_name: DEFAULT_CONTRACT_FEEDBACK_TABLE_NAME.to_string(),
        no_persist: true,
        live_advisory: false,
        roles: Vec::new(),
        model: None,
        temperature: None,
        cognitive_early_halt_threshold: None,
    }
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        must_ok(
            fs::create_dir_all(parent),
            "should create workdir fixture parent directories",
        );
    }
    must_ok(
        fs::write(path, content),
        "should write workdir fixture file",
    );
}

fn create_workdir_fixture(temp_dir: &TempDir) -> PathBuf {
    let workdir = temp_dir.path().join("demo-plan");
    must_ok(
        fs::create_dir_all(&workdir),
        "should create temporary workdir fixture root",
    );
    write_file(
        &workdir.join("qianji.toml"),
        r#"
version = 1

[plan]
name = "demo-plan"
surface = ["flowchart.mmd", "blueprint", "plan"]

[check]
require = ["flowchart.mmd", "blueprint", "plan", "blueprint/**/*.md", "plan/**/*.md"]
flowchart = ["blueprint", "plan"]
"#,
    );
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

fn package_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn flowhub_root() -> PathBuf {
    package_root().join("qianji-flowhub")
}

fn scenario_fixture_dir(name: &str) -> PathBuf {
    package_root().join(format!("tests/fixtures/flowhub/{name}"))
}

fn anchored_workdir_fixture_anchor() -> PathBuf {
    package_root().join("tests/fixtures/flowhub_modules/paper_deep_read_workdir/qianji.toml")
}

fn anchored_workdir_fixture_graph() -> PathBuf {
    package_root()
        .join("tests/fixtures/flowhub_modules/paper_deep_read_workdir/paper-deep-read.mmd")
}

fn anchored_workdir_fixture_scenario() -> &'static str {
    "deep_read"
}

fn create_invalid_scenario_fixture(temp_dir: &TempDir) -> PathBuf {
    let scenario_dir = temp_dir.path().join("scenario");
    must_ok(
        fs::create_dir_all(&scenario_dir),
        "should create scenario fixture root",
    );
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

#[cfg(feature = "wendao-integration")]
fn default_contract_feedback_storage_path_with(
    workspace_root: &Path,
    raw_data_home: Option<&str>,
) -> PathBuf {
    resolve_prj_data_home_with(workspace_root, raw_data_home)
        .join("xiuxian-qianji")
        .join("contract_feedback")
}

#[cfg(feature = "wendao-integration")]
fn resolve_prj_data_home_with(workspace_root: &Path, raw_data_home: Option<&str>) -> PathBuf {
    let resolved = resolve_path_from_value(Some(workspace_root), raw_data_home)
        .unwrap_or_else(|| workspace_root.join(".data"));
    normalize_prj_data_home(workspace_root, resolved)
}

fn assert_common_show_shape(rendered: &str) {
    assert!(rendered.starts_with("# "));
    assert!(rendered.contains("Location:"));
    assert!(rendered.contains("\n## "));
}
