use crate::qianji_server_cli::cli::QianjiServerServeCommand;
use crate::qianji_server_cli::run::build_qianji_server_router;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::Value;
#[cfg(feature = "valkey")]
use std::fs;
use std::net::SocketAddr;
#[cfg(feature = "valkey")]
use std::path::Path;
use tower::util::ServiceExt;

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

pub(super) async fn router_get_json(
    command: QianjiServerServeCommand,
    uri: &str,
) -> (StatusCode, Value) {
    let router = must_ok(
        build_qianji_server_router(&command),
        "qianji-server router should build",
    );
    let response = router
        .oneshot(
            Request::builder()
                .uri(uri)
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
