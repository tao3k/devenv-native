use super::support::{must_ok, must_parse_addr, write_file};
use crate::cli::QianjiServerServeCommand;
use crate::run::{
    build_workflow_control_service, resolve_qianji_server_bind_addr_with_env,
    resolve_qianji_server_require_valkey_ready_with_env, resolve_qianji_server_valkey_url_with_env,
};
use tempfile::TempDir;
use xiuxian_qianji::runtime_config::QianjiRuntimeEnv;
use xiuxian_qianji::{QianjiBpmnCheckpointStore, QianjiBpmnWorkflowCheckpointBackend};

#[test]
fn qianji_server_valkey_url_overrides_runtime_checkpoint_store() {
    let command = QianjiServerServeCommand {
        bind_addr: None,
        valkey_url: Some("redis://127.0.0.1:6382/0".to_string()),
        require_valkey_ready: None,
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
fn qianji_server_valkey_url_resolves_from_qianji_toml() {
    let (project_root, config_home) = write_qianji_server_config(
        r#"
[checkpoint]
valkey_url = "redis://127.0.0.1:6383/0"
"#,
    );

    let command = QianjiServerServeCommand {
        bind_addr: None,
        valkey_url: None,
        require_valkey_ready: None,
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
        valkey_url: Some("redis://127.0.0.1:6384/0".to_string()),
        require_valkey_ready: None,
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
        valkey_url: None,
        require_valkey_ready: None,
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
        valkey_url: None,
        require_valkey_ready: None,
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
fn qianji_server_require_valkey_ready_resolves_from_qianji_toml() {
    let (project_root, config_home) = write_qianji_server_config(
        r"
[server]
require_valkey_ready = true
",
    );

    let command = QianjiServerServeCommand {
        bind_addr: None,
        valkey_url: None,
        require_valkey_ready: None,
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
        valkey_url: None,
        require_valkey_ready: Some(false),
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
