//! Core types for MemRL state-action space.

use serde::{Deserialize, Serialize};

/// Discrete representation of the Agent's environment state.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryState {
    /// Level of information missingness (0-5, derived from CCS).
    pub context_entropy: u8,
    /// Active persona identifier hash.
    pub persona_hash: u64,
    /// Type of task being performed.
    pub task_kind: MemoryTaskKind,
}

/// Typed memory task kind identifier.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MemoryTaskKind(String);

impl MemoryTaskKind {
    /// Create a task kind from a string-like value.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the task kind as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for MemoryTaskKind {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for MemoryTaskKind {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl PartialEq<&str> for MemoryTaskKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Actions the memory system can take on a specific memory segment.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MemoryAction {
    /// Keep the memory in episodic storage.
    Retain,
    /// Completely remove the memory to save context budget.
    Purge,
    /// Move to working memory (high-priority injection).
    Promote,
}

impl MemoryAction {
    /// Iterator over all possible actions for Q-Value maximization.
    #[must_use]
    pub fn all() -> Vec<Self> {
        vec![Self::Retain, Self::Purge, Self::Promote]
    }
}
