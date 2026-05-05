use std::future::Future;
use std::time::Duration;

use crate::tool_runtime::{
    ToolRuntimeCallResult, ToolRuntimeListResult, ToolRuntimeToolDefinition,
};
use anyhow::Error;

pub(super) struct ToolCallExecutionOutput {
    pub(super) text: String,
    pub(super) is_error: bool,
}

pub(super) enum ToolCallExecution {
    Completed(ToolCallExecutionOutput),
    Timeout { detail: Option<String> },
    TransportError(Error),
}

pub(super) async fn execute_call_with_timeout<F, Fut>(
    operation: F,
    timeout_secs: u64,
) -> ToolCallExecution
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = anyhow::Result<ToolRuntimeCallResult>>,
{
    let timeout_secs = timeout_secs.max(1);
    let call_result = tokio::time::timeout(Duration::from_secs(timeout_secs), operation()).await;

    match call_result {
        Err(_) => ToolCallExecution::Timeout { detail: None },
        Ok(Err(error)) => {
            let message = format!("{error:#}");
            if is_timeout_error_message(&message) {
                ToolCallExecution::Timeout {
                    detail: Some(message),
                }
            } else {
                ToolCallExecution::TransportError(error)
            }
        }
        Ok(Ok(result)) => ToolCallExecution::Completed(decode_call_tool_result(&result)),
    }
}

pub(super) fn is_timeout_error_message(message: &str) -> bool {
    let lowercase = message.to_ascii_lowercase();
    lowercase.contains("timed out")
        || lowercase.contains("timeout")
        || lowercase.contains("tool_runtime.pool.call.waiting")
}

pub(super) fn degraded_tool_error_payload(
    tool: &str,
    source: Option<&str>,
    error_kind: &str,
    timeout_secs: Option<u64>,
    message: &str,
) -> String {
    let mut payload = serde_json::Map::new();
    payload.insert("ok".to_string(), serde_json::Value::Bool(false));
    payload.insert("degraded".to_string(), serde_json::Value::Bool(true));
    payload.insert(
        "tool".to_string(),
        serde_json::Value::String(tool.to_string()),
    );
    payload.insert(
        "error_kind".to_string(),
        serde_json::Value::String(error_kind.to_string()),
    );
    payload.insert(
        "message".to_string(),
        serde_json::Value::String(message.to_string()),
    );
    if let Some(source) = source {
        payload.insert(
            "source".to_string(),
            serde_json::Value::String(source.to_string()),
        );
    }
    if let Some(timeout_secs) = timeout_secs {
        payload.insert(
            "timeout_secs".to_string(),
            serde_json::Value::Number(serde_json::Number::from(timeout_secs)),
        );
    }
    serde_json::Value::Object(payload).to_string()
}

pub(super) fn timeout_tool_error_payload(source: &str, tool: &str, timeout_secs: u64) -> String {
    let message = format!(
        "Tool `{tool}` from {source} timed out after {timeout_secs}s and was aborted; continuing without tool result."
    );
    degraded_tool_error_payload(tool, Some(source), "timeout", Some(timeout_secs), &message)
}

pub(super) fn llm_tool_definitions(list: &ToolRuntimeListResult) -> Vec<serde_json::Value> {
    list.tools.iter().map(llm_tool_definition).collect()
}

fn llm_tool_definition(tool: &ToolRuntimeToolDefinition) -> serde_json::Value {
    let mut object = serde_json::Map::new();
    object.insert(
        "name".to_string(),
        serde_json::Value::String(tool.name.clone()),
    );
    if let Some(description) = &tool.description {
        object.insert(
            "description".to_string(),
            serde_json::Value::String(description.clone()),
        );
    }
    object.insert(
        "parameters".to_string(),
        serde_json::Value::Object(tool.input_schema.clone()),
    );
    serde_json::Value::Object(object)
}

fn decode_call_tool_result(result: &ToolRuntimeCallResult) -> ToolCallExecutionOutput {
    ToolCallExecutionOutput {
        text: result.text_segments.concat(),
        is_error: result.is_error,
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/agent/tool_dispatch/helpers.rs"]
mod tests;
