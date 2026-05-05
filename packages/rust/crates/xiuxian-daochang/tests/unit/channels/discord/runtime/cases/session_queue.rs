use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::Result;
use axum::{Json, Router, extract::State, routing::post};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tokio::time::timeout;
use xiuxian_daochang::{Channel, ForegroundQueueMode};

use super::support::{
    MockChannel, build_agent_with_inference_url, build_discord_foreground_runtime,
    inbound_for_session,
};

#[derive(Clone, Default)]
struct MockLlmState {
    requests: Arc<Mutex<Vec<String>>>,
    in_flight: Arc<AtomicUsize>,
    peak_in_flight: Arc<AtomicUsize>,
}

async fn handle_chat_completion(
    State(state): State<MockLlmState>,
    Json(payload): Json<Value>,
) -> Json<Value> {
    let user_message = extract_latest_user_message(&payload);
    state.requests.lock().await.push(user_message.clone());
    let current_in_flight = state.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
    state
        .peak_in_flight
        .fetch_max(current_in_flight, Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(150)).await;
    state.in_flight.fetch_sub(1, Ordering::SeqCst);
    Json(json!({
        "id": "mock-chatcmpl-session-queue",
        "object": "chat.completion",
        "created": 0,
        "model": "test-model",
        "choices": [{
            "index": 0,
            "finish_reason": "stop",
            "message": {
                "role": "assistant",
                "content": format!("reply:{user_message}")
            }
        }]
    }))
}

async fn spawn_mock_llm_server() -> Result<(String, MockLlmState, tokio::task::JoinHandle<()>)> {
    let state = MockLlmState::default();
    let app = Router::new()
        .route("/v1/chat/completions", post(handle_chat_completion))
        .with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    tokio::time::sleep(Duration::from_millis(40)).await;
    Ok((format!("http://{addr}/v1/chat/completions"), state, handle))
}

fn extract_latest_user_message(payload: &Value) -> String {
    payload
        .get("messages")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .rev()
        .find_map(|message| {
            let role = message.get("role").and_then(Value::as_str)?;
            if role != "user" {
                return None;
            }
            match message.get("content") {
                Some(Value::String(text)) => Some(text.clone()),
                Some(Value::Array(parts)) => parts.iter().find_map(|part| {
                    if part.get("type").and_then(Value::as_str) == Some("text") {
                        part.get("text")
                            .and_then(Value::as_str)
                            .map(ToString::to_string)
                    } else {
                        None
                    }
                }),
                _ => None,
            }
        })
        .unwrap_or_else(|| "<missing-user-message>".to_string())
}

#[tokio::test]
async fn discord_foreground_queue_serializes_same_session_turns() -> Result<()> {
    let (inference_url, llm_state, llm_handle) = spawn_mock_llm_server().await?;
    let agent = build_agent_with_inference_url(&inference_url).await?;
    let channel = Arc::new(MockChannel::with_acl(true, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let mut runtime =
        build_discord_foreground_runtime(agent, channel_dyn, 5, 2, ForegroundQueueMode::Queue);
    let session_key = "guild-1:channel-1:user-1";

    runtime
        .spawn_foreground_turn(inbound_for_session("first queued message", session_key))
        .await;
    runtime
        .spawn_foreground_turn(inbound_for_session("second queued message", session_key))
        .await;

    timeout(Duration::from_secs(5), async {
        while runtime.has_foreground_tasks() {
            runtime.join_next_foreground_task().await;
        }
    })
    .await?;

    let sent = channel.sent_messages().await;
    assert_eq!(
        sent.iter()
            .map(|(message, _)| message.clone())
            .collect::<Vec<_>>(),
        vec![
            "reply:first queued message".to_string(),
            "reply:second queued message".to_string(),
        ]
    );
    assert_eq!(
        llm_state.peak_in_flight.load(Ordering::SeqCst),
        1,
        "same-session queue mode must not run foreground turns concurrently"
    );
    assert_eq!(
        llm_state.requests.lock().await.clone(),
        vec![
            "first queued message".to_string(),
            "second queued message".to_string(),
        ]
    );

    llm_handle.abort();
    Ok(())
}
