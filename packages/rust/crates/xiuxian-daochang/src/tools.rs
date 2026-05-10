//! Tool name qualification for multiple external tool servers.

/// Format: `tool__{server}__{tool}` so the agent can route tool calls to the right external tool server.
#[must_use]
pub fn qualify_tool_name(server: &str, tool: &str) -> String {
    format!("tool__{server}__{tool}")
}

/// Parsed qualified tool name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualifiedToolName {
    /// External tool server id.
    pub server: String,
    /// Tool name within the external server.
    pub tool: String,
}

/// Parse a qualified name.
#[must_use]
pub fn parse_qualified_tool_name(qualified: &str) -> Option<QualifiedToolName> {
    let rest = qualified.strip_prefix("tool__")?;
    let (server, tool) = rest.split_once("__")?;
    if server.is_empty() || tool.is_empty() {
        return None;
    }
    Some(QualifiedToolName {
        server: server.to_string(),
        tool: tool.to_string(),
    })
}
