#![allow(
    missing_docs,
    unused_imports,
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::map_unwrap_or,
    clippy::option_as_ref_deref,
    clippy::unreadable_literal,
    clippy::useless_conversion,
    clippy::match_wildcard_for_single_variants,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_raw_string_hashes,
    clippy::manual_async_fn,
    clippy::manual_let_else,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::single_match_else,
    clippy::similar_names,
    clippy::format_collect,
    clippy::assigning_clones
)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use xiuxian_daochang::{Agent, AgentConfig};

#[derive(Clone)]
struct LlmRecoveryState {
    calls: Arc<AtomicUsize>,
}

async fn handle_context_overflow_then_success(
    State(state): State<LlmRecoveryState>,
    Json(_payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let call_index = state.calls.fetch_add(1, Ordering::SeqCst);
    if call_index == 0 {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "message": "litellm.APIConnectionError: MinimaxException - {\"type\":\"error\",\"error\":{\"type\":\"bad_request_error\",\"message\":\"invalid params, context window exceeds limit (2013)\",\"http_code\":\"400\"},\"request_id\":\"req-1\"}",
                    "type": "server_error"
                }
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "id": "mock-chatcmpl-context-repair",
            "object": "chat.completion",
            "created": 0,
            "model": "test-model",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "context repaired",
                    "tool_calls": null
                },
                "finish_reason": "stop"
            }]
        })),
    )
}

async fn handle_non_context_error(
    State(state): State<LlmRecoveryState>,
    Json(_payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let _ = state.calls.fetch_add(1, Ordering::SeqCst);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error": {
                "message": "provider temporary failure",
                "type": "server_error"
            }
        })),
    )
}

async fn spawn_context_overflow_then_success_server()
-> Result<Option<(String, Arc<AtomicUsize>, tokio::task::JoinHandle<()>)>> {
    let calls = Arc::new(AtomicUsize::new(0));
    let state = LlmRecoveryState {
        calls: Arc::clone(&calls),
    };
    let app = Router::new()
        .route(
            "/v1/chat/completions",
            post(handle_context_overflow_then_success),
        )
        .with_state(state);
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping context window recovery tests: local socket bind is not permitted");
            return Ok(None);
        }
        Err(err) => return Err(err.into()),
    };
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Ok(Some((
        format!("http://{addr}/v1/chat/completions"),
        calls,
        handle,
    )))
}

async fn spawn_non_context_error_server()
-> Result<Option<(String, Arc<AtomicUsize>, tokio::task::JoinHandle<()>)>> {
    let calls = Arc::new(AtomicUsize::new(0));
    let state = LlmRecoveryState {
        calls: Arc::clone(&calls),
    };
    let app = Router::new()
        .route("/v1/chat/completions", post(handle_non_context_error))
        .with_state(state);
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping context window recovery tests: local socket bind is not permitted");
            return Ok(None);
        }
        Err(err) => return Err(err.into()),
    };
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Ok(Some((
        format!("http://{addr}/v1/chat/completions"),
        calls,
        handle,
    )))
}

fn base_agent_config(inference_url: String) -> AgentConfig {
    AgentConfig {
        inference_url,
        model: "test-model".to_string(),
        api_key: None,
        max_tool_rounds: 1,
        context_budget_tokens: Some(4096),
        context_budget_reserve_tokens: 256,
        ..AgentConfig::default()
    }
}

#[tokio::test]
async fn run_turn_auto_recovers_from_context_window_error() -> Result<()> {
    let Some((inference_url, calls, server)) = spawn_context_overflow_then_success_server().await?
    else {
        return Ok(());
    };

    let agent = Agent::from_config(base_agent_config(inference_url)).await?;
    let out = agent.run_turn("ctx-repair", "hello").await?;
    assert_eq!(out, "context repaired");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "agent should retry once after context-window overflow"
    );

    server.abort();
    let _ = server.await;
    Ok(())
}

#[tokio::test]
async fn run_turn_does_not_retry_for_non_context_error() -> Result<()> {
    let Some((inference_url, calls, server)) = spawn_non_context_error_server().await? else {
        return Ok(());
    };

    let agent = Agent::from_config(base_agent_config(inference_url)).await?;
    let result = agent.run_turn("ctx-no-repair", "hello").await;
    assert!(result.is_err());
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "non-context server error should not trigger retry loop"
    );

    server.abort();
    let _ = server.await;
    Ok(())
}
