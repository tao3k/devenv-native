use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::json;

#[derive(Clone)]
struct EmbedTestState {
    http_delay: Duration,
    http_fail: bool,
    http_fail_first: bool,
    openai_fail: bool,
    http_calls: Arc<AtomicUsize>,
    litellm_calls: Arc<AtomicUsize>,
}

pub(super) fn http_vector_score(text: &str) -> f32 {
    let score_mod = text
        .as_bytes()
        .iter()
        .fold(0_u32, |acc, byte| acc.saturating_add(u32::from(*byte)))
        % 10_000;
    let score_mod = u16::try_from(score_mod).unwrap_or(u16::MAX);
    f32::from(score_mod) / 1000.0
}

pub(super) fn http_vectors_for_texts(texts: &[String]) -> Vec<Vec<f32>> {
    texts
        .iter()
        .map(|text| vec![http_vector_score(text), 1.0_f32])
        .collect()
}

pub(super) fn openai_vectors_for_texts(texts: &[String]) -> Vec<Vec<f32>> {
    texts
        .iter()
        .map(|text| vec![http_vector_score(text), 7.0_f32])
        .collect()
}

async fn handle_embed_batch(
    State(state): State<EmbedTestState>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let call_index = state.http_calls.fetch_add(1, Ordering::Relaxed) + 1;
    tokio::time::sleep(state.http_delay).await;
    if state.http_fail || (state.http_fail_first && call_index == 1) {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "embed backend unavailable"
            })),
        );
    }
    let texts = payload
        .get("texts")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    (
        StatusCode::OK,
        Json(json!({
            "vectors": http_vectors_for_texts(&texts)
        })),
    )
}

async fn handle_litellm_embeddings(
    State(state): State<EmbedTestState>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    state.litellm_calls.fetch_add(1, Ordering::Relaxed);
    if state.openai_fail {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "openai-compatible embedding unavailable"
            })),
        );
    }
    let texts = payload
        .get("input")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    let vectors = openai_vectors_for_texts(&texts);
    let data: Vec<serde_json::Value> = vectors
        .into_iter()
        .enumerate()
        .map(|(index, embedding)| {
            json!({
                "object": "embedding",
                "index": index,
                "embedding": embedding
            })
        })
        .collect();
    (
        StatusCode::OK,
        Json(json!({
            "object": "list",
            "data": data,
            "model": "test-embed-model",
            "usage": {"prompt_tokens": 0, "total_tokens": 0}
        })),
    )
}

pub(super) type SpawnedEmbeddingServer = (String, Arc<AtomicUsize>, Arc<AtomicUsize>);

pub(super) fn require_vectors(vectors: Option<Vec<Vec<f32>>>, context: &str) -> Vec<Vec<f32>> {
    match vectors {
        Some(vectors) => vectors,
        None => panic!("{context}"),
    }
}

pub(super) async fn spawn_embedding_mock_server(
    http_delay: Duration,
    http_fail: bool,
    http_fail_first: bool,
) -> Result<Option<SpawnedEmbeddingServer>> {
    spawn_embedding_mock_server_with_openai_failure(http_delay, http_fail, http_fail_first, false)
        .await
}

pub(super) async fn spawn_embedding_mock_server_with_openai_failure(
    http_delay: Duration,
    http_fail: bool,
    http_fail_first: bool,
    openai_fail: bool,
) -> Result<Option<SpawnedEmbeddingServer>> {
    let http_calls = Arc::new(AtomicUsize::new(0));
    let litellm_calls = Arc::new(AtomicUsize::new(0));
    let state = EmbedTestState {
        http_delay,
        http_fail,
        http_fail_first,
        openai_fail,
        http_calls: Arc::clone(&http_calls),
        litellm_calls: Arc::clone(&litellm_calls),
    };
    let app = Router::new()
        .route("/embed/batch", post(handle_embed_batch))
        .route("/v1/embeddings", post(handle_litellm_embeddings))
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping embedding client tests: local socket bind is not permitted");
            return Ok(None);
        }
        Err(err) => return Err(err.into()),
    };
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Ok(Some((format!("http://{addr}"), http_calls, litellm_calls)))
}
