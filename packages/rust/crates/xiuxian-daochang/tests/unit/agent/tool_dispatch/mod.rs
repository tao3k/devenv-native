use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

use super::super::Agent;
use super::diagnostics::tool_timeout_error_output;
use crate::agent::native_tools::registry::{NativeTool, NativeToolCallContext};
use crate::{AgentConfig, NativeToolRegistry};

struct RecordingTool {
    seen_context: Arc<Mutex<Option<NativeToolCallContext>>>,
}

#[async_trait]
impl NativeTool for RecordingTool {
    fn name(&self) -> &'static str {
        "mock.record_context"
    }

    fn description(&self) -> &'static str {
        "Records the invocation context for assertions."
    }

    fn parameters(&self) -> serde_json::Value {
        json!({})
    }

    async fn call(
        &self,
        _arguments: Option<serde_json::Value>,
        context: &NativeToolCallContext,
    ) -> Result<String> {
        *self.seen_context.lock().unwrap_or_else(|error| {
            panic!("recording tool mutex should not be poisoned: {error}")
        }) = Some(context.clone());
        Ok("recorded".to_string())
    }
}

async fn build_agent_with_native_tool(tool: Arc<dyn NativeTool>) -> Result<Agent> {
    let mut registry = NativeToolRegistry::new();
    registry.register(tool);

    let mut agent = Agent::from_config(AgentConfig {
        inference_url: "http://127.0.0.1:4000/v1/chat/completions".to_string(),
        ..AgentConfig::default()
    })
    .await?;
    agent.native_tools = Arc::new(registry);
    Ok(agent)
}

#[tokio::test]
async fn native_dispatch_preserves_tool_call_id_in_output_and_context() -> Result<()> {
    let seen_context = Arc::new(Mutex::new(None));
    let tool = Arc::new(RecordingTool {
        seen_context: Arc::clone(&seen_context),
    });
    let agent = build_agent_with_native_tool(tool).await?;

    let output = agent
        .call_tool_with_diagnostics(
            Some("telegram:1304799691"),
            Some("call_123"),
            "mock.record_context",
            None,
        )
        .await?;

    assert_eq!(output.text, "recorded");
    assert!(!output.is_error);
    assert_eq!(output.tool_call_id.as_deref(), Some("call_123"));

    let context = seen_context
        .lock()
        .unwrap_or_else(|error| panic!("recording tool mutex should not be poisoned: {error}"))
        .clone()
        .unwrap_or_else(|| panic!("native tool should have received a call context"));
    assert_eq!(context.session_id.as_deref(), Some("telegram:1304799691"));
    assert_eq!(context.tool_call_id.as_deref(), Some("call_123"));
    Ok(())
}

#[test]
fn timeout_output_preserves_tool_call_id() {
    let output = tool_timeout_error_output("native", "mock.record_context", 5, Some("call_456"));

    assert!(output.is_error);
    assert_eq!(output.tool_call_id.as_deref(), Some("call_456"));
    assert!(output.text.contains("timeout"));
}

#[test]
fn soft_fail_output_preserves_tool_call_id() {
    let error = anyhow::anyhow!("embedding timed out: tool runtime error: -32603");
    let output =
        Agent::soft_fail_tool_error_output("memory.search_memory", Some("call_789"), &error)
            .unwrap_or_else(|| panic!("embedding timeout should degrade to soft tool output"));

    assert!(output.is_error);
    assert_eq!(output.tool_call_id.as_deref(), Some("call_789"));
    assert!(output.text.contains("embedding_timeout"));
}
