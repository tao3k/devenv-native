use super::support::{must_err, router_get_json};
use crate::qianji_server_cli::cli::QianjiServerServeCommand;
use crate::qianji_server_cli::run::enforce_qianji_server_startup_readiness;
use axum::http::StatusCode;
use serde_json::Value;

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_healthz_reports_valkey_default_backend() {
    let (status, body) = router_get_json(default_command(), "/healthz").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "qianji-server");
    assert_eq!(body["checkpoint_default_backend"], "valkey");
    assert_eq!(body["valkey_configured"], true);
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_capabilities_reports_workflow_control_routes() {
    let (status, body) = router_get_json(default_command(), "/capabilities").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["service"], "qianji-server");
    assert_eq!(body["checkpoint_default_backend"], "valkey");
    let capabilities = body["capabilities"]
        .as_array()
        .unwrap_or_else(|| panic!("capabilities should be an array: {body}"));
    for capability in [
        "bpmn.workflow.task.complete-batch",
        "bpmn.workflow.task.fail",
        "qianji.control.workflow-source.admit",
        "qianji.control.bpmn-source.admit",
        "qianji.control.bpmn-source",
        "qianji.control.execution-graph",
        "qianji.control.history",
        "qianji.control.summary",
        "qianji.control.recovery",
        "qianji.control.diagnostics",
        "flowhub.scenarios",
    ] {
        assert_has_capability(capabilities, capability, &body);
    }
    #[cfg(feature = "duckdb")]
    assert_has_capability(
        capabilities,
        "qianji.control.run-console.arrow-flight",
        &body,
    );
    #[cfg(feature = "valkey")]
    assert_has_capability(capabilities, "qianji.control.recovery.apply", &body);
    #[cfg(all(feature = "duckdb", feature = "valkey", feature = "qianji-full"))]
    assert_has_capability(
        capabilities,
        "qianji.control.worker.openai-compatible-llm.run",
        &body,
    );
    #[cfg(all(feature = "duckdb", feature = "valkey", feature = "qianji-full"))]
    assert_has_capability(
        capabilities,
        "qianji.control.worker.openai-compatible-llm.run-and-complete",
        &body,
    );
    #[cfg(not(feature = "valkey"))]
    assert!(
        capabilities
            .iter()
            .all(|capability| capability != "qianji.control.recovery.apply"),
        "capabilities should not advertise control recovery apply without Valkey hot-state: {body}"
    );
}

fn assert_has_capability(capabilities: &[Value], expected: &str, body: &Value) {
    assert!(
        capabilities.iter().any(|capability| capability == expected),
        "capabilities should include {expected}: {body}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_startup_readiness_gate_is_opt_in() {
    enforce_qianji_server_startup_readiness(&QianjiServerServeCommand {
        bind_addr: None,
        flight_bind_addr: None,
        valkey_url: Some("not-a-valkey-url".to_string()),
        require_valkey_ready: Some(false),
        flowhub_root: None,
        control_ledger_path: None,
    })
    .await
    .unwrap_or_else(|error| panic!("disabled readiness gate should not ping Valkey: {error}"));
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_startup_readiness_gate_fails_fast() {
    let error = must_err(
        enforce_qianji_server_startup_readiness(&QianjiServerServeCommand {
            bind_addr: None,
            flight_bind_addr: None,
            valkey_url: Some("not-a-valkey-url".to_string()),
            require_valkey_ready: Some(true),
            flowhub_root: None,
            control_ledger_path: None,
        })
        .await,
        "enabled readiness gate should fail on invalid Valkey URL",
    );

    assert!(
        error.contains(expected_readiness_failure()),
        "unexpected startup readiness error: {error}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_readyz_reports_valkey_probe_failure() {
    let (status, body) = router_get_json(default_command(), "/readyz").await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["status"], "not_ready");
    assert_eq!(body["service"], "qianji-server");
    assert_eq!(body["checkpoint_default_backend"], "valkey");
    assert_eq!(body["valkey"]["status"], "not_ready");
    assert!(
        body["valkey"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("Valkey")),
        "unexpected readyz response: {body}"
    );
}

fn default_command() -> QianjiServerServeCommand {
    QianjiServerServeCommand {
        bind_addr: None,
        flight_bind_addr: None,
        valkey_url: Some("not-a-valkey-url".to_string()),
        require_valkey_ready: None,
        flowhub_root: None,
        control_ledger_path: None,
    }
}

#[cfg(feature = "valkey")]
fn expected_readiness_failure() -> &'static str {
    "qianji-server Valkey readiness check failed"
}

#[cfg(not(feature = "valkey"))]
fn expected_readiness_failure() -> &'static str {
    "qianji-server Valkey readiness requires the `valkey` feature"
}
