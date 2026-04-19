use super::{
    ContractFeedbackCliCommand, DEFAULT_CONTRACT_FEEDBACK_TABLE_NAME, DirCliCommand,
    MaterializeCliTarget, REST_DOCS_PACK_ID, RestDocsCliCommand, ShowCliTarget,
    build_contract_feedback_config, build_rest_docs_collection_context,
    parse_contract_feedback_command, parse_dir_command, resolve_workspace_root,
    run_deterministic_rest_docs_contract_feedback, run_dir_command,
    run_scaffold_rest_docs_contract_feedback, sanitize_prj_cache_home,
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use xiuxian_config_core::{resolve_cache_home_from_value, resolve_project_root};

mod cache_paths;
mod dir_parsing;
mod dir_runtime;
mod rest_docs;

fn to_args(values: &[&str]) -> Vec<String> {
    values.iter().map(ToString::to_string).collect()
}

fn must_ok<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) -> T {
    result.unwrap_or_else(|error| panic!("{context}: {error}"))
}

fn must_some<T>(value: Option<T>, context: &str) -> T {
    value.unwrap_or_else(|| panic!("{context}"))
}

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

fn repo_root() -> PathBuf {
    resolve_project_root()
        .unwrap_or_else(|| panic!("workspace root should resolve from PRJ_ROOT or git ancestry"))
}

fn flowhub_root() -> PathBuf {
    repo_root().join("qianji-flowhub")
}

fn scenario_fixture_dir(name: &str) -> PathBuf {
    repo_root().join(format!(
        "packages/rust/crates/xiuxian-qianji/tests/fixtures/flowhub/{name}"
    ))
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

fn default_contract_feedback_storage_path_with(
    workspace_root: &Path,
    raw_cache_home: Option<&str>,
) -> PathBuf {
    resolve_prj_cache_home_with(workspace_root, raw_cache_home)
        .join("wendao")
        .join("contract_feedback")
}

fn resolve_prj_cache_home_with(workspace_root: &Path, raw_cache_home: Option<&str>) -> PathBuf {
    let resolved = resolve_cache_home_from_value(Some(workspace_root), raw_cache_home)
        .unwrap_or_else(|| workspace_root.join(".cache"));
    sanitize_prj_cache_home(workspace_root, resolved)
}

fn assert_common_show_shape(rendered: &str) {
    assert!(rendered.starts_with("# "));
    assert!(rendered.contains("Location:"));
    assert!(rendered.contains("\n## "));
}
