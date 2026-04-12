//! Vercel AI SDK UI Message Stream endpoint (`POST /vercel/stream`).
//!
//! Accepts AI SDK v6 `UIMessage[]`, drives `Agent::run_turn_stream`, and
//! emits SSE events in the v1 Data Stream Protocol expected by `useChat`'s
//! `DefaultChatTransport`.
//!
//! This is a protocol-specific adapter -- the Agent itself is
//! protocol-agnostic. Wire-format encoding lives entirely in this module.

use std::convert::Infallible;
use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::http::HeaderValue;
use axum::response::IntoResponse;
use axum::response::sse::{Event, KeepAlive, Sse};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::agent::AgentStreamEvent;

use super::types::GatewayState;

/// Request body: AI SDK v6 UIMessage array.
#[derive(Debug, serde::Deserialize)]
pub(super) struct VercelStreamRequest {
    /// Full conversation history from the client.
    messages: Vec<UiMessage>,
    /// Session identifier (optional; defaults to a generated UUID).
    #[serde(default)]
    session_id: Option<String>,
}

/// One AI SDK UIMessage.
#[derive(Debug, serde::Deserialize)]
struct UiMessage {
    #[allow(dead_code)]
    id: String,
    role: String,
    parts: Vec<UiMessagePart>,
}

/// A single part within a UIMessage.
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
enum UiMessagePart {
    Text { text: String },
    #[serde(other)]
    Other,
}

/// Extract the last user message text from the UIMessage array.
fn extract_last_user_message(messages: &[UiMessage]) -> Option<String> {
    messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .and_then(|m| {
            let text: String = m
                .parts
                .iter()
                .filter_map(|p| match p {
                    UiMessagePart::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            if text.is_empty() { None } else { Some(text) }
        })
}

/// `POST /vercel/stream` -- Vercel AI SDK streaming chat endpoint.
pub(super) async fn handle_vercel_stream(
    State(state): State<GatewayState>,
    Json(body): Json<VercelStreamRequest>,
) -> impl IntoResponse {
    let session_id = body
        .session_id
        .unwrap_or_else(|| format!("web_{}", uuid::Uuid::new_v4().simple()));

    let user_message = extract_last_user_message(&body.messages)
        .unwrap_or_default();

    let message_id = format!("msg_{}", uuid::Uuid::new_v4().simple());

    // Channel for SSE events
    let (sse_tx, sse_rx) = mpsc::channel::<Result<Event, Infallible>>(64);
    // Channel for Agent stream events
    let (agent_tx, mut agent_rx) = mpsc::channel::<AgentStreamEvent>(64);

    let agent = state.agent.clone();

    // Spawn the Agent turn
    tokio::spawn(async move {
        let _ = agent
            .run_turn_stream(&session_id, &user_message, &agent_tx)
            .await;
        // Channel drops naturally when agent_tx goes out of scope.
    });

    // Spawn the SSE encoder: reads AgentStreamEvents and writes SSE
    let msg_id = message_id.clone();
    tokio::spawn(async move {
        encode_agent_events_to_sse(msg_id, &mut agent_rx, &sse_tx).await;
    });

    let stream = ReceiverStream::new(sse_rx);
    let sse = Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text(""),
    );

    let mut response = sse.into_response();
    response.headers_mut().insert(
        "x-vercel-ai-ui-message-stream",
        HeaderValue::from_static("v1"),
    );
    response
}

/// Encode `AgentStreamEvent`s into AI SDK v1 Data Stream Protocol SSE events.
async fn encode_agent_events_to_sse(
    message_id: String,
    agent_rx: &mut mpsc::Receiver<AgentStreamEvent>,
    sse_tx: &mpsc::Sender<Result<Event, Infallible>>,
) {
    let mut text_id: Option<String> = None;
    let mut step_open = false;

    // Start message
    let _ = send_json(sse_tx, &serde_json::json!({"type": "start", "messageId": message_id})).await;

    while let Some(event) = agent_rx.recv().await {
        match event {
            AgentStreamEvent::TextDelta(delta) => {
                if !step_open {
                    let _ = send_json(sse_tx, &serde_json::json!({"type": "start-step"})).await;
                    step_open = true;
                }
                // Lazily open text block
                if text_id.is_none() {
                    let id = format!("text_{}", uuid::Uuid::new_v4().simple());
                    let _ = send_json(sse_tx, &serde_json::json!({"type": "text-start", "id": id})).await;
                    text_id = Some(id);
                }
                if let Some(ref id) = text_id {
                    let _ = send_json(sse_tx, &serde_json::json!({"type": "text-delta", "id": id, "delta": delta})).await;
                }
            }

            AgentStreamEvent::ToolCallsStarted { tool_calls } => {
                // Close any open text block before tool calls
                if let Some(id) = text_id.take() {
                    let _ = send_json(sse_tx, &serde_json::json!({"type": "text-end", "id": id})).await;
                }
                if !step_open {
                    let _ = send_json(sse_tx, &serde_json::json!({"type": "start-step"})).await;
                    step_open = true;
                }
                for tc in &tool_calls {
                    let input: serde_json::Value =
                        serde_json::from_str(&tc.function.arguments).unwrap_or_default();
                    let _ = send_json(sse_tx, &serde_json::json!({
                        "type": "tool-input-start",
                        "toolCallId": tc.id,
                        "toolName": tc.function.name,
                    })).await;
                    let _ = send_json(sse_tx, &serde_json::json!({
                        "type": "tool-input-available",
                        "toolCallId": tc.id,
                        "toolName": tc.function.name,
                        "input": input,
                    })).await;
                }
            }

            AgentStreamEvent::ToolResult { tool_call_id, output, .. } => {
                let output_json: serde_json::Value =
                    serde_json::from_str(&output).unwrap_or(serde_json::Value::String(output));
                let _ = send_json(sse_tx, &serde_json::json!({
                    "type": "tool-output-available",
                    "toolCallId": tool_call_id,
                    "output": output_json,
                })).await;
                // Close step after last tool result, next loop iteration
                // opens a new step for the next LLM call
                let _ = send_json(sse_tx, &serde_json::json!({"type": "finish-step"})).await;
                step_open = false;
            }

            AgentStreamEvent::TurnComplete { .. } => {
                // Close any open text block
                if let Some(id) = text_id.take() {
                    let _ = send_json(sse_tx, &serde_json::json!({"type": "text-end", "id": id})).await;
                }
                if step_open {
                    let _ = send_json(sse_tx, &serde_json::json!({"type": "finish-step"})).await;
                }
                let _ = send_json(sse_tx, &serde_json::json!({"type": "finish"})).await;
                let _ = sse_tx.send(Ok(Event::default().data("[DONE]"))).await;
                return;
            }

            AgentStreamEvent::TurnError { error } => {
                if let Some(id) = text_id.take() {
                    let _ = send_json(sse_tx, &serde_json::json!({"type": "text-end", "id": id})).await;
                }
                if step_open {
                    let _ = send_json(sse_tx, &serde_json::json!({"type": "finish-step"})).await;
                }
                let _ = send_json(sse_tx, &serde_json::json!({"type": "error", "errorText": error})).await;
                let _ = send_json(sse_tx, &serde_json::json!({"type": "finish"})).await;
                let _ = sse_tx.send(Ok(Event::default().data("[DONE]"))).await;
                return;
            }
        }
    }

    // Channel closed without TurnComplete/TurnError -- clean up
    if let Some(id) = text_id.take() {
        let _ = send_json(sse_tx, &serde_json::json!({"type": "text-end", "id": id})).await;
    }
    if step_open {
        let _ = send_json(sse_tx, &serde_json::json!({"type": "finish-step"})).await;
    }
    let _ = send_json(sse_tx, &serde_json::json!({"type": "finish"})).await;
    let _ = sse_tx.send(Ok(Event::default().data("[DONE]"))).await;
}

/// Send a JSON-encoded SSE event.
async fn send_json(
    tx: &mpsc::Sender<Result<Event, Infallible>>,
    value: &serde_json::Value,
) -> Result<(), mpsc::error::SendError<Result<Event, Infallible>>> {
    let data = serde_json::to_string(value).unwrap_or_default();
    tx.send(Ok(Event::default().data(data))).await
}
