use super::support::{must_err, spawn_qianji_server_router};
use crate::qianji_server_cli::cli::QianjiServerServeCommand;
use crate::qianji_server_cli::run::enforce_qianji_server_startup_readiness;
use serde_json::Value;

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_healthz_reports_valkey_default_backend() {
    let base_url = spawn_qianji_server_router(QianjiServerServeCommand {
        bind_addr: None,
        valkey_url: Some("not-a-valkey-url".to_string()),
        require_valkey_ready: None,
        flowhub_root: None,
    })
    .await;

    let response = reqwest::Client::new()
        .get(format!("{base_url}/healthz"))
        .send()
        .await
        .unwrap_or_else(|error| panic!("healthz request should send: {error}"));

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let body = response
        .json::<Value>()
        .await
        .unwrap_or_else(|error| panic!("healthz response should decode: {error}"));
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "qianji-server");
    assert_eq!(body["checkpoint_default_backend"], "valkey");
    assert_eq!(body["valkey_configured"], true);
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_startup_readiness_gate_is_opt_in() {
    enforce_qianji_server_startup_readiness(&QianjiServerServeCommand {
        bind_addr: None,
        valkey_url: Some("not-a-valkey-url".to_string()),
        require_valkey_ready: Some(false),
        flowhub_root: None,
    })
    .await
    .unwrap_or_else(|error| panic!("disabled readiness gate should not ping Valkey: {error}"));
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_startup_readiness_gate_fails_fast() {
    let error = must_err(
        enforce_qianji_server_startup_readiness(&QianjiServerServeCommand {
            bind_addr: None,
            valkey_url: Some("not-a-valkey-url".to_string()),
            require_valkey_ready: Some(true),
            flowhub_root: None,
        })
        .await,
        "enabled readiness gate should fail on invalid Valkey URL",
    );

    assert!(
        error.contains("qianji-server Valkey readiness check failed"),
        "unexpected startup readiness error: {error}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_readyz_reports_valkey_probe_failure() {
    let base_url = spawn_qianji_server_router(QianjiServerServeCommand {
        bind_addr: None,
        valkey_url: Some("not-a-valkey-url".to_string()),
        require_valkey_ready: None,
        flowhub_root: None,
    })
    .await;

    let response = reqwest::Client::new()
        .get(format!("{base_url}/readyz"))
        .send()
        .await
        .unwrap_or_else(|error| panic!("readyz request should send: {error}"));

    assert_eq!(response.status(), reqwest::StatusCode::SERVICE_UNAVAILABLE);
    let body = response
        .json::<Value>()
        .await
        .unwrap_or_else(|error| panic!("readyz response should decode: {error}"));
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
