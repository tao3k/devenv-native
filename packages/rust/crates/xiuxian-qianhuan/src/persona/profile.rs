//! Persona profile records for prompt voice and reasoning style.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Profile defining an AI persona's voice, constraints and reasoning style.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaProfile {
    /// Unique identifier for the persona.
    pub id: String,
    /// Friendly display name.
    pub name: String,
    /// Detailed description of the voice and tone.
    pub voice_tone: String,
    /// Detailed background or system instructions for this specific persona.
    #[serde(default)]
    pub background: Option<String>,
    /// Explicit rules and behavioral guidelines the persona MUST follow.
    #[serde(default)]
    pub guidelines: Vec<String>,
    /// Keywords or anchors that must be present in the grounding context.
    pub style_anchors: Vec<String>,
    /// Template used for Chain-of-Thought reasoning.
    pub cot_template: String,
    /// List of phrases the persona is forbidden to use.
    pub forbidden_words: Vec<String>,
    /// Optional metadata for extended persona traits.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}
