//! Minimal chat data types used when provider execution is disabled.

/// Minimal chat request type available when provider execution is disabled.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChatRequest {
    /// Provider model id.
    pub model: String,
    /// Chat messages.
    pub messages: Vec<ChatMessage>,
}

/// Minimal chat message type available when provider execution is disabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    /// Message role.
    pub role: MessageRole,
    /// Message content.
    pub content: Option<MessageContent>,
    /// Function call payload placeholder.
    pub function_call: Option<serde_json::Value>,
    /// Optional sender name.
    pub name: Option<String>,
    /// Tool call id.
    pub tool_call_id: Option<String>,
    /// Tool call payload placeholder.
    pub tool_calls: Option<Vec<serde_json::Value>>,
    /// Thinking payload placeholder.
    pub thinking: Option<serde_json::Value>,
    /// Audio payload placeholder.
    pub audio: Option<serde_json::Value>,
}

impl Default for ChatMessage {
    fn default() -> Self {
        Self {
            role: MessageRole::User,
            content: None,
            function_call: None,
            name: None,
            tool_call_id: None,
            tool_calls: None,
            thinking: None,
            audio: None,
        }
    }
}

/// Minimal message role type available when provider execution is disabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageRole {
    /// System message.
    System,
    /// Developer message.
    Developer,
    /// User message.
    User,
    /// Assistant message.
    Assistant,
    /// Tool message.
    Tool,
}

/// Minimal message content type available when provider execution is disabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageContent {
    /// Plain text content.
    Text(String),
    /// Multimodal content parts.
    Parts(Vec<ContentPart>),
}

/// Minimal multimodal content part type available when provider execution is disabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentPart {
    /// Text part.
    Text {
        /// Text payload.
        text: String,
    },
    /// Image URL part.
    ImageUrl {
        /// Image URL payload.
        image_url: ImageUrlContent,
    },
}

/// Minimal image URL content type available when provider execution is disabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageUrlContent {
    /// URL or data URI.
    pub url: String,
    /// Image detail hint.
    pub detail: Option<String>,
}

/// Minimal chat choice type available when provider execution is disabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatChoice {
    /// Choice index.
    pub index: usize,
    /// Assistant message.
    pub message: ChatMessage,
}

/// Minimal chat response type available when provider execution is disabled.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChatResponse {
    /// Response id.
    pub id: String,
    /// Response choices.
    pub choices: Vec<ChatChoice>,
}
