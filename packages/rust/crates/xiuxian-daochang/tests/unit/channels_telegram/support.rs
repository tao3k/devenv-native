use std::sync::Arc;
use std::time::{Duration, Instant};

pub(super) use anyhow::{Result, anyhow};
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use tokio::sync::Mutex;
pub(super) use xiuxian_daochang::{
    Channel, ChannelAttachment, TELEGRAM_MAX_MESSAGE_LENGTH, TelegramChannel,
    TelegramSessionPartition, decorate_chunk_for_telegram, markdown_to_telegram_html,
    markdown_to_telegram_markdown_v2, split_message_for_telegram,
};

pub(super) fn require_some<T>(value: Option<T>, context: &str) -> T {
    match value {
        Some(value) => value,
        None => panic!("{context}"),
    }
}

pub(super) fn group_text_update(chat_id: i64, user_id: i64, text: &str) -> serde_json::Value {
    serde_json::json!({
        "update_id": 10001,
        "message": {
            "message_id": 77,
            "text": text,
            "chat": {"id": chat_id},
            "from": {"id": user_id, "username": "alice"}
        }
    })
}

pub(super) fn group_text_update_with_title(
    chat_id: i64,
    user_id: i64,
    text: &str,
    title: &str,
) -> serde_json::Value {
    serde_json::json!({
        "update_id": 10002,
        "message": {
            "message_id": 78,
            "text": text,
            "chat": {"id": chat_id, "title": title, "type": "group"},
            "from": {"id": user_id, "username": "bob"}
        }
    })
}

pub(super) fn group_text_update_with_thread(
    chat_id: i64,
    user_id: i64,
    text: &str,
    thread_id: i64,
) -> serde_json::Value {
    let mut update = group_text_update(chat_id, user_id, text);
    update["message"]["message_thread_id"] = serde_json::json!(thread_id);
    update
}

pub(super) fn group_photo_update(
    chat_id: i64,
    user_id: i64,
    caption: Option<&str>,
) -> serde_json::Value {
    let mut update = serde_json::json!({
        "update_id": 10010,
        "message": {
            "message_id": 88,
            "photo": [{"file_id": "abc"}],
            "chat": {"id": chat_id},
            "from": {"id": user_id, "username": "alice"}
        }
    });
    if let Some(caption) = caption {
        update["message"]["caption"] = serde_json::Value::String(caption.to_string());
    }
    update
}

#[derive(Clone, Default)]
pub(super) struct MockTelegramState {
    pub(super) requests: Arc<Mutex<Vec<serde_json::Value>>>,
    pub(super) first_markdown_error: Arc<Mutex<Option<String>>>,
}

async fn handle_send_message(
    State(state): State<MockTelegramState>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    state.requests.lock().await.push(payload.clone());
    let parse_mode = payload
        .get("parse_mode")
        .and_then(serde_json::Value::as_str);
    if parse_mode == Some("MarkdownV2") {
        let mut first_markdown_error = state.first_markdown_error.lock().await;
        if let Some(description) = first_markdown_error.take() {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "ok": false,
                    "description": description
                })),
            );
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({"ok": true, "result": {"message_id": 1}})),
    )
}

pub(super) async fn spawn_mock_telegram_api(
    first_markdown_error: Option<&str>,
) -> Result<Option<(String, MockTelegramState, tokio::task::JoinHandle<()>)>> {
    let state = MockTelegramState {
        requests: Arc::new(Mutex::new(Vec::new())),
        first_markdown_error: Arc::new(Mutex::new(
            first_markdown_error.map(std::string::ToString::to_string),
        )),
    };

    let app = Router::new()
        .route("/botfake-token/sendMessage", post(handle_send_message))
        .with_state(state.clone());
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping telegram mock api tests: local socket bind is not permitted");
            return Ok(None);
        }
        Err(err) => return Err(err.into()),
    };
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    wait_for_listener(addr).await;

    Ok(Some((format!("http://{addr}"), state, handle)))
}

#[derive(Clone, Default)]
pub(super) struct MockTelegramApiLevelErrorState {
    pub(super) requests: Arc<Mutex<Vec<serde_json::Value>>>,
    pub(super) first_markdown_error: Arc<Mutex<Option<String>>>,
}

async fn handle_send_message_api_level_error(
    State(state): State<MockTelegramApiLevelErrorState>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    state.requests.lock().await.push(payload.clone());
    let parse_mode = payload
        .get("parse_mode")
        .and_then(serde_json::Value::as_str);

    if parse_mode == Some("MarkdownV2") {
        let mut first_markdown_error = state.first_markdown_error.lock().await;
        if let Some(description) = first_markdown_error.take() {
            return (
                StatusCode::OK,
                Json(serde_json::json!({
                    "ok": false,
                    "error_code": 400,
                    "description": description
                })),
            );
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({"ok": true, "result": {"message_id": 1}})),
    )
}

pub(super) async fn spawn_mock_telegram_api_level_error(
    first_markdown_error: Option<&str>,
) -> Result<
    Option<(
        String,
        MockTelegramApiLevelErrorState,
        tokio::task::JoinHandle<()>,
    )>,
> {
    let state = MockTelegramApiLevelErrorState {
        requests: Arc::new(Mutex::new(Vec::new())),
        first_markdown_error: Arc::new(Mutex::new(
            first_markdown_error.map(std::string::ToString::to_string),
        )),
    };

    let app = Router::new()
        .route(
            "/botfake-token/sendMessage",
            post(handle_send_message_api_level_error),
        )
        .with_state(state.clone());
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping telegram mock api tests: local socket bind is not permitted");
            return Ok(None);
        }
        Err(err) => return Err(err.into()),
    };
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    wait_for_listener(addr).await;

    Ok(Some((format!("http://{addr}"), state, handle)))
}

#[derive(Clone)]
struct DelayedSendState {
    delay: Duration,
}

async fn handle_delayed_send_message(
    State(state): State<DelayedSendState>,
    Json(_payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    tokio::time::sleep(state.delay).await;
    (
        StatusCode::OK,
        Json(serde_json::json!({"ok": true, "result": {"message_id": 1}})),
    )
}

pub(super) async fn spawn_delayed_send_mock_telegram_api(
    delay: Duration,
) -> Result<Option<(String, tokio::task::JoinHandle<()>)>> {
    let app = Router::new()
        .route(
            "/botfake-token/sendMessage",
            post(handle_delayed_send_message),
        )
        .with_state(DelayedSendState { delay });
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping telegram mock api tests: local socket bind is not permitted");
            return Ok(None);
        }
        Err(err) => return Err(err.into()),
    };
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    wait_for_listener(addr).await;

    Ok(Some((format!("http://{addr}"), handle)))
}

#[derive(Clone)]
pub(super) struct RetryThenSuccessState {
    pub(super) requests: Arc<Mutex<Vec<serde_json::Value>>>,
    remaining_failures: Arc<Mutex<usize>>,
}

async fn handle_send_message_retry_then_success(
    State(state): State<RetryThenSuccessState>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    state.requests.lock().await.push(payload);

    let mut remaining = state.remaining_failures.lock().await;
    if *remaining > 0 {
        *remaining -= 1;
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "ok": false,
                "error_code": 429,
                "description": "Too Many Requests: retry later",
                "parameters": {
                    "retry_after": 0
                }
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({"ok": true, "result": {"message_id": 1}})),
    )
}

pub(super) async fn spawn_retry_then_success_mock_telegram_api(
    failures_before_success: usize,
) -> Result<Option<(String, RetryThenSuccessState, tokio::task::JoinHandle<()>)>> {
    let state = RetryThenSuccessState {
        requests: Arc::new(Mutex::new(Vec::new())),
        remaining_failures: Arc::new(Mutex::new(failures_before_success)),
    };
    let app = Router::new()
        .route(
            "/botfake-token/sendMessage",
            post(handle_send_message_retry_then_success),
        )
        .with_state(state.clone());
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping telegram mock api tests: local socket bind is not permitted");
            return Ok(None);
        }
        Err(err) => return Err(err.into()),
    };
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    wait_for_listener(addr).await;

    Ok(Some((format!("http://{addr}"), state, handle)))
}

#[derive(Clone)]
pub(super) struct TimedTelegramRequest {
    pub(super) payload: serde_json::Value,
    pub(super) received_at: Instant,
}

#[derive(Clone)]
pub(super) struct RateLimitGateState {
    pub(super) requests: Arc<Mutex<Vec<TimedTelegramRequest>>>,
    pub(super) first_rate_limit_emitted: Arc<Mutex<bool>>,
}

async fn handle_send_message_rate_limit_once(
    State(state): State<RateLimitGateState>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    state.requests.lock().await.push(TimedTelegramRequest {
        payload: payload.clone(),
        received_at: Instant::now(),
    });

    let text = payload
        .get("text")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let mut emitted = state.first_rate_limit_emitted.lock().await;
    if text == "firstgatecheck" && !*emitted {
        *emitted = true;
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "ok": false,
                "error_code": 429,
                "description": "Too Many Requests: retry later",
                "parameters": {
                    "retry_after": 1
                }
            })),
        );
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({"ok": true, "result": {"message_id": 1}})),
    )
}

pub(super) async fn spawn_rate_limit_gate_mock_telegram_api()
-> Result<Option<(String, RateLimitGateState, tokio::task::JoinHandle<()>)>> {
    let state = RateLimitGateState {
        requests: Arc::new(Mutex::new(Vec::new())),
        first_rate_limit_emitted: Arc::new(Mutex::new(false)),
    };
    let app = Router::new()
        .route(
            "/botfake-token/sendMessage",
            post(handle_send_message_rate_limit_once),
        )
        .with_state(state.clone());
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping telegram mock api tests: local socket bind is not permitted");
            return Ok(None);
        }
        Err(err) => return Err(err.into()),
    };
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    wait_for_listener(addr).await;

    Ok(Some((format!("http://{addr}"), state, handle)))
}

#[derive(Clone, Default)]
pub(super) struct PollingMediaState {
    get_updates_calls: Arc<Mutex<usize>>,
    pub(super) get_file_requests: Arc<Mutex<Vec<serde_json::Value>>>,
}

async fn handle_get_updates_with_photo(
    State(state): State<PollingMediaState>,
    Json(_payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut calls = state.get_updates_calls.lock().await;
    *calls += 1;
    if *calls == 1 {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "ok": true,
                "result": [{
                    "update_id": 50001,
                    "message": {
                        "message_id": 901,
                        "caption": "please analyze this image",
                        "photo": [
                            {"file_id": "photo_file_small", "file_size": 32},
                            {"file_id": "photo_file_large", "file_size": 2048}
                        ],
                        "chat": {"id": -200_123, "type": "group", "title": "vision-lab"},
                        "from": {"id": 888, "username": "alice"}
                    }
                }]
            })),
        );
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({ "ok": true, "result": [] })),
    )
}

async fn handle_get_file_for_photo(
    State(state): State<PollingMediaState>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    state.get_file_requests.lock().await.push(payload.clone());
    let file_id = payload
        .get("file_id")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let file_path = if file_id == "photo_file_large" {
        "photos/vision.jpg"
    } else {
        "photos/fallback.jpg"
    };
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "ok": true,
            "result": {
                "file_path": file_path
            }
        })),
    )
}

async fn handle_set_commands_ok(
    Json(_payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({"ok": true, "result": true})),
    )
}

async fn handle_chat_action_ok(
    Json(_payload): Json<serde_json::Value>,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({"ok": true, "result": true})),
    )
}

pub(super) async fn spawn_mock_telegram_polling_media_api()
-> Result<Option<(String, PollingMediaState, tokio::task::JoinHandle<()>)>> {
    let state = PollingMediaState::default();
    let app = Router::new()
        .route(
            "/botfake-token/getUpdates",
            post(handle_get_updates_with_photo),
        )
        .route("/botfake-token/getFile", post(handle_get_file_for_photo))
        .route("/botfake-token/setMyCommands", post(handle_set_commands_ok))
        .route("/botfake-token/sendChatAction", post(handle_chat_action_ok))
        .with_state(state.clone());
    let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("skipping telegram polling media tests: local socket bind is not permitted");
            return Ok(None);
        }
        Err(err) => return Err(err.into()),
    };
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    wait_for_listener(addr).await;
    Ok(Some((format!("http://{addr}"), state, handle)))
}

async fn wait_for_listener(addr: std::net::SocketAddr) {
    for _ in 0..20 {
        if tokio::net::TcpStream::connect(addr).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}
