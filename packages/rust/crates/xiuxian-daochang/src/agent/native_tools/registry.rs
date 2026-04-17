use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Per-invocation context injected by the host runtime.
#[derive(Debug, Clone, Default)]
pub struct NativeToolCallContext {
    /// Session scope key (for example `telegram:1304799691`).
    pub session_id: Option<String>,
    /// Original tool call id when the invocation came from an LLM tool call.
    pub tool_call_id: Option<String>,
}

/// Interface for tools implemented directly in Rust.
/// These tools run in-process with 100% reliability.
#[async_trait]
pub trait NativeTool: Send + Sync {
    /// Unique name of the tool (e.g., "journal.record").
    fn name(&self) -> &str;

    /// Description provided to the LLM to explain the tool's usage.
    fn description(&self) -> &str;

    /// JSON Schema for the tool's parameters.
    fn parameters(&self) -> Value;

    /// The actual internal execution logic.
    async fn call(
        &self,
        arguments: Option<Value>,
        context: &NativeToolCallContext,
    ) -> Result<String>;
}

/// Registry for native tools.
pub struct NativeToolRegistry {
    tools: HashMap<String, Arc<dyn NativeTool>>,
}

impl NativeToolRegistry {
    /// Creates a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Registers a new native tool.
    pub fn register(&mut self, tool: Arc<dyn NativeTool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Retrieves a tool by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<Arc<dyn NativeTool>> {
        self.tools.get(name).cloned()
    }

    /// Returns a list of tool definitions for the LLM.
    #[must_use]
    pub fn list_for_llm(&self) -> Vec<Value> {
        let mut tools = self
            .tools
            .values()
            .map(|t| {
                serde_json::json!({
                    "name": t.name(),
                    "description": t.description(),
                    "parameters": t.parameters(),
                })
            })
            .collect::<Vec<_>>();
        tools.sort_by(|left, right| extract_tool_name(left).cmp(extract_tool_name(right)));
        tools
    }

    /// Provides a textual summary of registered native tools for system prompt injection.
    #[must_use]
    pub fn get_registry_summary(&self) -> String {
        let mut summaries = self
            .tools
            .values()
            .map(|tool| format!("{}: {}", tool.name(), tool.description()))
            .collect::<Vec<_>>();
        summaries.sort_unstable();
        format!(
            "Native Core Tools available:\n- {}\nUse these in-process tools proactively when user intent matches.",
            summaries.join("\n- ")
        )
    }
}

fn extract_tool_name(tool: &Value) -> &str {
    tool.get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or("")
}

impl Default for NativeToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
