//! Integration tests for OpenAI-compatible embedding transport behavior.
//!
//! Covers valid JSON decoding and non-JSON success payload handling.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc
)]

use axum::Router;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::routing::post;
use xiuxian_llm::embedding::openai_compat::embed_openai_compatible;

#[derive(Clone)]
struct OpenAiCompatMockState {
    status: StatusCode,
    content_type: &'static str,
    body: &'static str,
}

async fn handle_embeddings(
    State(state): State<OpenAiCompatMockState>,
) -> (
    StatusCode,
    [(header::HeaderName, &'static str); 1],
    &'static str,
) {
    (
        state.status,
        [(header::CONTENT_TYPE, state.content_type)],
        state.body,
    )
}

async fn spawn_openai_compat_server(state: OpenAiCompatMockState) -> Option<String> {
    let app = Router::new()
        .route("/v1/embeddings", post(handle_embeddings))
        .with_state(state);
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => return None,
        Err(err) => panic!("failed to bind test server: {err}"),
    };
    let addr = listener
        .local_addr()
        .unwrap_or_else(|_| panic!("test server should expose a local address"));
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Some(format!("http://{addr}"))
}

#[tokio::test]
async fn embed_openai_compatible_parses_valid_json_payload() {
    let Some(base_url) = spawn_openai_compat_server(OpenAiCompatMockState {
        status: StatusCode::OK,
        content_type: "application/json",
        body: r#"{"data":[{"embedding":[0.1,0.2,0.3]}]}"#,
    })
    .await
    else {
        return;
    };

    let client = reqwest::Client::new();
    let texts = vec!["hello".to_string()];

    let vectors =
        embed_openai_compatible(&client, &base_url, &texts, Some("qwen3-embedding:0.6b")).await;

    assert_eq!(vectors, Some(vec![vec![0.1_f32, 0.2_f32, 0.3_f32]]));
}

#[tokio::test]
async fn embed_openai_compatible_returns_none_on_non_json_success_payload() {
    let Some(base_url) = spawn_openai_compat_server(OpenAiCompatMockState {
        status: StatusCode::OK,
        content_type: "text/plain",
        body: "service unavailable",
    })
    .await
    else {
        return;
    };

    let client = reqwest::Client::new();
    let texts = vec!["hello".to_string()];

    let vectors =
        embed_openai_compatible(&client, &base_url, &texts, Some("qwen3-embedding:0.6b")).await;

    assert!(vectors.is_none());
}
