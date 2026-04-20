use serde::{Deserialize, Serialize};

/// Execution mode for per-node Qianhuan annotation bindings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum NodeQianhuanExecutionMode {
    /// Use an isolated ephemeral injection window for each node execution.
    ///
    /// This is the default mode for multi-persona adversarial loops to avoid
    /// context contamination across nodes.
    #[default]
    Isolated,
    /// Reuse and append to a continuous history window via a context key.
    ///
    /// Use this mode only for same-persona multi-step tool execution chains.
    Appended,
}

impl NodeQianhuanExecutionMode {
    /// Returns the stable string representation used in telemetry payloads.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Isolated => "isolated",
            Self::Appended => "appended",
        }
    }
}

/// Qianhuan binding metadata attached to a node.
///
/// This formalizes Phase E of the Qianji-Qianhuan interface in TOML:
/// `[[nodes]] ... [nodes.qianhuan]`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub struct NodeQianhuanBinding {
    /// Persona profile identifier resolved by `PersonaRegistry`.
    pub persona_id: Option<String>,
    /// Logical template target consumed by manifestation/runtime layers.
    pub template_target: Option<String>,
    /// Execution-mode selector for context window behavior.
    #[serde(default)]
    pub execution_mode: NodeQianhuanExecutionMode,
    /// Whitelisted context keys that can be marshaled into this node's
    /// annotation narrative blocks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_keys: Vec<String>,
    /// Context key used for appended mode history persistence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_key: Option<String>,
    /// Output context key that stores the generated annotation snapshot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_key: Option<String>,
}

/// LLM tenant binding metadata attached to a node.
///
/// This enables node-scoped provider/model selection in TOML:
/// `[[nodes]] ... [nodes.llm]`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub struct NodeLlmBinding {
    /// Optional backend/provider identifier (for example `openai`, `litellm_rs`).
    pub provider: Option<String>,
    /// Optional model override for this node.
    pub model: Option<String>,
    /// Optional OpenAI-compatible base URL override for this node.
    pub base_url: Option<String>,
    /// Optional environment variable name containing API key for this node.
    pub api_key_env: Option<String>,
}
