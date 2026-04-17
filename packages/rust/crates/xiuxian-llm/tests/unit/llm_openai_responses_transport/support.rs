pub(super) use anyhow::{Result, anyhow};
pub(super) use axum::Router;
pub(super) use axum::body::Bytes;
pub(super) use axum::extract::State;
pub(super) use axum::http::{StatusCode, header::CONTENT_TYPE};
pub(super) use axum::response::IntoResponse;
pub(super) use axum::routing::post;
pub(super) use litellm_rs::core::types::chat::{ChatMessage, ChatRequest as LiteChatRequest};
pub(super) use litellm_rs::core::types::message::{MessageContent, MessageRole};
pub(super) use litellm_rs::core::types::tools::{FunctionDefinition, Tool, ToolType};
pub(super) use reqwest::Client;
pub(super) use serde_json::{Value, json};
pub(super) use std::sync::Arc;
pub(super) use std::sync::Mutex;
pub(super) use std::sync::atomic::{AtomicUsize, Ordering};
pub(super) use std::time::Duration;
pub(super) use tokio::net::TcpListener;
pub(super) use xiuxian_llm::llm::providers::{
    execute_openai_responses_request, is_openai_like_stream_required_error_message,
};

#[derive(Clone)]
pub(super) struct MockResponse {
    pub(super) status: StatusCode,
    pub(super) content_type: &'static str,
    pub(super) body: &'static str,
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

pub(super) async fn spawn_mock_responses_server(state: MockResponse) -> Result<String> {
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

pub(super) async fn spawn_mock_responses_sequence_server(
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

pub(super) async fn spawn_mock_delayed_responses_server(
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

pub(super) async fn spawn_mock_captured_responses_server(
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

pub(super) fn request_with_tool_alias() -> LiteChatRequest {
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
                parameters: Some(json!({
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
