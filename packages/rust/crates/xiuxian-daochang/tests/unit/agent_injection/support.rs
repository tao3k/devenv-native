use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::unit::live_gates::{live_valkey_enabled, resolve_live_valkey_url};
use crate::unit::tool_runtime_mock::{
    MockCallToolReply, MockToolRuntimeConfig, call_handler, permissive_tool_definition,
    spawn_mock_tool_runtime, text_result,
};
use anyhow::Result;
use axum::{Json, Router, extract::State, routing::post};
use tokio::time::sleep;
use xiuxian_daochang::{AgentConfig, ToolServerEntry, set_config_home_override};

pub(super) fn require_ok<T, E>(result: std::result::Result<T, E>, context: &str) -> T
where
    E: std::fmt::Display,
{
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

pub(super) fn require_some<T>(value: Option<T>, context: &str) -> T {
    match value {
        Some(value) => value,
        None => panic!("{context}"),
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) enum MockLlmScenario {
    ValidToolArguments,
    MalformedToolArguments,
    ReflectionHintRecovery,
    RoleMixSwitch,
    LargePayloadBudgetPressure,
    ToolTimeoutRecovery,
}

#[derive(Clone)]
struct MockLlmServerState {
    requests: Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
    scenario: MockLlmScenario,
    round: Arc<AtomicUsize>,
}

struct MockLlmRequestFacts {
    has_tool_response: bool,
    next_turn_hint: Option<String>,
    latest_user_message: String,
}

fn llm_text_response(content: &str) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": "mock-chatcmpl-text",
        "object": "chat.completion",
        "created": 0,
        "model": "test-model",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": content
            },
            "finish_reason": "stop"
        }]
    }))
}

fn llm_tool_response(name: &str, arguments: &str) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": "mock-chatcmpl-tool",
        "object": "chat.completion",
        "created": 0,
        "model": "test-model",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": {
                        "name": name,
                        "arguments": arguments
                    }
                }]
            },
            "finish_reason": "tool_calls"
        }]
    }))
}

fn llm_role_mix_normal_response() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": "mock-chatcmpl-role-mix-normal",
        "object": "chat.completion",
        "created": 0,
        "model": "test-model",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "role-mix-normal",
                "tool_calls": null
            },
            "finish_reason": "stop"
        }]
    }))
}

fn collect_mock_llm_request_facts(payload: &serde_json::Value) -> MockLlmRequestFacts {
    let has_tool_response = payload_messages(payload)
        .iter()
        .any(|message| message.get("role").and_then(serde_json::Value::as_str) == Some("tool"));
    let next_turn_hint = find_message_by_name(payload, "agent.next_turn_hint")
        .and_then(|message| message.get("content"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let latest_user_message = payload_messages(payload)
        .iter()
        .rev()
        .find_map(|message| {
            (message.get("role").and_then(serde_json::Value::as_str) == Some("user")).then(|| {
                message
                    .get("content")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default()
                    .to_string()
            })
        })
        .unwrap_or_default();
    MockLlmRequestFacts {
        has_tool_response,
        next_turn_hint,
        latest_user_message,
    }
}

fn mock_llm_scenario_response(
    scenario: MockLlmScenario,
    facts: &MockLlmRequestFacts,
) -> Json<serde_json::Value> {
    match scenario {
        MockLlmScenario::ValidToolArguments => {
            if facts.has_tool_response {
                return llm_text_response("react-ok");
            }
            llm_tool_response("bridge.echo", r#"{"task":"react-loop"}"#)
        }
        MockLlmScenario::MalformedToolArguments => {
            if facts.has_tool_response {
                return llm_text_response("react-ok");
            }
            llm_tool_response("bridge.echo", "{not-json")
        }
        MockLlmScenario::ReflectionHintRecovery => {
            if facts.has_tool_response {
                return llm_text_response("react-ok");
            }
            if facts.next_turn_hint.is_some() {
                return llm_tool_response(
                    "bridge.echo",
                    r#"{"task":"corrected-by-next-turn-hint"}"#,
                );
            }
            llm_tool_response("bridge.always_fail", "{}")
        }
        MockLlmScenario::RoleMixSwitch => {
            if facts.has_tool_response {
                return llm_text_response("role-mix-recovery-ok");
            }
            if let Some(hint) = facts.next_turn_hint.as_deref()
                && hint.contains("role_mix_profile=recovery")
            {
                return llm_tool_response("bridge.echo", r#"{"task":"role-mix-recovery"}"#);
            }
            if facts
                .latest_user_message
                .contains("trigger role mix failure")
            {
                return llm_tool_response("bridge.always_fail", "{}");
            }
            llm_role_mix_normal_response()
        }
        MockLlmScenario::LargePayloadBudgetPressure => {
            if facts.has_tool_response {
                return llm_text_response("budget-ok");
            }
            llm_tool_response("bridge.large_payload", r#"{"size":12000}"#)
        }
        MockLlmScenario::ToolTimeoutRecovery => {
            if facts.has_tool_response {
                return llm_text_response("timeout-recovered-ok");
            }
            llm_tool_response("bridge.hang", r#"{"delay_ms":5000}"#)
        }
    }
}

async fn mock_llm_chat_handler(
    State(state): State<MockLlmServerState>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    require_ok(state.requests.lock(), "mock llm requests lock poisoned").push(payload.clone());
    let _round = state.round.fetch_add(1, Ordering::SeqCst);
    let facts = collect_mock_llm_request_facts(&payload);
    mock_llm_scenario_response(state.scenario, &facts)
}

pub(super) async fn reserve_local_addr() -> std::net::SocketAddr {
    let probe = require_ok(
        tokio::net::TcpListener::bind("127.0.0.1:0").await,
        "reserve local addr",
    );
    let addr = require_ok(probe.local_addr(), "read reserved local addr");
    drop(probe);
    addr
}

pub(super) async fn spawn_mock_bridge_server(
    addr: std::net::SocketAddr,
) -> (
    tokio::task::JoinHandle<()>,
    Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
) {
    let recorded_arguments = Arc::new(std::sync::Mutex::new(Vec::new()));
    let reject_metadata_once_for_flaky = Arc::new(AtomicBool::new(true));
    let server = spawn_mock_tool_runtime(
        addr,
        MockToolRuntimeConfig::with_static_tools(
            vec![
                permissive_tool_definition("bridge.echo", "Echo JSON arguments"),
                permissive_tool_definition("bridge.flaky", "Reject first metadata-rich call"),
                permissive_tool_definition("bridge.always_fail", "Always fail tool invocation"),
                permissive_tool_definition(
                    "bridge.hang",
                    "Sleep long enough to trigger timeout KO",
                ),
                permissive_tool_definition(
                    "bridge.large_payload",
                    "Return very large tool payload",
                ),
            ],
            call_handler({
                let recorded_arguments = Arc::clone(&recorded_arguments);
                let reject_metadata_once_for_flaky = Arc::clone(&reject_metadata_once_for_flaky);
                move |request| {
                    let recorded_arguments = Arc::clone(&recorded_arguments);
                    let reject_metadata_once_for_flaky =
                        Arc::clone(&reject_metadata_once_for_flaky);
                    async move {
                        let args_json = request
                            .arguments
                            .clone()
                            .map_or_else(|| serde_json::json!({}), serde_json::Value::Object);
                        require_ok(
                            recorded_arguments.lock(),
                            "recorded arguments lock poisoned",
                        )
                        .push(args_json.clone());

                        match request.name.as_str() {
                            "bridge.flaky" => {
                                let has_metadata = request
                                    .arguments
                                    .as_ref()
                                    .and_then(|value| value.get("_omni"))
                                    .is_some();
                                if has_metadata
                                    && reject_metadata_once_for_flaky.swap(false, Ordering::SeqCst)
                                {
                                    return MockCallToolReply::RpcError {
                                        code: -32_603,
                                        message: "metadata not accepted for first attempt"
                                            .to_string(),
                                        data: None,
                                    };
                                }
                                MockCallToolReply::Result(text_result("fallback-ok"))
                            }
                            "bridge.always_fail" => MockCallToolReply::RpcError {
                                code: -32_603,
                                message: "forced tool failure for resilience tests".to_string(),
                                data: None,
                            },
                            "bridge.hang" => {
                                let delay_ms = request
                                    .arguments
                                    .as_ref()
                                    .and_then(|value| value.get("delay_ms"))
                                    .and_then(serde_json::Value::as_u64)
                                    .unwrap_or(5_000)
                                    .clamp(500, 30_000);
                                sleep(Duration::from_millis(delay_ms)).await;
                                MockCallToolReply::Result(text_result("hang-finished"))
                            }
                            "bridge.large_payload" => {
                                let size = request
                                    .arguments
                                    .as_ref()
                                    .and_then(|value| value.get("size"))
                                    .and_then(serde_json::Value::as_u64)
                                    .and_then(|value| usize::try_from(value).ok())
                                    .unwrap_or(12_000)
                                    .clamp(256, 20_000);
                                MockCallToolReply::Result(text_result("X".repeat(size)))
                            }
                            _ => {
                                let payload = serde_json::to_string(&args_json)
                                    .unwrap_or_else(|_| "{\"error\":\"serialize\"}".to_string());
                                MockCallToolReply::Result(text_result(payload))
                            }
                        }
                    }
                }
            }),
        ),
    )
    .await;

    (server, recorded_arguments)
}

pub(super) async fn spawn_mock_llm_server(
    addr: std::net::SocketAddr,
    scenario: MockLlmScenario,
) -> (
    tokio::task::JoinHandle<()>,
    Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
) {
    let requests = Arc::new(std::sync::Mutex::new(Vec::new()));
    let state = MockLlmServerState {
        requests: Arc::clone(&requests),
        scenario,
        round: Arc::new(AtomicUsize::new(0)),
    };
    let app = Router::new()
        .route("/v1/chat/completions", post(mock_llm_chat_handler))
        .with_state(state);
    let listener = require_ok(
        tokio::net::TcpListener::bind(addr).await,
        "bind mock llm listener",
    );
    (
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        }),
        requests,
    )
}

fn ensure_http_llm_backend_for_tests() {
    static CONFIG_HOME: OnceLock<PathBuf> = OnceLock::new();
    let path = CONFIG_HOME.get_or_init(|| {
        let root = std::env::temp_dir()
            .join("xiuxian-daochang-tests")
            .join("agent_injection");
        let settings_dir = root.join("xiuxian-artisan-workshop");
        require_ok(
            std::fs::create_dir_all(&settings_dir),
            "create isolated config home for tests",
        );
        require_ok(
            std::fs::write(
                settings_dir.join("xiuxian.toml"),
                "[agent]\nllm_backend = \"http\"\nagenda_validation_policy = \"never\"\n",
            ),
            "write isolated runtime settings for tests",
        );
        root
    });
    set_config_home_override(path.clone());
}

pub(super) fn base_config(inference_url: String, tool_url: String) -> AgentConfig {
    ensure_http_llm_backend_for_tests();
    AgentConfig {
        inference_url,
        model: "test-model".to_string(),
        tool_servers: vec![ToolServerEntry {
            name: "mock".to_string(),
            url: Some(tool_url),
            command: None,
            args: None,
        }],
        tool_handshake_timeout_secs: 2,
        tool_connect_retries: 2,
        tool_connect_retry_backoff_ms: 50,
        tool_timeout_secs: 15,
        tool_list_cache_ttl_ms: 100,
        max_tool_rounds: 3,
        ..AgentConfig::default()
    }
}

pub(super) fn live_redis_url() -> Option<String> {
    if !live_valkey_enabled() {
        return None;
    }
    resolve_live_valkey_url()
}

pub(super) fn unique_key_prefix(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{prefix}-{nanos}")
}

pub(super) async fn latest_stream_event_fields(
    redis_url: &str,
    key_prefix: &str,
    stream_name: &str,
) -> Result<Option<std::collections::HashMap<String, String>>> {
    let client = redis::Client::open(redis_url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;
    let stream_key = format!("{key_prefix}:stream:{stream_name}");
    let entries: Vec<(String, std::collections::HashMap<String, String>)> = redis::cmd("XREVRANGE")
        .arg(&stream_key)
        .arg("+")
        .arg("-")
        .arg("COUNT")
        .arg(1)
        .query_async(&mut conn)
        .await?;
    Ok(entries.into_iter().next().map(|(_, fields)| fields))
}

pub(super) fn payload_messages(payload: &serde_json::Value) -> &[serde_json::Value] {
    payload
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .map_or(&[], Vec::as_slice)
}

pub(super) fn find_message_by_name<'a>(
    payload: &'a serde_json::Value,
    name: &str,
) -> Option<&'a serde_json::Value> {
    payload_messages(payload)
        .iter()
        .find(|message| message.get("name").and_then(serde_json::Value::as_str) == Some(name))
}

pub(super) fn has_next_turn_hint(payload: &serde_json::Value) -> bool {
    find_message_by_name(payload, "agent.next_turn_hint").is_some()
}

pub(super) fn find_tool_message(payload: &serde_json::Value) -> Option<&serde_json::Value> {
    payload_messages(payload)
        .iter()
        .find(|message| message.get("role").and_then(serde_json::Value::as_str) == Some("tool"))
}
