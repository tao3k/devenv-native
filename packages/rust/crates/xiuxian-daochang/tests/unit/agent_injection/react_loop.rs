use std::time::{Duration, Instant};

use anyhow::Result;
use xiuxian_daochang::{Agent, MemoryConfig};

use super::support::{
    MockLlmScenario, base_config, find_tool_message, require_ok, require_some, reserve_local_addr,
    spawn_mock_bridge_server, spawn_mock_llm_server,
};

#[tokio::test]
async fn react_loop_tool_call_roundtrip_with_mock_llm_and_tool_runtime() -> Result<()> {
    let tool_addr = reserve_local_addr().await;
    let (tool_server, recorded_arguments) = spawn_mock_bridge_server(tool_addr).await;
    let llm_addr = reserve_local_addr().await;
    let (llm_server, llm_requests) =
        spawn_mock_llm_server(llm_addr, MockLlmScenario::ValidToolArguments).await;
    let tool_url = format!("http://{tool_addr}/sse");
    let inference_url = format!("http://{llm_addr}/v1/chat/completions");

    let agent = Agent::from_config(base_config(inference_url, tool_url)).await?;
    let output = agent
        .run_turn(
            "telegram:-100200:42",
            "use bridge.echo to complete the task",
        )
        .await?;
    assert_eq!(output, "react-ok");

    let captured = require_ok(
        recorded_arguments.lock(),
        "recorded arguments lock poisoned",
    )
    .clone();
    assert_eq!(captured.len(), 1, "react flow should issue one tool call");
    assert_eq!(captured[0]["task"], "react-loop");

    let llm_payloads = require_ok(llm_requests.lock(), "mock llm requests lock poisoned").clone();
    assert_eq!(
        llm_payloads.len(),
        2,
        "react loop should call LLM twice (tool plan + final answer)"
    );
    assert!(
        llm_payloads[0]
            .get("tools")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|tools| !tools.is_empty()),
        "first LLM request should include tool definitions"
    );
    assert!(
        find_tool_message(&llm_payloads[1]).is_some(),
        "second LLM request should include tool result message"
    );
    let tool_message = require_some(
        find_tool_message(&llm_payloads[1]),
        "second LLM request should contain tool message",
    );
    assert_eq!(
        tool_message
            .get("tool_call_id")
            .and_then(serde_json::Value::as_str),
        Some("call_1"),
        "tool result message should preserve original tool_call_id"
    );

    tool_server.abort();
    let _ = tool_server.await;
    llm_server.abort();
    let _ = llm_server.await;
    Ok(())
}

#[tokio::test]
async fn react_shortcut_strips_prefix_before_llm_prompt() -> Result<()> {
    let tool_addr = reserve_local_addr().await;
    let (tool_server, _recorded_arguments) = spawn_mock_bridge_server(tool_addr).await;
    let llm_addr = reserve_local_addr().await;
    let (llm_server, llm_requests) =
        spawn_mock_llm_server(llm_addr, MockLlmScenario::ValidToolArguments).await;
    let tool_url = format!("http://{tool_addr}/sse");
    let inference_url = format!("http://{llm_addr}/v1/chat/completions");

    let agent = Agent::from_config(base_config(inference_url, tool_url)).await?;
    let output = agent
        .run_turn(
            "telegram:-100300:7",
            "!react call bridge.echo with task react-loop",
        )
        .await?;
    assert_eq!(output, "react-ok");

    let llm_payloads = require_ok(llm_requests.lock(), "mock llm requests lock poisoned").clone();
    let first_messages = llm_payloads[0]
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("first llm payload must include messages"));
    let user_message = first_messages
        .iter()
        .rev()
        .find(|message| message.get("role").and_then(serde_json::Value::as_str) == Some("user"))
        .and_then(|message| message.get("content"))
        .and_then(serde_json::Value::as_str);
    assert_eq!(
        user_message,
        Some("call bridge.echo with task react-loop"),
        "`!react` prefix should be removed before sending prompt to LLM"
    );

    tool_server.abort();
    let _ = tool_server.await;
    llm_server.abort();
    let _ = llm_server.await;
    Ok(())
}

#[tokio::test]
async fn react_loop_malformed_tool_arguments_fall_back_to_empty_object() -> Result<()> {
    let tool_addr = reserve_local_addr().await;
    let (tool_server, recorded_arguments) = spawn_mock_bridge_server(tool_addr).await;
    let llm_addr = reserve_local_addr().await;
    let (llm_server, _llm_requests) =
        spawn_mock_llm_server(llm_addr, MockLlmScenario::MalformedToolArguments).await;
    let tool_url = format!("http://{tool_addr}/sse");
    let inference_url = format!("http://{llm_addr}/v1/chat/completions");

    let mut config = base_config(inference_url, tool_url);
    let temp_dir = tempfile::tempdir()?;
    config.memory = Some(MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name: "react_malformed_tool_arguments".to_string(),
        persistence_backend: "local".to_string(),
        embedding_base_url: Some("http://127.0.0.1:9".to_string()),
        ..MemoryConfig::default()
    });
    let agent = Agent::from_config(config).await?;

    let output = agent
        .run_turn("telegram:-100300:8", "simulate malformed tool arguments")
        .await?;
    assert_eq!(output, "react-ok");

    let captured = require_ok(
        recorded_arguments.lock(),
        "recorded arguments lock poisoned",
    )
    .clone();
    assert_eq!(captured.len(), 1, "expected one tool call");
    assert!(
        captured[0]
            .as_object()
            .is_some_and(serde_json::Map::is_empty),
        "invalid JSON tool arguments should degrade to empty object"
    );

    tool_server.abort();
    let _ = tool_server.await;
    llm_server.abort();
    let _ = llm_server.await;
    Ok(())
}

#[tokio::test]
async fn react_loop_tool_timeout_is_ko_and_turn_continues() -> Result<()> {
    let tool_addr = reserve_local_addr().await;
    let (tool_server, recorded_arguments) = spawn_mock_bridge_server(tool_addr).await;
    let llm_addr = reserve_local_addr().await;
    let (llm_server, llm_requests) =
        spawn_mock_llm_server(llm_addr, MockLlmScenario::ToolTimeoutRecovery).await;
    let tool_url = format!("http://{tool_addr}/sse");
    let inference_url = format!("http://{llm_addr}/v1/chat/completions");

    let mut config = base_config(inference_url, tool_url);
    config.tool_timeout_secs = 1;
    let agent = Agent::from_config(config).await?;

    let started = Instant::now();
    let output = agent
        .run_turn("telegram:-100300:9", "trigger a hanging tool and continue")
        .await?;
    let elapsed = started.elapsed();
    assert_eq!(output, "timeout-recovered-ok");
    assert!(
        elapsed < Duration::from_secs(4),
        "tool timeout KO should keep latency bounded; elapsed={elapsed:?}"
    );

    let captured = require_ok(
        recorded_arguments.lock(),
        "recorded arguments lock poisoned",
    )
    .clone();
    assert_eq!(
        captured.len(),
        1,
        "timeout scenario should issue exactly one tool call"
    );
    assert_eq!(captured[0]["delay_ms"], 5000);

    let llm_payloads = require_ok(llm_requests.lock(), "mock llm requests lock poisoned").clone();
    assert_eq!(
        llm_payloads.len(),
        2,
        "expected plan + final response rounds"
    );
    let tool_message = require_some(
        find_tool_message(&llm_payloads[1]),
        "timeout recovery round should include tool result message",
    );
    assert_eq!(
        tool_message
            .get("tool_call_id")
            .and_then(serde_json::Value::as_str),
        Some("call_1"),
        "timeout degradation should preserve original tool_call_id"
    );
    let tool_content = tool_message
        .get("content")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    assert!(
        tool_content.contains("\"error_kind\":\"timeout\""),
        "tool message should surface timeout degradation payload"
    );
    assert!(
        tool_content.contains("\"tool\":\"bridge.hang\""),
        "tool message should identify the timed-out tool"
    );

    tool_server.abort();
    let _ = tool_server.await;
    llm_server.abort();
    let _ = llm_server.await;
    Ok(())
}
