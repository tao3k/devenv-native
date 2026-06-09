use crate::qianji_server_cli::cli::QianjiServerServeCommand;
use crate::qianji_server_cli::run::build_qianji_server_router_with_internal_security_and_runtime_env;
use crate::qianji_server_cli::security::QianjiInternalServiceSecurity;
use crate::runtime_config::QianjiRuntimeEnv;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::Value;
#[cfg(feature = "valkey")]
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tower::util::ServiceExt;
use xiuxian_security::{
    PublicProtocolSurface, SignedPrincipalSigner, WENDAO_AUTH_SCOPE_HEADER,
    WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY, WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
    WENDAO_PUBLIC_PROTOCOL_HEADER, WENDAO_SIGNED_PRINCIPAL_HEADER,
};

pub(super) fn must_ok<T, E>(result: Result<T, E>, context: &str) -> T
where
    E: std::fmt::Display,
{
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

pub(super) fn must_err<T, E>(result: Result<T, E>, context: &str) -> String
where
    E: std::fmt::Display,
{
    match result {
        Ok(_) => panic!("{context}: expected error"),
        Err(error) => error.to_string(),
    }
}

pub(super) fn must_parse_addr(value: &str) -> SocketAddr {
    match value.parse() {
        Ok(addr) => addr,
        Err(error) => panic!("bind address should be valid: {error}"),
    }
}

#[cfg(feature = "valkey")]
pub(super) fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("fixture parent should be created: {error}"));
    }
    fs::write(path, content).unwrap_or_else(|error| panic!("fixture file should write: {error}"));
}

pub(super) fn test_internal_service_security() -> QianjiInternalServiceSecurity {
    QianjiInternalServiceSecurity::gateway(
        Arc::<str>::from("internal-secret"),
        Arc::<str>::from("QIANJI_INTERNAL_PRINCIPAL_REQUIRED"),
    )
}

pub(super) fn build_test_qianji_server_router(
    command: &QianjiServerServeCommand,
    context: &str,
) -> Router {
    must_ok(
        build_qianji_server_router_with_internal_security_and_runtime_env(
            command,
            Some(test_internal_service_security()),
            test_qianji_runtime_env(command),
        ),
        context,
    )
}

pub(super) fn test_qianji_runtime_env(command: &QianjiServerServeCommand) -> QianjiRuntimeEnv {
    let project_root = command
        .control_ledger_path
        .as_ref()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .or_else(|| {
            command
                .flowhub_root
                .as_ref()
                .and_then(|path| path.parent().map(Path::to_path_buf))
        })
        .unwrap_or_else(|| PathBuf::from("."));
    QianjiRuntimeEnv {
        prj_root: Some(project_root.clone()),
        prj_data_home: Some(project_root.join(".data")),
        qianji_checkpoint_valkey_url: command.valkey_url.clone(),
        openai_api_base: Some("http://127.0.0.1:1/v1".to_string()),
        openai_api_key: Some("qianji-server-test-key".to_string()),
        qianji_llm_model: Some("openai-compatible/qianji-test-model".to_string()),
        qianji_llm_wire_api: Some("chat_completions".to_string()),
        ..QianjiRuntimeEnv::default()
    }
}

pub(super) fn with_test_internal_service_headers(
    builder: axum::http::request::Builder,
) -> axum::http::request::Builder {
    let surface = PublicProtocolSurface::HttpsJsonSse;
    let signed_principal = SignedPrincipalSigner::new(
        Arc::<str>::from(WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY),
        Arc::<str>::from("internal-secret"),
    )
    .sign_user_token(surface, "public-token");

    builder
        .header(
            WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
            WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY,
        )
        .header(WENDAO_PUBLIC_PROTOCOL_HEADER, surface.protocol())
        .header(WENDAO_AUTH_SCOPE_HEADER, surface.scope())
        .header(WENDAO_SIGNED_PRINCIPAL_HEADER, signed_principal)
}

pub(super) async fn router_get_json(
    command: QianjiServerServeCommand,
    uri: &str,
) -> (StatusCode, Value) {
    let router = build_test_qianji_server_router(&command, "qianji-server router should build");
    let response = router
        .oneshot(
            with_test_internal_service_headers(Request::builder().uri(uri))
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("GET request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("qianji-server route should respond: {error}"));
    let status = response.status();
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|error| panic!("response body should read: {error}"));
    let json = serde_json::from_slice(&body)
        .unwrap_or_else(|error| panic!("response body should decode as JSON: {error}"));
    (status, json)
}
