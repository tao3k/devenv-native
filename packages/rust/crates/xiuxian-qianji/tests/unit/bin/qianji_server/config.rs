use super::support::{must_ok, must_parse_addr, write_file};
use crate::qianji_server_cli::cli::QianjiServerServeCommand;
use crate::qianji_server_cli::run::{
    build_workflow_control_service, build_workflow_http_state,
    resolve_qianji_server_bind_addr_with_env, resolve_qianji_server_control_ledger_path_with_env,
    resolve_qianji_server_flight_bind_addr_with_env,
    resolve_qianji_server_require_valkey_ready_with_env, resolve_qianji_server_valkey_url_with_env,
};
use crate::runtime_config::QianjiRuntimeEnv;
use crate::{QianjiBpmnCheckpointStore, QianjiBpmnHostBridge, QianjiBpmnWorkflowCheckpointBackend};
use tempfile::TempDir;

#[test]
fn qianji_server_valkey_url_overrides_runtime_checkpoint_store() {
    let command = QianjiServerServeCommand {
        bind_addr: None,
        flight_bind_addr: None,
        valkey_url: Some("redis://127.0.0.1:6382/0".to_string()),
        require_valkey_ready: None,
        flowhub_root: None,
        control_ledger_path: None,
    };
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
fn qianji_server_http_state_installs_default_runtime_env_for_llm_admission() {
    let command = QianjiServerServeCommand {
        bind_addr: None,
        flight_bind_addr: None,
        valkey_url: Some("redis://127.0.0.1:6382/0".to_string()),
        require_valkey_ready: None,
        flowhub_root: None,
        control_ledger_path: None,
    };
    let state = build_test_workflow_http_state(
        build_workflow_control_service(&command),
        QianjiBpmnHostBridge::default(),
        &command,
        None,
    );

    assert!(
        state.runtime_env.is_some(),
        "qianji-server must install default runtime env so server-owned LLM admission does not get skipped"
    );
}

#[cfg(feature = "valkey")]
fn build_test_workflow_http_state(
    service: crate::QianjiBpmnWorkflowControlService,
    host: crate::QianjiBpmnHostBridge,
    command: &QianjiServerServeCommand,
    control_ledger: Option<crate::qianji_server_cli::run::SharedControlLedger>,
) -> crate::QianjiBpmnWorkflowHttpState<crate::QianjiBpmnHostBridge> {
    must_ok(
        build_workflow_http_state(service, host, command, control_ledger),
        "qianji-server HTTP state should build",
    )
}

#[cfg(not(feature = "valkey"))]
fn build_test_workflow_http_state(
    service: crate::QianjiBpmnWorkflowControlService,
    host: crate::QianjiBpmnHostBridge,
    command: &QianjiServerServeCommand,
    control_ledger: Option<crate::qianji_server_cli::run::SharedControlLedger>,
) -> crate::QianjiBpmnWorkflowHttpState<crate::QianjiBpmnHostBridge> {
    build_workflow_http_state(service, host, command, control_ledger)
}

#[test]
fn qianji_server_valkey_url_resolves_from_qianji_toml() {
    let (project_root, config_home) = write_qianji_server_config(
        r#"
[checkpoint]
valkey_url = "redis://127.0.0.1:6383/0"
"#,
    );

    let command = QianjiServerServeCommand {
        bind_addr: None,
        flight_bind_addr: None,
        valkey_url: None,
        require_valkey_ready: None,
        flowhub_root: None,
        control_ledger_path: None,
    };
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

    let command = QianjiServerServeCommand {
        bind_addr: None,
        flight_bind_addr: None,
        valkey_url: Some("redis://127.0.0.1:6384/0".to_string()),
        require_valkey_ready: None,
        flowhub_root: None,
        control_ledger_path: None,
    };
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
fn qianji_server_bind_addr_resolves_from_qianji_toml() {
    let (project_root, config_home) = write_qianji_server_config(
        r#"
[server]
bind_addr = "127.0.0.1:38131"
"#,
    );

    let command = QianjiServerServeCommand {
        bind_addr: None,
        flight_bind_addr: None,
        valkey_url: None,
        require_valkey_ready: None,
        flowhub_root: None,
        control_ledger_path: None,
    };
    let bind_addr = must_ok(
        resolve_qianji_server_bind_addr_with_env(
            &command,
            &QianjiRuntimeEnv {
                prj_root: Some(project_root),
                prj_config_home: Some(config_home),
                ..QianjiRuntimeEnv::default()
            },
        ),
        "qianji-server bind address should resolve from qianji.toml",
    );

    assert_eq!(bind_addr, must_parse_addr("127.0.0.1:38131"));
}

#[test]
fn qianji_server_cli_bind_overrides_qianji_toml() {
    let (project_root, config_home) = write_qianji_server_config(
        r#"
[server]
bind_addr = "127.0.0.1:38131"
"#,
    );

    let command = QianjiServerServeCommand {
        bind_addr: Some(must_parse_addr("127.0.0.1:38132")),
        flight_bind_addr: None,
        valkey_url: None,
        require_valkey_ready: None,
        flowhub_root: None,
        control_ledger_path: None,
    };
    let bind_addr = must_ok(
        resolve_qianji_server_bind_addr_with_env(
            &command,
            &QianjiRuntimeEnv {
                prj_root: Some(project_root),
                prj_config_home: Some(config_home),
                ..QianjiRuntimeEnv::default()
            },
        ),
        "CLI bind should override qianji.toml",
    );

    assert_eq!(bind_addr, must_parse_addr("127.0.0.1:38132"));
}

#[test]
fn qianji_server_flight_bind_addr_defaults_from_runtime_config() {
    let command = QianjiServerServeCommand {
        bind_addr: None,
        flight_bind_addr: None,
        valkey_url: None,
        require_valkey_ready: None,
        flowhub_root: None,
        control_ledger_path: None,
    };
    let flight_bind_addr = must_ok(
        resolve_qianji_server_flight_bind_addr_with_env(&command, &QianjiRuntimeEnv::default()),
        "qianji-server Flight bind address should resolve by default",
    );

    assert_eq!(flight_bind_addr, Some(must_parse_addr("127.0.0.1:38131")));
}

#[test]
fn qianji_server_flight_bind_addr_resolves_from_qianji_toml() {
    let (project_root, config_home) = write_qianji_server_config(
        r#"
[server]
flight_bind_addr = "127.0.0.1:38136"
"#,
    );

    let command = QianjiServerServeCommand {
        bind_addr: None,
        flight_bind_addr: None,
        valkey_url: None,
        require_valkey_ready: None,
        flowhub_root: None,
        control_ledger_path: None,
    };
    let flight_bind_addr = must_ok(
        resolve_qianji_server_flight_bind_addr_with_env(
            &command,
            &QianjiRuntimeEnv {
                prj_root: Some(project_root),
                prj_config_home: Some(config_home),
                ..QianjiRuntimeEnv::default()
            },
        ),
        "qianji-server Flight bind address should resolve from qianji.toml",
    );

    assert_eq!(flight_bind_addr, Some(must_parse_addr("127.0.0.1:38136")));
}

#[test]
fn qianji_server_cli_flight_bind_overrides_qianji_toml() {
    let (project_root, config_home) = write_qianji_server_config(
        r#"
[server]
flight_bind_addr = "127.0.0.1:38136"
"#,
    );

    let command = QianjiServerServeCommand {
        bind_addr: None,
        flight_bind_addr: Some(must_parse_addr("127.0.0.1:38137")),
        valkey_url: None,
        require_valkey_ready: None,
        flowhub_root: None,
        control_ledger_path: None,
    };
    let flight_bind_addr = must_ok(
        resolve_qianji_server_flight_bind_addr_with_env(
            &command,
            &QianjiRuntimeEnv {
                prj_root: Some(project_root),
                prj_config_home: Some(config_home),
                ..QianjiRuntimeEnv::default()
            },
        ),
        "CLI Flight bind should override qianji.toml",
    );

    assert_eq!(flight_bind_addr, Some(must_parse_addr("127.0.0.1:38137")));
}

#[test]
fn qianji_server_control_ledger_defaults_under_prj_data_home() {
    let temp_dir = TempDir::new()
        .unwrap_or_else(|error| panic!("temp dir should allocate for ledger test: {error}"));
    let project_root = temp_dir.path().join("project");
    let data_home = project_root.join(".data");
    let command = QianjiServerServeCommand {
        bind_addr: None,
        flight_bind_addr: None,
        valkey_url: None,
        require_valkey_ready: None,
        flowhub_root: None,
        control_ledger_path: None,
    };

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

#[test]
fn qianji_server_require_valkey_ready_resolves_from_qianji_toml() {
    let (project_root, config_home) = write_qianji_server_config(
        r"
[server]
require_valkey_ready = true
",
    );

    let command = QianjiServerServeCommand {
        bind_addr: None,
        flight_bind_addr: None,
        valkey_url: None,
        require_valkey_ready: None,
        flowhub_root: None,
        control_ledger_path: None,
    };
    let require_valkey_ready = must_ok(
        resolve_qianji_server_require_valkey_ready_with_env(
            &command,
            &QianjiRuntimeEnv {
                prj_root: Some(project_root),
                prj_config_home: Some(config_home),
                ..QianjiRuntimeEnv::default()
            },
        ),
        "qianji-server readiness gate should resolve from qianji.toml",
    );

    assert!(require_valkey_ready);
}

#[test]
fn qianji_server_cli_readiness_gate_overrides_qianji_toml() {
    let (project_root, config_home) = write_qianji_server_config(
        r"
[server]
require_valkey_ready = true
",
    );

    let command = QianjiServerServeCommand {
        bind_addr: None,
        flight_bind_addr: None,
        valkey_url: None,
        require_valkey_ready: Some(false),
        flowhub_root: None,
        control_ledger_path: None,
    };
    let require_valkey_ready = must_ok(
        resolve_qianji_server_require_valkey_ready_with_env(
            &command,
            &QianjiRuntimeEnv {
                prj_root: Some(project_root),
                prj_config_home: Some(config_home),
                ..QianjiRuntimeEnv::default()
            },
        ),
        "CLI readiness gate should override qianji.toml",
    );

    assert!(!require_valkey_ready);
}

fn write_qianji_server_config(content: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let temp_dir = TempDir::new()
        .unwrap_or_else(|error| panic!("temp dir should allocate for qianji-server test: {error}"));
    let project_root = temp_dir.keep().join("project");
    let config_home = project_root.join(".config");
    write_file(
        &project_root.join("packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml"),
        content,
    );
    (project_root, config_home)
}
