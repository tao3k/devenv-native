use crate::qianji_server_cli::cli::QianjiServerServeCommand;
use crate::qianji_server_cli::run::build_workflow_http_state;
use std::{fs, net::SocketAddr, path::Path};
use tempfile::TempDir;

pub(super) fn must_ok<T, E>(result: Result<T, E>, context: &str) -> T
where
    E: std::fmt::Display,
{
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

pub(super) fn must_parse_addr(value: &str) -> SocketAddr {
    match value.parse() {
        Ok(addr) => addr,
        Err(error) => panic!("bind address should be valid: {error}"),
    }
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("fixture parent should be created: {error}"));
    }
    fs::write(path, content).unwrap_or_else(|error| panic!("fixture file should write: {error}"));
}

pub(super) fn server_command() -> QianjiServerServeCommand {
    QianjiServerServeCommand {
        bind_addr: None,
        flight_bind_addr: None,
        valkey_url: None,
        require_valkey_ready: None,
        flowhub_root: None,
        control_ledger_path: None,
    }
}

#[cfg(feature = "valkey")]
pub(super) fn build_test_workflow_http_state(
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
pub(super) fn build_test_workflow_http_state(
    service: crate::QianjiBpmnWorkflowControlService,
    host: crate::QianjiBpmnHostBridge,
    command: &QianjiServerServeCommand,
    control_ledger: Option<crate::qianji_server_cli::run::SharedControlLedger>,
) -> crate::QianjiBpmnWorkflowHttpState<crate::QianjiBpmnHostBridge> {
    build_workflow_http_state(service, host, command, control_ledger)
}

pub(super) fn write_qianji_server_config(
    content: &str,
) -> (std::path::PathBuf, std::path::PathBuf) {
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
