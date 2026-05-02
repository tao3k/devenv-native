use crate::tool_runtime::{
    ToolRuntimeCallResult, ToolRuntimeListResult, ToolRuntimeToolDefinition,
};
use anyhow::{Result, anyhow};

use super::{
    ToolCallExecution, execute_call_with_timeout, llm_tool_definitions, timeout_tool_error_payload,
};

#[test]
fn timeout_payload_marks_degraded_error_shape() -> Result<()> {
    let payload = timeout_tool_error_payload("tool_runtime", "memory.search_memory", 3);
    let value: serde_json::Value = serde_json::from_str(&payload)?;
    assert_eq!(value["ok"], false);
    assert_eq!(value["degraded"], true);
    assert_eq!(value["source"], "tool_runtime");
    assert_eq!(value["error_kind"], "timeout");
    assert_eq!(value["timeout_secs"], 3);
    Ok(())
}

#[test]
fn llm_tool_definitions_maps_description_and_schema() {
    let tool = ToolRuntimeToolDefinition {
        name: "mock.echo".to_string(),
        description: Some("Echo tool".to_string()),
        input_schema: serde_json::Map::from_iter([(
            "type".to_string(),
            serde_json::Value::String("object".to_string()),
        )]),
    };
    let list = ToolRuntimeListResult { tools: vec![tool] };
    let values = llm_tool_definitions(&list);
    assert_eq!(values.len(), 1);
    assert_eq!(values[0]["name"], "mock.echo");
    assert_eq!(values[0]["description"], "Echo tool");
    assert_eq!(values[0]["parameters"]["type"], "object");
}

#[tokio::test]
async fn execute_call_with_timeout_classifies_timeout_like_errors() {
    let result = execute_call_with_timeout(
        || async { Err(anyhow!("request timed out while waiting for upstream")) },
        1,
    )
    .await;
    match result {
        ToolCallExecution::Timeout { detail } => {
            assert!(detail.is_some());
        }
        _ => panic!("expected timeout classification"),
    }
}

#[tokio::test]
async fn execute_call_with_timeout_decodes_text_results() {
    let result = execute_call_with_timeout(
        || async {
            Ok(ToolRuntimeCallResult {
                text_segments: vec!["hello".to_string()],
                is_error: false,
            })
        },
        1,
    )
    .await;
    match result {
        ToolCallExecution::Completed(output) => {
            assert_eq!(output.text, "hello");
            assert!(!output.is_error);
        }
        _ => panic!("expected completed result"),
    }
}
