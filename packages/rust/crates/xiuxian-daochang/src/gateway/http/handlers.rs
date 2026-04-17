use axum::Extension;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;

use super::runtime::resolve_embed_model;
use super::types::{
    EmbedBatchRequest, EmbedBatchResponse, EmbedRequest, EmbedResponse, GatewayEmbeddingRuntime,
    GatewayExternalToolHealthResponse, GatewayHealthResponse, GatewayJsonError, GatewayJsonResult,
    GatewayState, MessageRequest, MessageResponse, OpenAiEmbeddingData, OpenAiEmbeddingUsage,
    OpenAiEmbeddingsRequest, OpenAiEmbeddingsResponse,
};

#[must_use]
pub(crate) fn fallback_hash_embed_batch(inputs: &[String], dimension: usize) -> Vec<Vec<f32>> {
    inputs
        .iter()
        .map(|input| fallback_hash_embed(input, dimension))
        .collect()
}

fn fallback_hash_embed(input: &str, dimension: usize) -> Vec<f32> {
    let mut vector = Vec::with_capacity(dimension);
    let mut counter = 0u64;
    while vector.len() < dimension {
        let mut digest = Sha256::new();
        digest.update(input.as_bytes());
        digest.update(counter.to_le_bytes());
        let bytes = digest.finalize();
        for chunk in bytes.chunks_exact(4) {
            if vector.len() == dimension {
                break;
            }
            let raw = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let normalized = f32::from_bits(0x3F80_0000 | (raw >> 9)) - 1.0;
            vector.push(normalized * 2.0 - 1.0);
        }
        counter = counter.saturating_add(1);
    }
    vector
}

/// Validate request body; returns error for empty `session_id` or message.
///
/// # Errors
/// Returns an HTTP 400 tuple when request fields are empty after trimming.
pub fn validate_message_request(
    body: &MessageRequest,
) -> Result<(String, String), (StatusCode, String)> {
    let session_id = body.session_id.trim().to_string();
    let message = body.message.trim().to_string();
    if session_id.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "session_id must be non-empty".to_string(),
        ));
    }
    if message.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "message must be non-empty".to_string(),
        ));
    }
    Ok((session_id, message))
}

pub(super) async fn handle_message(
    State(state): State<GatewayState>,
    Json(body): Json<MessageRequest>,
) -> GatewayJsonResult<MessageResponse> {
    let (session_id, message) = validate_message_request(&body)?;
    let _permit = if let Some(ref sem) = state.concurrency_semaphore {
        Some(sem.acquire().await.map_err(|_| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "concurrency limit closed".to_string(),
            )
        })?)
    } else {
        None
    };

    let timeout_secs = state.turn_timeout_secs;
    let output = match tokio::time::timeout(
        Duration::from_secs(timeout_secs),
        state.agent.run_turn(&session_id, &message),
    )
    .await
    {
        Ok(Ok(out)) => out,
        Ok(Err(error)) => return Err((StatusCode::INTERNAL_SERVER_ERROR, error.to_string())),
        Err(_) => {
            return Err((
                StatusCode::GATEWAY_TIMEOUT,
                format!("agent turn timed out after {timeout_secs}s"),
            ));
        }
    };

    Ok(Json(MessageResponse { output, session_id }))
}

fn parse_openai_embedding_input(input: &Value) -> Result<Vec<String>, GatewayJsonError> {
    match input {
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                Err((
                    StatusCode::BAD_REQUEST,
                    "input text must be non-empty".to_string(),
                ))
            } else {
                Ok(vec![trimmed.to_string()])
            }
        }
        Value::Array(items) => {
            if items.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "input array must be non-empty".to_string(),
                ));
            }

            let mut texts = Vec::with_capacity(items.len());
            for item in items {
                let Some(text) = item.as_str() else {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "input array must contain non-empty strings".to_string(),
                    ));
                };
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "input array must contain non-empty strings".to_string(),
                    ));
                }
                texts.push(trimmed.to_string());
            }
            Ok(texts)
        }
        _ => Err((
            StatusCode::BAD_REQUEST,
            "input must be a string or an array of strings".to_string(),
        )),
    }
}

pub(super) async fn handle_embed_batch(
    Extension(embedding_runtime): Extension<Arc<GatewayEmbeddingRuntime>>,
    Json(body): Json<EmbedBatchRequest>,
) -> GatewayJsonResult<EmbedBatchResponse> {
    if body.texts.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "texts must be a non-empty array".to_string(),
        ));
    }

    let model = resolve_embed_model(
        body.model.as_deref(),
        embedding_runtime.default_model.as_deref(),
    )?;

    let vectors = embedding_runtime
        .client
        .embed_batch_with_model(&body.texts, Some(model.as_str()))
        .await
        .ok_or((
            StatusCode::SERVICE_UNAVAILABLE,
            "embedding backend unavailable".to_string(),
        ))?;

    if vectors.len() != body.texts.len() {
        return Err((
            StatusCode::BAD_GATEWAY,
            "embedding backend returned invalid vector count".to_string(),
        ));
    }

    Ok(Json(EmbedBatchResponse { vectors }))
}

pub(super) async fn handle_embed(
    Extension(embedding_runtime): Extension<Arc<GatewayEmbeddingRuntime>>,
    Json(body): Json<EmbedRequest>,
) -> GatewayJsonResult<EmbedResponse> {
    let text = body.text.trim();
    if text.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "text must be non-empty".to_string(),
        ));
    }

    let model = resolve_embed_model(
        body.model.as_deref(),
        embedding_runtime.default_model.as_deref(),
    )?;

    let vector = embedding_runtime
        .client
        .embed_with_model(text, Some(model.as_str()))
        .await
        .ok_or((
            StatusCode::SERVICE_UNAVAILABLE,
            "embedding backend unavailable".to_string(),
        ))?;

    Ok(Json(EmbedResponse { vector }))
}

pub(super) async fn handle_openai_embeddings(
    Extension(embedding_runtime): Extension<Arc<GatewayEmbeddingRuntime>>,
    Json(body): Json<OpenAiEmbeddingsRequest>,
) -> GatewayJsonResult<OpenAiEmbeddingsResponse> {
    let texts = parse_openai_embedding_input(&body.input)?;
    let model = resolve_embed_model(
        body.model.as_deref(),
        embedding_runtime.default_model.as_deref(),
    )?;

    let vectors = embedding_runtime
        .client
        .embed_batch_with_model(&texts, Some(model.as_str()))
        .await
        .ok_or((
            StatusCode::SERVICE_UNAVAILABLE,
            "embedding backend unavailable".to_string(),
        ))?;

    if vectors.len() != texts.len() {
        return Err((
            StatusCode::BAD_GATEWAY,
            "embedding backend returned invalid vector count".to_string(),
        ));
    }

    let data = vectors
        .into_iter()
        .enumerate()
        .map(|(index, embedding)| OpenAiEmbeddingData {
            object: "embedding",
            index,
            embedding,
        })
        .collect::<Vec<_>>();

    Ok(Json(OpenAiEmbeddingsResponse {
        object: "list",
        model,
        data,
        usage: OpenAiEmbeddingUsage {
            prompt_tokens: 0,
            total_tokens: 0,
        },
    }))
}

pub(super) async fn handle_health(
    State(state): State<GatewayState>,
) -> Json<GatewayHealthResponse> {
    let tool_list_cache = state.agent.inspect_tool_list_cache_stats();
    let in_flight_turns = state.max_concurrent_turns.and_then(|max| {
        state
            .concurrency_semaphore
            .as_ref()
            .map(|sem| max.saturating_sub(sem.available_permits()))
    });

    Json(GatewayHealthResponse {
        status: "healthy",
        turn_timeout_secs: state.turn_timeout_secs,
        max_concurrent_turns: state.max_concurrent_turns,
        in_flight_turns,
        tools: GatewayExternalToolHealthResponse {
            enabled: tool_list_cache.is_some(),
            tools_list_cache: tool_list_cache,
        },
    })
}
