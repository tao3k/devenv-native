//! Builder module for extracting injection blocks from chat messages.

use crate::session::ChatMessage;

/// Represents an extracted injection block.
#[derive(Debug, Clone)]
pub struct InjectionBlock {
    pub block_id: String,
    pub content: String,
}

/// Result of extracting blocks from messages.
pub struct ExtractionResult {
    pub blocks: Vec<InjectionBlock>,
    pub passthrough_messages: Vec<ChatMessage>,
}

/// Extract injection blocks from chat messages.
pub(crate) fn extract_blocks(
    _session_id: &str,
    _turn_id: u64,
    messages: Vec<ChatMessage>,
) -> ExtractionResult {
    // Placeholder implementation - pass through all messages without extraction
    ExtractionResult {
        blocks: Vec::new(),
        passthrough_messages: messages,
    }
}
