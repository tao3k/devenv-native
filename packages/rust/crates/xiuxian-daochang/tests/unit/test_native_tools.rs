//! Native-tool registry smoke tests.

use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use xiuxian_daochang::{NativeTool, NativeToolCallContext, NativeToolRegistry};

struct MockTool;
struct AlphaTool;

#[async_trait]
impl NativeTool for MockTool {
    fn name(&self) -> &'static str {
        "mock.test"
    }
    fn description(&self) -> &'static str {
        "Mock tool for testing"
    }
    fn parameters(&self) -> serde_json::Value {
        json!({})
    }
    async fn call(
        &self,
        _args: Option<serde_json::Value>,
        _context: &NativeToolCallContext,
    ) -> anyhow::Result<String> {
        Ok("Mock success".to_string())
    }
}

#[async_trait]
impl NativeTool for AlphaTool {
    fn name(&self) -> &'static str {
        "alpha.test"
    }
    fn description(&self) -> &'static str {
        "Alphabetically earlier tool"
    }
    fn parameters(&self) -> serde_json::Value {
        json!({})
    }
    async fn call(
        &self,
        _args: Option<serde_json::Value>,
        _context: &NativeToolCallContext,
    ) -> anyhow::Result<String> {
        Ok("Alpha success".to_string())
    }
}

#[tokio::test]
async fn test_native_tool_registration_and_dispatch() {
    let mut registry = NativeToolRegistry::new();
    registry.register(Arc::new(MockTool));

    let tool = registry
        .get("mock.test")
        .unwrap_or_else(|| panic!("tool should be registered"));
    assert_eq!(tool.name(), "mock.test");

    let result = tool
        .call(None, &NativeToolCallContext::default())
        .await
        .unwrap_or_else(|error| panic!("call should succeed: {error}"));
    assert_eq!(result, "Mock success");
}

#[test]
fn test_registry_summary_injection() {
    let mut registry = NativeToolRegistry::new();
    registry.register(Arc::new(MockTool));

    let summary = registry.get_registry_summary();
    assert!(
        summary.contains("mock.test"),
        "Summary should contain tool name"
    );
    assert!(
        summary.contains("Native Core Tools"),
        "Summary should have standard prefix"
    );
}

#[test]
fn test_list_for_llm_is_sorted_by_tool_name() {
    let mut registry = NativeToolRegistry::new();
    registry.register(Arc::new(MockTool));
    registry.register(Arc::new(AlphaTool));

    let tools = registry.list_for_llm();
    let names = tools
        .iter()
        .filter_map(|tool| tool.get("name").and_then(serde_json::Value::as_str))
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["alpha.test", "mock.test"]);
}
