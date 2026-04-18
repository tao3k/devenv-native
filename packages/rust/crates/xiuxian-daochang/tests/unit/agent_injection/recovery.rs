use std::time::Duration;

use anyhow::Result;
use tokio::time::sleep;
use xiuxian_daochang::{Agent, SessionStore};
use xiuxian_qianhuan::InjectionPolicy;

use super::support::{
    MockLlmScenario, base_config, find_message_by_name, has_next_turn_hint,
    latest_stream_event_fields, live_redis_url, payload_messages, require_ok, require_some,
    reserve_local_addr, spawn_mock_bridge_server, spawn_mock_llm_server, unique_key_prefix,
};

#[tokio::test]
async fn graph_shortcut_publishes_route_trace_to_route_events_stream() -> Result<()> {
    let Some(redis_url) = live_redis_url() else {
        return Ok(());
    };

    let addr = reserve_local_addr().await;
    let (server_handle, _recorded_arguments) = spawn_mock_bridge_server(addr).await;
    let tool_url = format!("http://{addr}/sse");
    let key_prefix = unique_key_prefix("xiuxian-daochang-route-trace");

    let config = base_config(
        "http://127.0.0.1:4000/v1/chat/completions".to_string(),
        tool_url,
    );
    let session =
        SessionStore::new_with_redis(redis_url.clone(), Some(key_prefix.clone()), Some(120))?;
    let agent = Agent::from_config_with_session_backends_for_test(config, session, None).await?;
    let session_id = "telegram:-100400:11";

    let output = agent
        .run_turn(
            session_id,
            r#"graph bridge.echo {"task":"route-trace-stream"}"#,
        )
        .await?;
    let payload: serde_json::Value = serde_json::from_str(&output)?;
    assert_eq!(payload["task"], "route-trace-stream");

    let mut fields = None;
    for _ in 0..30 {
        fields = latest_stream_event_fields(&redis_url, &key_prefix, "route.events").await?;
        if fields.is_some() {
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }

    let fields = require_some(fields, "expected route trace stream event in route.events");
    assert_eq!(
        fields.get("kind").map(String::as_str),
        Some("session.route.trace_emitted")
    );
    assert_eq!(
        fields.get("session_id").map(String::as_str),
        Some(session_id)
    );
    assert_eq!(
        fields.get("selected_route").map(String::as_str),
        Some("graph")
    );
    assert_eq!(
        fields.get("workflow_mode").map(String::as_str),
        Some("graph")
    );
    assert_eq!(
        fields.get("graph_steps_count").map(String::as_str),
        Some("3")
    );
    assert!(
        fields
            .get("plan_id")
            .is_some_and(|value| !value.trim().is_empty()),
        "plan_id should be persisted for graph route trace stream events"
    );
    assert!(
        fields
            .get("graph_steps_json")
            .is_some_and(|value| value.contains("invoke_graph_tool")),
        "graph_steps_json should include invoke_graph_tool step"
    );

    let trace_json = require_some(
        fields.get("route_trace_json"),
        "route_trace_json should exist",
    );
    let trace: serde_json::Value = require_ok(
        serde_json::from_str(trace_json),
        "route_trace_json should be valid json",
    );
    assert_eq!(trace["selected_route"], "graph");
    assert_eq!(trace["workflow_mode"], "graph");
    assert_eq!(trace["session_id"], session_id);
    assert_eq!(trace["graph_steps"].as_array().map(Vec::len), Some(3));

    let stream_key = format!("{key_prefix}:stream:route.events");
    let client = redis::Client::open(redis_url)?;
    let mut conn = client.get_multiplexed_async_connection().await?;
    let _: () = redis::cmd("DEL")
        .arg(stream_key)
        .query_async(&mut conn)
        .await?;

    server_handle.abort();
    let _ = server_handle.await;
    Ok(())
}

#[tokio::test]
async fn react_failure_reflection_injects_next_turn_hint_and_recovers() -> Result<()> {
    let tool_addr = reserve_local_addr().await;
    let (tool_server, recorded_arguments) = spawn_mock_bridge_server(tool_addr).await;
    let llm_addr = reserve_local_addr().await;
    let (llm_server, llm_requests) =
        spawn_mock_llm_server(llm_addr, MockLlmScenario::ReflectionHintRecovery).await;
    let tool_url = format!("http://{tool_addr}/sse");
    let inference_url = format!("http://{llm_addr}/v1/chat/completions");
    let agent = Agent::from_config(base_config(inference_url, tool_url)).await?;
    let session_id = "telegram:-100500:10";

    let first_attempt = agent
        .run_turn(session_id, "trigger correction flow with a failing tool")
        .await;
    let Err(first_error) = first_attempt else {
        panic!("first turn should fail and trigger reflection");
    };
    assert!(
        first_error
            .to_string()
            .contains("forced tool failure for resilience tests")
    );

    let output = agent
        .run_turn(session_id, "retry after reflection correction")
        .await?;
    assert_eq!(output, "react-ok");

    let llm_payloads = require_ok(llm_requests.lock(), "mock llm requests lock poisoned").clone();
    assert_eq!(
        llm_payloads.len(),
        3,
        "expected 1 request on failed turn and 2 requests on recovered turn"
    );
    assert!(
        !has_next_turn_hint(&llm_payloads[0]),
        "first turn must not include next-turn hint before reflection exists"
    );
    assert!(
        has_next_turn_hint(&llm_payloads[1]),
        "second turn should inject stored next-turn hint"
    );
    let hint_message = find_message_by_name(&llm_payloads[1], "agent.next_turn_hint")
        .and_then(|message| message.get("content"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    assert!(hint_message.contains("reason=previous_turn_error_requires_verification"));
    assert!(hint_message.contains("role_mix_profile=recovery"));

    let captured = require_ok(
        recorded_arguments.lock(),
        "recorded arguments lock poisoned",
    )
    .clone();
    assert_eq!(
        captured.len(),
        2,
        "expected two tool invocations (one forced failure + one corrected call)"
    );
    assert_eq!(captured[1]["task"], "corrected-by-next-turn-hint");

    tool_server.abort();
    let _ = tool_server.await;
    llm_server.abort();
    let _ = llm_server.await;
    Ok(())
}

#[tokio::test]
async fn react_budget_pressure_truncates_tool_payload_and_keeps_core_injection_anchor() -> Result<()>
{
    let tool_addr = reserve_local_addr().await;
    let (tool_server, _recorded_arguments) = spawn_mock_bridge_server(tool_addr).await;
    let llm_addr = reserve_local_addr().await;
    let (llm_server, llm_requests) =
        spawn_mock_llm_server(llm_addr, MockLlmScenario::LargePayloadBudgetPressure).await;
    let tool_url = format!("http://{tool_addr}/sse");
    let inference_url = format!("http://{llm_addr}/v1/chat/completions");

    let mut config = base_config(inference_url, tool_url);
    config.context_budget_tokens = Some(140);
    config.context_budget_reserve_tokens = 20;
    let agent = Agent::from_config(config).await?;
    let session_id = "telegram:-100500:11";

    agent
        .upsert_session_system_prompt_injection_xml(
            session_id,
            r"
<system_prompt_injection>
  <qa>
    <q>core anchor</q>
    <a>Keep genesis_rules and persona_steering anchors available under budget pressure.</a>
  </qa>
</system_prompt_injection>
",
        )
        .await?;

    let first = agent
        .run_turn(session_id, "run budget-pressure payload test turn one")
        .await?;
    assert_eq!(first, "budget-ok");

    let second = agent
        .run_turn(session_id, "run budget-pressure payload test turn two")
        .await?;
    assert_eq!(second, "budget-ok");

    let llm_payloads = require_ok(llm_requests.lock(), "mock llm requests lock poisoned").clone();
    assert!(
        llm_payloads.len() >= 4,
        "expected two turns, each with tool-plan + final-answer calls"
    );

    let tool_message_content = payload_messages(&llm_payloads[1])
        .iter()
        .find(|message| message.get("role").and_then(serde_json::Value::as_str) == Some("tool"))
        .and_then(|message| message.get("content"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    assert!(
        tool_message_content.chars().count() <= InjectionPolicy::default().max_chars,
        "tool payload should be truncated by injection policy max_chars"
    );

    let second_turn_first_payload = &llm_payloads[2];
    let injection_message = find_message_by_name(
        second_turn_first_payload,
        "agent.system_prompt.injection.context",
    )
    .and_then(|message| message.get("content"))
    .and_then(serde_json::Value::as_str)
    .unwrap_or_default();
    assert!(
        injection_message.contains("genesis_rules"),
        "core genesis_rules anchor should survive budget pressure"
    );
    assert!(
        injection_message.contains("persona_steering"),
        "core persona_steering anchor should survive budget pressure"
    );

    let snapshot = require_some(
        agent.inspect_context_budget_snapshot(session_id).await,
        "context budget snapshot should be recorded",
    );
    assert!(
        snapshot.pre_tokens > snapshot.post_tokens,
        "budget pressure scenario should drop/truncate context tokens"
    );

    tool_server.abort();
    let _ = tool_server.await;
    llm_server.abort();
    let _ = llm_server.await;
    Ok(())
}

#[tokio::test]
async fn react_role_mix_switches_from_recovery_back_to_normal_after_failure_cycle() -> Result<()> {
    let tool_addr = reserve_local_addr().await;
    let (tool_server, recorded_arguments) = spawn_mock_bridge_server(tool_addr).await;
    let llm_addr = reserve_local_addr().await;
    let (llm_server, llm_requests) =
        spawn_mock_llm_server(llm_addr, MockLlmScenario::RoleMixSwitch).await;
    let tool_url = format!("http://{tool_addr}/sse");
    let inference_url = format!("http://{llm_addr}/v1/chat/completions");
    let agent = Agent::from_config(base_config(inference_url, tool_url)).await?;
    let session_id = "telegram:-100500:12";

    let first_attempt = agent.run_turn(session_id, "trigger role mix failure").await;
    assert!(
        first_attempt.is_err(),
        "first turn should fail and arm recovery role mix"
    );

    let recovered = agent
        .run_turn(session_id, "second turn should enter recovery profile")
        .await?;
    assert_eq!(recovered, "role-mix-recovery-ok");

    let normal = agent
        .run_turn(session_id, "third turn should return to normal profile")
        .await?;
    assert_eq!(normal, "role-mix-normal");

    let llm_payloads = require_ok(llm_requests.lock(), "mock llm requests lock poisoned").clone();
    assert!(
        llm_payloads.len() >= 4,
        "expected failure turn + recovery turn + normal turn request sequence"
    );

    let recovery_hint = find_message_by_name(&llm_payloads[1], "agent.next_turn_hint")
        .and_then(|message| message.get("content"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    assert!(
        recovery_hint.contains("role_mix_profile=recovery"),
        "failure-following turn should switch into recovery role mix"
    );
    assert!(
        !has_next_turn_hint(&llm_payloads[3]),
        "subsequent normal turn should not keep stale recovery hint"
    );

    let captured = require_ok(
        recorded_arguments.lock(),
        "recorded arguments lock poisoned",
    )
    .clone();
    assert_eq!(
        captured.len(),
        2,
        "recovery cycle should invoke failing tool once and recovery tool once"
    );
    assert_eq!(captured[1]["task"], "role-mix-recovery");

    tool_server.abort();
    let _ = tool_server.await;
    llm_server.abort();
    let _ = llm_server.await;
    Ok(())
}
