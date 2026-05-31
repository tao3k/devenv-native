use super::support::{
    build_test_workflow_http_state, must_ok, server_command, write_qianji_server_config,
};
use crate::QianjiBpmnHostBridge;
use crate::qianji_server_cli::run::{
    build_workflow_control_service, resolve_qianji_server_require_valkey_ready_with_env,
};
use crate::runtime_config::QianjiRuntimeEnv;

#[test]
fn qianji_server_http_state_installs_default_runtime_env_for_llm_admission() {
    let mut command = server_command();
    command.valkey_url = Some("redis://127.0.0.1:6382/0".to_string());
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

#[test]
fn qianji_server_require_valkey_ready_resolves_from_qianji_toml() {
    let (project_root, config_home) = write_qianji_server_config(
        r"
[server]
require_valkey_ready = true
",
    );

    let command = server_command();
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

    let mut command = server_command();
    command.require_valkey_ready = Some(false);
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
