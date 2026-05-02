//! Tool-runtime request, response, and schema types.

use serde::{Deserialize, Serialize};

/// Result payload returned by the remote tool runtime `tools/list` operation.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ToolRuntimeListResult {
    /// Tool definitions returned by the remote runtime.
    pub tools: Vec<ToolRuntimeToolDefinition>,
}

/// Tool metadata advertised by a remote tool runtime.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolRuntimeToolDefinition {
    /// Stable tool name used for `tools/call`.
    pub name: String,
    /// Optional human-readable tool description.
    pub description: Option<String>,
    /// JSON Schema object describing accepted input arguments.
    pub input_schema: serde_json::Map<String, serde_json::Value>,
}

/// Normalized result payload returned by a remote tool call.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ToolRuntimeCallResult {
    /// Text content blocks emitted by the remote tool.
    pub text_segments: Vec<String>,
    /// Whether the remote runtime marked the tool call as an error.
    pub is_error: bool,
}

/// Pagination parameters forwarded to the remote tool runtime `tools/list` call.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ToolRuntimeListRequestParams {
    /// Opaque pagination cursor from a previous `tools/list` response.
    pub cursor: Option<String>,
}
