//! Context-block records for prompt-injection snapshots.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Stable block identifier for prompt context replay/audit.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PromptContextBlockId(String);

impl PromptContextBlockId {
    /// Creates a prompt context block identifier.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for PromptContextBlockId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PromptContextBlockId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl From<PromptContextBlockId> for String {
    fn from(value: PromptContextBlockId) -> Self {
        value.0
    }
}

impl PartialEq<&str> for PromptContextBlockId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

/// Scope identifier that binds a block to a session or channel.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PromptSessionScope(String);

impl PromptSessionScope {
    /// Creates a prompt session scope.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for PromptSessionScope {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PromptSessionScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Source domain that produced a context block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptContextSource {
    /// Retrieved from short-term memory recall.
    MemoryRecall,
    /// From session-level XML/system prompt injection history.
    SessionXml,
    /// Condensed summary from context window manager.
    WindowSummary,
    /// Retrieved from durable knowledge.
    Knowledge,
    /// Reflection artifacts from previous turns.
    Reflection,
    /// Runtime-generated execution hints.
    RuntimeHint,
    /// Governance/policy directives.
    Policy,
}

/// Category used by policy-level budget and ordering rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptContextCategory {
    /// Safety-critical guidance.
    Safety,
    /// Governance/policy guidance.
    Policy,
    /// Memory recall content.
    MemoryRecall,
    /// Session XML content.
    SessionXml,
    /// Window summary content.
    WindowSummary,
    /// Durable knowledge content.
    Knowledge,
    /// Reflection content.
    Reflection,
    /// Runtime hint content.
    RuntimeHint,
}

/// Immutable context block in a typed injection snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptContextBlock {
    /// Stable identifier for audit/replay.
    pub block_id: PromptContextBlockId,
    /// Producer source.
    pub source: PromptContextSource,
    /// Policy category.
    pub category: PromptContextCategory,
    /// Higher value means higher priority.
    pub priority: u16,
    /// Scope identifier, usually a session key.
    pub session_scope: PromptSessionScope,
    /// Rendered payload text/XML.
    pub payload: String,
    /// Character count of payload at snapshot time.
    pub payload_chars: usize,
    /// Whether this block is non-evictable.
    pub anchor: bool,
}

/// Named request for constructing a prompt context block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptContextBlockInput {
    /// Stable identifier for audit/replay.
    pub block_id: PromptContextBlockId,
    /// Producer source.
    pub source: PromptContextSource,
    /// Policy category.
    pub category: PromptContextCategory,
    /// Higher value means higher priority.
    pub priority: u16,
    /// Scope identifier, usually a session key.
    pub session_scope: PromptSessionScope,
    /// Rendered payload text/XML.
    pub payload: String,
    /// Whether this block is non-evictable.
    pub anchor: bool,
}

impl PromptContextBlock {
    /// Construct a block and compute `payload_chars` from payload text.
    #[must_use]
    pub fn new(input: PromptContextBlockInput) -> Self {
        let payload = input.payload;
        Self {
            block_id: input.block_id,
            source: input.source,
            category: input.category,
            priority: input.priority,
            session_scope: input.session_scope,
            payload_chars: payload.chars().count(),
            payload,
            anchor: input.anchor,
        }
    }
}
