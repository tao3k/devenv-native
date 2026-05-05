//! Anthropic response and fallback transport data contracts.

use serde_json::Value;

/// Default environment variable used to resolve Anthropic credentials.
pub const DEFAULT_ANTHROPIC_KEY_ENV: &str = "ANTHROPIC_API_KEY";

/// Transport order entry for Anthropic custom-base fallback orchestration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnthropicCustomBaseTransport {
    /// OpenAI-compatible provider transport.
    OpenAi,
    /// `MiniMax` OpenAI-like transport.
    Minimax,
    /// Direct Anthropic `/v1/messages` bypass transport.
    AnthropicMessagesBypass,
}

/// Failed Anthropic custom-base fallback attempts.
#[derive(Debug)]
pub struct AnthropicCustomBaseFallbackFailure<E> {
    pub(crate) attempts: Vec<(AnthropicCustomBaseTransport, E)>,
}

impl<E> AnthropicCustomBaseFallbackFailure<E> {
    /// Access failed transport attempts in execution order.
    #[must_use]
    pub fn attempts(&self) -> &[(AnthropicCustomBaseTransport, E)] {
        self.attempts.as_slice()
    }

    /// Return last failure payload (if any).
    #[must_use]
    pub fn last_error(&self) -> Option<&E> {
        self.attempts.last().map(|(_transport, error)| error)
    }

    /// Consume and return all failed attempts.
    #[must_use]
    pub fn into_attempts(self) -> Vec<(AnthropicCustomBaseTransport, E)> {
        self.attempts
    }
}

/// Structured `tool_use` item decoded from Anthropic `messages` response.
#[derive(Debug, Clone, PartialEq)]
pub struct AnthropicToolUse {
    /// Tool call identifier.
    pub id: String,
    /// Tool name.
    pub name: String,
    /// Tool input payload.
    pub input: Value,
}

/// Parsed subset of Anthropic `messages` response content used by callers.
#[derive(Debug, Clone, PartialEq)]
pub struct AnthropicParsedResponse {
    /// Concatenated plain text segments.
    pub text: Option<String>,
    /// Structured tool calls.
    pub tool_uses: Vec<AnthropicToolUse>,
}
