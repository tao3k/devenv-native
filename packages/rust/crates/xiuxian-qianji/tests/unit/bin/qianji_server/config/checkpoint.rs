use super::support::{must_ok, server_command, write_qianji_server_config};
use crate::qianji_server_cli::run::{
    build_workflow_control_service, resolve_qianji_server_control_ledger_path_with_env,
    resolve_qianji_server_valkey_url_with_env,
};
use crate::runtime_config::QianjiRuntimeEnv;
use crate::{QianjiBpmnCheckpointStore, QianjiBpmnWorkflowCheckpointBackend};

use tempfile::TempDir;

#[test]
fn qianji_server_valkey_url_overrides_runtime_checkpoint_store() {
    let mut command = server_command();
    command.valkey_url = Some("redis://127.0.0.1:6382/0".to_string());
    let service = build_workflow_control_service(&command);
    let store = must_ok(
        service.resolve_checkpoint_store(Some(&QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey)),
        "server-owned valkey URL should resolve checkpoint store",
    );

    match store {
        Some(QianjiBpmnCheckpointStore::Valkey { url }) => {
            assert_eq!(url, "redis://127.0.0.1:6382/0");
        }
        other => panic!("expected Valkey checkpoint store, got {other:?}"),
    }
}

#[test]
fn qianji_server_valkey_url_resolves_from_qianji_toml() {
    let (project_root, config_home) = write_qianji_server_config(
        r#"
[checkpoint]
valkey_url = "redis://127.0.0.1:6383/0"
"#,
    );

    let command = server_command();
    let valkey_url = must_ok(
        resolve_qianji_server_valkey_url_with_env(
            &command,
            &QianjiRuntimeEnv {
                prj_root: Some(project_root),
                prj_config_home: Some(config_home),
                ..QianjiRuntimeEnv::default()
            },
        ),
        "qianji-server Valkey URL should resolve from qianji.toml",
    );

    assert_eq!(valkey_url, "redis://127.0.0.1:6383/0");
}

#[test]
fn qianji_server_cli_valkey_url_overrides_qianji_toml() {
    let (project_root, config_home) = write_qianji_server_config(
        r#"
[checkpoint]
valkey_url = "redis://127.0.0.1:6383/0"
"#,
    );

    let mut command = server_command();
    command.valkey_url = Some("redis://127.0.0.1:6384/0".to_string());
    let valkey_url = must_ok(
        resolve_qianji_server_valkey_url_with_env(
            &command,
            &QianjiRuntimeEnv {
                prj_root: Some(project_root),
                prj_config_home: Some(config_home),
                ..QianjiRuntimeEnv::default()
            },
        ),
        "CLI Valkey URL should override qianji.toml",
    );

    assert_eq!(valkey_url, "redis://127.0.0.1:6384/0");
}

#[test]
fn qianji_server_control_ledger_defaults_under_prj_data_home() {
    let temp_dir = TempDir::new()
        .unwrap_or_else(|error| panic!("temp dir should allocate for ledger test: {error}"));
    let project_root = temp_dir.path().join("project");
    let data_home = project_root.join(".data");
    let command = server_command();

    let ledger_path = must_ok(
        resolve_qianji_server_control_ledger_path_with_env(
            &command,
            &QianjiRuntimeEnv {
                prj_root: Some(project_root.clone()),
                prj_data_home: Some(data_home.clone()),
                ..QianjiRuntimeEnv::default()
            },
            false,
        ),
        "default control ledger path should resolve",
    );

    assert_eq!(
        ledger_path,
        data_home.join("xiuxian-qianji/duckdb/control-ledger.duckdb")
    );
}
