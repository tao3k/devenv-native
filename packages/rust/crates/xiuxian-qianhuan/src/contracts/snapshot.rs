//! Snapshot records produced by prompt-injection policy evaluation.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{InjectionPolicy, PromptContextBlock, RoleMixProfile};

/// Stable snapshot identifier for injection replay/audit.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InjectionSnapshotId(String);

impl InjectionSnapshotId {
    /// Creates an injection snapshot identifier.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for InjectionSnapshotId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InjectionSnapshotId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl PartialEq<&str> for InjectionSnapshotId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

/// Session identifier associated with an injection snapshot.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InjectionSessionId(String);

impl InjectionSessionId {
    /// Creates an injection session identifier.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for InjectionSessionId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InjectionSessionId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Turn sequence identifier associated with an injection snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InjectionTurnId(u64);

impl InjectionTurnId {
    /// Creates an injection turn identifier.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw turn sequence value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

impl PartialEq<u64> for InjectionTurnId {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

impl From<InjectionTurnId> for u64 {
    fn from(value: InjectionTurnId) -> Self {
        value.0
    }
}

/// Immutable turn-level injection snapshot consumed by execution runtime.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InjectionSnapshot {
    /// Snapshot identifier for replay/audit.
    pub snapshot_id: InjectionSnapshotId,
    /// Session identifier.
    pub session_id: InjectionSessionId,
    /// Turn sequence number in this session.
    pub turn_id: InjectionTurnId,
    /// Policy used to produce this snapshot.
    pub policy: InjectionPolicy,
    /// Selected role-mix profile, if any.
    pub role_mix: Option<RoleMixProfile>,
    /// Retained blocks in final snapshot.
    pub blocks: Vec<PromptContextBlock>,
    /// Aggregate chars across retained blocks.
    pub total_chars: usize,
    /// Block IDs dropped by budget policy.
    pub dropped_block_ids: Vec<String>,
    /// Block IDs truncated by budget policy.
    pub truncated_block_ids: Vec<String>,
}

/// Named request for building an injection snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct InjectionSnapshotInput {
    /// Snapshot identifier for replay/audit.
    pub snapshot_id: InjectionSnapshotId,
    /// Session identifier.
    pub session_id: InjectionSessionId,
    /// Turn sequence number in this session.
    pub turn_id: InjectionTurnId,
    /// Policy used to produce this snapshot.
    pub policy: InjectionPolicy,
    /// Selected role-mix profile, if any.
    pub role_mix: Option<RoleMixProfile>,
    /// Retained blocks in final snapshot.
    pub blocks: Vec<PromptContextBlock>,
}

impl InjectionSnapshot {
    /// Build a snapshot and compute `total_chars` from blocks.
    #[must_use]
    pub fn from_blocks(input: InjectionSnapshotInput) -> Self {
        let total_chars = input.blocks.iter().map(|block| block.payload_chars).sum();
        Self {
            snapshot_id: input.snapshot_id,
            session_id: input.session_id,
            turn_id: input.turn_id,
            policy: input.policy,
            role_mix: input.role_mix,
            blocks: input.blocks,
            total_chars,
            dropped_block_ids: Vec::new(),
            truncated_block_ids: Vec::new(),
        }
    }

    /// Validate key contract invariants for this snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error when `total_chars` does not match the retained blocks,
    /// or when the snapshot exceeds the configured block or character budgets.
    pub fn validate(&self) -> Result<(), String> {
        let computed_chars: usize = self.blocks.iter().map(|block| block.payload_chars).sum();
        if computed_chars != self.total_chars {
            return Err(format!(
                "injection snapshot total_chars mismatch: computed={computed_chars} stored={}",
                self.total_chars
            ));
        }
        if self.blocks.len() > self.policy.max_blocks {
            return Err(format!(
                "injection snapshot exceeds max_blocks: blocks={} max_blocks={}",
                self.blocks.len(),
                self.policy.max_blocks
            ));
        }
        if self.total_chars > self.policy.max_chars {
            return Err(format!(
                "injection snapshot exceeds max_chars: total_chars={} max_chars={}",
                self.total_chars, self.policy.max_chars
            ));
        }
        Ok(())
    }
}
