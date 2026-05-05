//! Manifestation render request and runtime context models.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Logical template target resolved by Qianhuan manifestation layer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ManifestationTemplateTarget {
    /// Agenda rendering target.
    DailyAgenda,
    /// XML system prompt target.
    SystemPromptV2Xml,
    /// Custom template file name.
    Custom(String),
}

impl ManifestationTemplateTarget {
    /// Returns the template file name mapped to this logical target.
    #[must_use]
    pub fn template_name(&self) -> &str {
        match self {
            Self::DailyAgenda => "daily_agenda.md",
            Self::SystemPromptV2Xml => "system_prompt_v2.xml",
            Self::Custom(name) => name.as_str(),
        }
    }
}

/// Runtime context used to enrich template payloads.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ManifestationRuntimeContext {
    /// High-level state context used by context injection heuristics.
    pub state_context: Option<String>,
    /// Optional active persona identifier.
    pub persona_id: Option<String>,
    /// Optional active task domain.
    pub domain: Option<String>,
    /// Arbitrary extension fields for template payload.
    #[serde(default)]
    pub extra: HashMap<String, Value>,
}

/// Render request for manifestation manager.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ManifestationRenderRequest {
    /// Template target selector.
    pub target: ManifestationTemplateTarget,
    /// Primary payload data for template rendering.
    pub data: Value,
    /// Runtime context for dynamic injection.
    #[serde(default)]
    pub runtime: ManifestationRuntimeContext,
}
