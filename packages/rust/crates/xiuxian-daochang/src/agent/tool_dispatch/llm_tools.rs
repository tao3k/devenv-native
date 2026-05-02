use std::collections::HashSet;

use anyhow::Result;

use super::execution::llm_tool_definitions;
use crate::Agent;

impl Agent {
    /// List all tools (Native + Zhenfa + external tool runtime) for the LLM.
    pub(in crate::agent) async fn tool_definitions_for_llm(
        &self,
    ) -> Result<Option<Vec<serde_json::Value>>> {
        let mut tools = self.native_tools.list_for_llm();
        let mut seen_tool_names = collect_seen_tool_names(&tools);

        if let Some(ref zhenfa_tools) = self.zhenfa_tools {
            for tool in zhenfa_tools.list_for_llm() {
                push_unique_tool(&mut tools, &mut seen_tool_names, tool);
            }
        }

        if let Some(ref tool_runtime) = self.tool_runtime {
            let list = tool_runtime.list_tools(None).await?;
            for tool in llm_tool_definitions(&list) {
                push_unique_tool(&mut tools, &mut seen_tool_names, tool);
            }
        }

        if tools.is_empty() {
            return Ok(None);
        }
        Ok(Some(tools))
    }
}

fn collect_seen_tool_names(tools: &[serde_json::Value]) -> HashSet<String> {
    tools
        .iter()
        .filter_map(extract_tool_name)
        .map(ToString::to_string)
        .collect()
}

fn push_unique_tool(
    tools: &mut Vec<serde_json::Value>,
    seen_tool_names: &mut HashSet<String>,
    tool: serde_json::Value,
) {
    if let Some(name) = extract_tool_name(&tool)
        && !seen_tool_names.insert(name.to_string())
    {
        return;
    }
    tools.push(tool);
}

fn extract_tool_name(tool: &serde_json::Value) -> Option<&str> {
    tool.get("name")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
}
