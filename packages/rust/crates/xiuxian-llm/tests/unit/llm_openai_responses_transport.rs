#![cfg(feature = "provider-litellm")]

//! Integration tests for `OpenAI` `/responses` transport execution.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::{Result, anyhow};
use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{StatusCode, header::CONTENT_TYPE};
use axum::response::IntoResponse;
use axum::routing::post;
use litellm_rs::core::types::chat::{ChatMessage, ChatRequest as LiteChatRequest};
use litellm_rs::core::types::message::{MessageContent, MessageRole};
use litellm_rs::core::types::tools::{FunctionDefinition, Tool, ToolType};
use serde_json::Value;
use tokio::net::TcpListener;
use xiuxian_llm::llm::providers::{
    execute_openai_responses_request, is_openai_like_stream_required_error_message,
};

#[derive(Clone)]
struct MockResponse {
    status: StatusCode,
    content_type: &'static str,
    body: &'static str,
}

#[derive(Clone)]
struct MockSequenceState {
    responses: Arc<Vec<MockResponse>>,
    requests_seen: Arc<AtomicUsize>,
}

#[derive(Clone)]
struct DelayedResponseState {
    response: MockResponse,
    header_delay: Duration,
}

#[derive(Clone)]
struct CapturedResponseState {
    response: MockResponse,
    requests: Arc<Mutex<Vec<Value>>>,
}

async fn responses(State(state): State<MockResponse>) -> impl IntoResponse {
    (
        state.status,
        [(CONTENT_TYPE, state.content_type)],
        state.body.to_string(),
    )
}

async fn responses_sequence(State(state): State<MockSequenceState>) -> impl IntoResponse {
    let index = state.requests_seen.fetch_add(1, Ordering::SeqCst);
    let selected = state
        .responses
        .get(index)
        .cloned()
        .or_else(|| state.responses.last().cloned())
        .unwrap_or(MockResponse {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            content_type: "text/plain",
            body: "missing mock response",
        });
    (
        selected.status,
        [(CONTENT_TYPE, selected.content_type)],
        selected.body.to_string(),
    )
}

async fn delayed_responses(State(state): State<DelayedResponseState>) -> impl IntoResponse {
    tokio::time::sleep(state.header_delay).await;
    (
        state.response.status,
        [(CONTENT_TYPE, state.response.content_type)],
        state.response.body.to_string(),
    )
}

async fn responses_with_capture(
    State(state): State<CapturedResponseState>,
    body: Bytes,
) -> impl IntoResponse {
    let payload = serde_json::from_slice::<Value>(&body)
        .unwrap_or_else(|error| Value::String(format!("invalid_json:{error}")));
    let mut requests = match state.requests.lock() {
        Ok(requests) => requests,
        Err(error) => panic!("capture lock should not be poisoned: {error}"),
    };
    requests.push(payload);
    (
        state.response.status,
        [(CONTENT_TYPE, state.response.content_type)],
        state.response.body.to_string(),
    )
}

async fn spawn_mock_responses_server(state: MockResponse) -> Result<String> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let app = Router::new()
        .route("/v1/responses", post(responses))
        .with_state(state);
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Ok(format!("http://{addr}/v1/responses"))
}

async fn spawn_mock_responses_sequence_server(
    responses: Vec<MockResponse>,
) -> Result<(String, Arc<AtomicUsize>)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let requests_seen = Arc::new(AtomicUsize::new(0));
    let state = MockSequenceState {
        responses: Arc::new(responses),
        requests_seen: Arc::clone(&requests_seen),
    };
    let app = Router::new()
        .route("/v1/responses", post(responses_sequence))
        .with_state(state);
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Ok((format!("http://{addr}/v1/responses"), requests_seen))
}

async fn spawn_mock_delayed_responses_server(
    response: MockResponse,
    header_delay: Duration,
) -> Result<String> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let app = Router::new()
        .route("/v1/responses", post(delayed_responses))
        .with_state(DelayedResponseState {
            response,
            header_delay,
        });
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Ok(format!("http://{addr}/v1/responses"))
}

async fn spawn_mock_captured_responses_server(
    response: MockResponse,
) -> Result<(String, Arc<Mutex<Vec<Value>>>)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let requests = Arc::new(Mutex::new(Vec::new()));
    let app = Router::new()
        .route("/v1/responses", post(responses_with_capture))
        .with_state(CapturedResponseState {
            response,
            requests: Arc::clone(&requests),
        });
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    Ok((format!("http://{addr}/v1/responses"), requests))
}

fn request_with_tool_alias() -> LiteChatRequest {
    LiteChatRequest {
        model: "gpt-5-codex".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("hello".to_string())),
            ..Default::default()
        }],
        tools: Some(vec![Tool {
            tool_type: ToolType::Function,
            function: FunctionDefinition {
                name: "qianhuan.reload".to_string(),
                description: Some("Reload qianhuan runtime".to_string()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "scope": { "type": "string" }
                    }
                })),
            },
        }]),
        ..Default::default()
    }
}

#[tokio::test]
async fn execute_openai_responses_request_parses_text_and_tool_calls() -> Result<()> {
    let endpoint = spawn_mock_responses_server(MockResponse {
        status: StatusCode::OK,
        content_type: "text/event-stream",
        body: r#"data: {"type":"response.output_item.done","item":{"type":"message","content":[{"type":"output_text","text":"pong"}]}}
data: {"type":"response.output_item.done","item":{"type":"function_call","id":"call_1","call_id":"call_1","name":"qianhuan_reload","arguments":"{\"scope\":\"all\"}"}}
data: [DONE]"#,
    })
    .await?;

    let parsed = execute_openai_responses_request(
        &reqwest::Client::new(),
        &endpoint,
        Some("test-key"),
        &request_with_tool_alias(),
    )
    .await?;

    assert_eq!(parsed.content.as_deref(), Some("pong"));
    assert_eq!(parsed.tool_calls.len(), 1);
    assert_eq!(parsed.tool_calls[0].function.name, "qianhuan.reload");
    assert_eq!(
        parsed.tool_calls[0].function.arguments,
        r#"{"scope":"all"}"#
    );
    Ok(())
}

#[tokio::test]
async fn execute_openai_responses_request_sends_developer_role_in_payload() -> Result<()> {
    let (endpoint, requests) = spawn_mock_captured_responses_server(MockResponse {
        status: StatusCode::OK,
        content_type: "text/event-stream",
        body: r#"data: {"type":"response.output_item.done","item":{"type":"message","content":[{"type":"output_text","text":"pong"}]}}
data: [DONE]"#,
    })
    .await?;

    let request = LiteChatRequest {
        model: "gpt-5-codex".to_string(),
        messages: vec![
            ChatMessage {
                role: MessageRole::Developer,
                content: Some(MessageContent::Text(
                    "Prefer terse implementation-first answers.".to_string(),
                )),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("hello".to_string())),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let parsed = execute_openai_responses_request(
        &reqwest::Client::new(),
        &endpoint,
        Some("test-key"),
        &request,
    )
    .await?;

    assert_eq!(parsed.content.as_deref(), Some("pong"));
    let captured = match requests.lock() {
        Ok(captured) => captured.clone(),
        Err(error) => panic!("capture lock should not be poisoned: {error}"),
    };
    let payload = captured
        .first()
        .ok_or_else(|| anyhow!("expected a captured request payload"))?;
    let input = payload["input"]
        .as_array()
        .ok_or_else(|| anyhow!("captured payload should include input array: {payload}"))?;

    assert_eq!(input[0]["role"], serde_json::json!("developer"));
    assert_eq!(
        input[0]["content"],
        serde_json::json!("Prefer terse implementation-first answers.")
    );
    assert_eq!(input[1]["role"], serde_json::json!("user"));
    Ok(())
}

#[tokio::test]
async fn execute_openai_responses_request_retries_transient_503_and_succeeds() -> Result<()> {
    let (endpoint, requests_seen) = spawn_mock_responses_sequence_server(vec![
        MockResponse {
            status: StatusCode::SERVICE_UNAVAILABLE,
            content_type: "text/event-stream",
            body: "upstream connect error or disconnect/reset before headers. reset reason: connection termination",
        },
        MockResponse {
            status: StatusCode::OK,
            content_type: "text/event-stream",
            body: r#"data: {"type":"response.output_item.done","item":{"type":"message","content":[{"type":"output_text","text":"pong"}]}}
data: [DONE]"#,
        },
    ])
    .await?;

    let parsed = execute_openai_responses_request(
        &reqwest::Client::new(),
        &endpoint,
        Some("test-key"),
        &request_with_tool_alias(),
    )
    .await?;

    assert_eq!(parsed.content.as_deref(), Some("pong"));
    assert_eq!(requests_seen.load(Ordering::SeqCst), 2);
    Ok(())
}

#[tokio::test]
async fn execute_openai_responses_request_surfaces_http_error_status() -> Result<()> {
    let endpoint = spawn_mock_responses_server(MockResponse {
        status: StatusCode::BAD_REQUEST,
        content_type: "application/json",
        body: r#"{"error":{"message":"invalid request"}}"#,
    })
    .await?;

    let err = execute_openai_responses_request(
        &reqwest::Client::new(),
        &endpoint,
        Some("test-key"),
        &request_with_tool_alias(),
    )
    .await
    .err()
    .ok_or_else(|| anyhow!("400 status should fail"))?;
    let rendered = err.to_string();
    if !rendered.contains("status 400") {
        return Err(anyhow!("unexpected error message: {rendered}"));
    }
    Ok(())
}

#[tokio::test(start_paused = true)]
async fn execute_openai_responses_request_fails_fast_when_headers_stall() -> Result<()> {
    let endpoint = spawn_mock_delayed_responses_server(
        MockResponse {
            status: StatusCode::OK,
            content_type: "text/event-stream",
            body: "data: [DONE]",
        },
        Duration::from_secs(60),
    )
    .await?;

    let request = request_with_tool_alias();
    let task = tokio::spawn(async move {
        execute_openai_responses_request(
            &reqwest::Client::new(),
            &endpoint,
            Some("test-key"),
            &request,
        )
        .await
    });

    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(31)).await;

    let result = task.await?;
    let err = result
        .err()
        .ok_or_else(|| anyhow!("stalled headers should fail"))?;
    let rendered = err.to_string();
    if !rendered.contains("response headers were not received within 10s") {
        return Err(anyhow!("unexpected timeout error message: {rendered}"));
    }
    Ok(())
}

#[test]
fn stream_required_detector_matches_expected_error_shape() {
    assert!(is_openai_like_stream_required_error_message(
        r#"API error for openai_like (status 400): {"detail":"Stream must be set to true"}"#,
    ));
    assert!(!is_openai_like_stream_required_error_message(
        r#"API error for openai_like (status 400): {"detail":"invalid request"}"#,
    ));
}

#[tokio::test]
async fn execute_openai_responses_request_rejects_duplicate_tool_outputs_before_send() -> Result<()>
{
    let (endpoint, requests_seen) = spawn_mock_responses_sequence_server(vec![MockResponse {
        status: StatusCode::OK,
        content_type: "text/event-stream",
        body: r#"data: {"type":"response.output_item.done","item":{"type":"message","content":[{"type":"output_text","text":"pong"}]}}
data: [DONE]"#,
    }])
    .await?;

    let request = LiteChatRequest {
        model: "gpt-5-codex".to_string(),
        messages: vec![
            ChatMessage {
                role: MessageRole::Assistant,
                tool_calls: Some(vec![litellm_rs::core::types::tools::ToolCall {
                    id: "call_dup".to_string(),
                    tool_type: "function".to_string(),
                    function: litellm_rs::core::types::tools::FunctionCall {
                        name: "qianhuan.reload".to_string(),
                        arguments: r#"{"scope":"all"}"#.to_string(),
                    },
                }]),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::Tool,
                content: Some(MessageContent::Text("first tool output".to_string())),
                tool_call_id: Some("call_dup".to_string()),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::Tool,
                content: Some(MessageContent::Text("duplicate tool output".to_string())),
                tool_call_id: Some("call_dup".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let err = execute_openai_responses_request(
        &reqwest::Client::new(),
        &endpoint,
        Some("test-key"),
        &request,
    )
    .await
    .err()
    .ok_or_else(|| anyhow!("duplicate tool output should fail locally"))?;

    let rendered = err.to_string();
    if !rendered.contains("without an available preceding function_call") {
        return Err(anyhow!("unexpected error message: {rendered}"));
    }
    assert_eq!(requests_seen.load(Ordering::SeqCst), 0);
    Ok(())
}

#[tokio::test]
async fn execute_openai_responses_request_rejects_orphan_tool_outputs_before_send() -> Result<()> {
    let (endpoint, requests_seen) = spawn_mock_responses_sequence_server(vec![MockResponse {
        status: StatusCode::OK,
        content_type: "text/event-stream",
        body: r#"data: {"type":"response.output_item.done","item":{"type":"message","content":[{"type":"output_text","text":"pong"}]}}
data: [DONE]"#,
    }])
    .await?;

    let request = LiteChatRequest {
        model: "gpt-5-codex".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::Tool,
            content: Some(MessageContent::Text("orphan tool output".to_string())),
            tool_call_id: Some("call_orphan".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let err = execute_openai_responses_request(
        &reqwest::Client::new(),
        &endpoint,
        Some("test-key"),
        &request,
    )
    .await
    .err()
    .ok_or_else(|| anyhow!("orphan tool output should fail locally"))?;

    let rendered = err.to_string();
    if !rendered.contains("function_call_output items without an available preceding function_call")
    {
        return Err(anyhow!("unexpected error message: {rendered}"));
    }
    assert_eq!(requests_seen.load(Ordering::SeqCst), 0);
    Ok(())
}
