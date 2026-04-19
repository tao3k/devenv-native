#[cfg(feature = "agent-provider-litellm")]
use litellm_rs::core::types::tools::{
    FunctionDefinition as LiteFunctionDefinition, Tool as LiteTool, ToolType as LiteToolType,
};

use super::types::{FunctionDef, ToolDef};

#[derive(Debug, Clone)]
pub(super) struct PreparedTool {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<serde_json::Value>,
}

impl PreparedTool {
    pub(super) fn to_http_tool_def(&self) -> ToolDef {
        ToolDef {
            typ: "function".to_string(),
            function: FunctionDef {
                name: self.name.clone(),
                description: self.description.clone(),
                parameters: self.parameters.clone(),
            },
        }
    }

    #[cfg(feature = "agent-provider-litellm")]
    pub(super) fn to_litellm_tool(&self) -> LiteTool {
        LiteTool {
            tool_type: LiteToolType::Function,
            function: LiteFunctionDefinition {
                name: self.name.clone(),
                description: self.description.clone(),
                parameters: self.parameters.clone(),
            },
        }
    }
}

pub(super) fn parse_tools_json(tools_json: Option<Vec<serde_json::Value>>) -> Vec<PreparedTool> {
    tools_json
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| {
            let name = v.get("name")?.as_str()?.trim().to_string();
            if name.is_empty() {
                return None;
            }
            let description = v
                .get("description")
                .and_then(|d| d.as_str())
                .map(str::trim)
                .filter(|d| !d.is_empty())
                .map(String::from);
            let parameters = v
                .get("input_schema")
                .cloned()
                .or_else(|| v.get("parameters").cloned());
            Some(PreparedTool {
                name,
                description,
                parameters,
            })
        })
        .collect()
}
