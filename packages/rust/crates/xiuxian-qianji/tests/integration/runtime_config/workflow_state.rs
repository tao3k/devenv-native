use super::{resolve_workflow_state, write_file};
use std::path::PathBuf;
use tempfile::TempDir;
use xiuxian_qianji::runtime_config::QianjiRuntimeEnv;

#[test]
fn runtime_workflow_state_config_uses_system_default_path() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    let cfg = resolve_workflow_state(&QianjiRuntimeEnv {
        prj_root: Some(project_root.clone()),
        prj_config_home: Some(config_home),
        prj_data_home: Some(project_root.join(".data")),
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(
        cfg.local_duckdb_path,
        project_root.join(".data/xiuxian-qianji/duckdb/workflow-state.duckdb")
    );
}

#[test]
fn runtime_workflow_state_config_uses_prj_data_home_namespace() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    let cfg = resolve_workflow_state(&QianjiRuntimeEnv {
        prj_root: Some(project_root.clone()),
        prj_config_home: Some(config_home),
        extra_env: vec![("PRJ_DATA_HOME".to_string(), ".runtime/data".to_string())],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(
        cfg.local_duckdb_path,
        project_root.join(".runtime/data/xiuxian-qianji/duckdb/workflow-state.duckdb")
    );
}

#[test]
fn runtime_workflow_state_config_prefers_qianji_toml_over_env_fallback() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[workflow_state]
local_duckdb_path = ".run/system-state.duckdb"
"#,
    );

    let cfg = resolve_workflow_state(&QianjiRuntimeEnv {
        prj_root: Some(project_root.clone()),
        prj_config_home: Some(config_home),
        extra_env: vec![(
            "QIANJI_WORKFLOW_STATE_DUCKDB_PATH".to_string(),
            ".run/env-state.duckdb".to_string(),
        )],
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(
        cfg.local_duckdb_path,
        project_root.join(".run/system-state.duckdb")
    );
}

#[test]
fn runtime_workflow_state_config_runtime_env_overrides_file() {
    let tmp = TempDir::new()
        .unwrap_or_else(|err| panic!("failed to create temp dir for runtime config test: {err}"));
    let project_root = tmp.path().join("project");
    let config_home = project_root.join(".config");

    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        r#"
[workflow_state]
local_duckdb_path = ".run/system-state.duckdb"
"#,
    );

    let cfg = resolve_workflow_state(&QianjiRuntimeEnv {
        prj_root: Some(project_root.clone()),
        prj_config_home: Some(config_home),
        qianji_workflow_state_duckdb_path: Some(PathBuf::from(".run/override-state.duckdb")),
        ..QianjiRuntimeEnv::default()
    });

    assert_eq!(
        cfg.local_duckdb_path,
        project_root.join(".run/override-state.duckdb")
    );
}
