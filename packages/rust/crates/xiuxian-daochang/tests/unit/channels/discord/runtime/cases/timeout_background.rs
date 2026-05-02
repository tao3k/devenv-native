use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};
use axum::{Json, Router, extract::State, routing::post};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use xiuxian_daochang::{Channel, ForegroundQueueMode};

use super::support::{
    MockChannel, build_agent_with_inference_url, build_discord_foreground_runtime, inbound,
};

#[derive(Clone)]
struct SlowLlmState {
    delay: Duration,
    requests: Arc<Mutex<usize>>,
}

async fn handle_slow_chat_completion(
    State(state): State<SlowLlmState>,
    Json(_payload): Json<Value>,
) -> Json<Value> {
    {
        let mut requests = state.requests.lock().await;
        *requests += 1;
    }
    tokio::time::sleep(state.delay).await;
    Json(json!({
        "id": "mock-chatcmpl-timeout",
        "object": "chat.completion",
        "created": 0,
        "model": "test-model",
        "choices": [{
            "index": 0,
            "finish_reason": "stop",
            "message": {
                "role": "assistant",
                "content": "background completed"
            }
        }]
    }))
}

async fn spawn_slow_llm_server(
    delay: Duration,
) -> Result<(String, Arc<Mutex<usize>>, JoinHandle<()>)> {
    let requests = Arc::new(Mutex::new(0usize));
    let state = SlowLlmState {
        delay,
        requests: Arc::clone(&requests),
    };
    let app = Router::new()
        .route("/v1/chat/completions", post(handle_slow_chat_completion))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    tokio::time::sleep(Duration::from_millis(40)).await;
    Ok((
        format!("http://{addr}/v1/chat/completions"),
        requests,
        handle,
    ))
}

fn extract_job_id(reply: &str) -> Result<String> {
    let mut parts = reply.split('`');
    let _prefix = parts.next();
    let Some(job_id) = parts.next() else {
        return Err(anyhow!("reply does not contain a job id: {reply}"));
    };
    if job_id.trim().is_empty() {
        return Err(anyhow!("reply contains an empty job id: {reply}"));
    }
    Ok(job_id.to_string())
}

#[tokio::test]
async fn discord_foreground_timeout_requeues_as_background_job() -> Result<()> {
    let (inference_url, _request_count, llm_handle) =
        spawn_slow_llm_server(Duration::from_millis(1_500)).await?;
    let agent = build_agent_with_inference_url(&inference_url).await?;
    let channel = Arc::new(MockChannel::with_acl(true, std::iter::empty::<&str>()));
    let channel_dyn: Arc<dyn Channel> = channel.clone();
    let mut runtime =
        build_discord_foreground_runtime(agent, channel_dyn, 1, 1, ForegroundQueueMode::Queue);
    runtime
        .spawn_foreground_turn(inbound(
            "Search Daochang knowledge and summarize the memory gate.",
        ))
        .await;
    timeout(Duration::from_secs(5), async {
        while runtime.has_foreground_tasks() {
            runtime.join_next_foreground_task().await;
        }
    })
    .await?;

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 1);
    assert!(
        sent[0].0.contains("Moved it to background job `job-"),
        "expected timeout reply to enqueue a background job, got: {}",
        sent[0].0
    );
    assert!(
        sent[0]
            .0
            .contains("will post the result here when it's ready"),
        "expected timeout reply to mention background completion delivery, got: {}",
        sent[0].0
    );

    let job_id = extract_job_id(&sent[0].0)?;
    let completion_pushed = timeout(
        Duration::from_secs(5),
        runtime.push_next_background_completion(),
    )
    .await?;
    assert!(
        completion_pushed,
        "expected one background completion to be emitted"
    );

    let sent = channel.sent_messages().await;
    assert_eq!(sent.len(), 2);
    assert!(
        sent[1]
            .0
            .contains(format!("Finished background job `{job_id}`.").as_str()),
        "expected background completion ack for {job_id}, got: {}",
        sent[1].0
    );
    assert!(
        sent[1].0.contains("background completed"),
        "expected completion payload in second discord message, got: {}",
        sent[1].0
    );

    llm_handle.abort();
    Ok(())
}
