//! Memory lifecycle contract and deterministic recall priors.
//!
//! The lifecycle model is the Rust-owned contract for deciding whether a
//! memory-like record is cache, temporary work, scheduled work, episodic
//! history, or long-term knowledge. Host adapters can project their native
//! metadata into this model without owning the policy.

use serde::{Deserialize, Serialize};

/// Lifecycle layer for one memory-like record.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryLayer {
    /// Rebuildable artifact or intermediate projection, never default recall.
    Cache,
    /// Active short-term task or session memory.
    Temporary,
    /// Time-bound memory with a schedule/deadline contract.
    Scheduled,
    /// Closed historical episode that can be recalled by scope or signature.
    Episodic,
    /// Promoted reusable knowledge with long-term recall eligibility.
    Knowledge,
}

impl MemoryLayer {
    /// Return the stable serialized layer name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cache => "cache",
            Self::Temporary => "temporary",
            Self::Scheduled => "scheduled",
            Self::Episodic => "episodic",
            Self::Knowledge => "knowledge",
        }
    }

    /// Deterministic prior used before semantic similarity, Q-value, or graph
    /// evidence is applied.
    #[must_use]
    pub const fn recall_prior(self) -> f32 {
        match self {
            Self::Cache => 0.0,
            Self::Temporary => 0.62,
            Self::Scheduled => 1.0,
            Self::Episodic => 0.58,
            Self::Knowledge => 0.94,
        }
    }
}

/// Lifecycle status for one memory-like record.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryStatus {
    /// Record is still being collected.
    Open,
    /// Record is active and can be recalled by default when otherwise eligible.
    Active,
    /// Record is closed history; it can still support scoped recall.
    Closed,
    /// Record has been replaced by a newer memory.
    Superseded,
    /// Record is intentionally archived out of normal recall.
    Archived,
    /// Record is time- or version-expired.
    Expired,
    /// Record is stale and must not be selected by default.
    Stale,
    /// Record was rejected as unsuitable memory.
    Rejected,
    /// Record is blocked by policy.
    Blocked,
}

impl MemoryStatus {
    /// Return the stable serialized status name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Active => "active",
            Self::Closed => "closed",
            Self::Superseded => "superseded",
            Self::Archived => "archived",
            Self::Expired => "expired",
            Self::Stale => "stale",
            Self::Rejected => "rejected",
            Self::Blocked => "blocked",
        }
    }

    /// Deterministic status multiplier for recall scoring.
    #[must_use]
    pub const fn recall_multiplier(self) -> f32 {
        match self {
            Self::Open => 0.72,
            Self::Active => 1.0,
            Self::Closed => 0.78,
            Self::Superseded
            | Self::Archived
            | Self::Expired
            | Self::Stale
            | Self::Rejected
            | Self::Blocked => 0.0,
        }
    }

    /// Return whether this status blocks memory-object projection.
    #[must_use]
    pub const fn blocks_projection(self) -> bool {
        matches!(
            self,
            Self::Superseded
                | Self::Archived
                | Self::Expired
                | Self::Stale
                | Self::Rejected
                | Self::Blocked
        )
    }
}

/// Default recall intent attached to one memory-like record.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryRecallDefault {
    /// Eligible for default recall when layer/status allow it.
    Yes,
    /// Not eligible for default recall but eligible when query scope matches.
    Scoped,
    /// Never enter normal recall packets.
    No,
}

impl MemoryRecallDefault {
    /// Return the stable serialized recall-default name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Yes => "yes",
            Self::Scoped => "scoped",
            Self::No => "no",
        }
    }

    /// Deterministic recall-default multiplier.
    #[must_use]
    pub const fn recall_multiplier(self) -> f32 {
        match self {
            Self::Yes => 1.0,
            Self::Scoped => 0.82,
            Self::No => 0.0,
        }
    }
}

/// Normalized lifecycle facts extracted from host metadata.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryLifecycleFacts {
    /// Lifecycle layer.
    pub layer: MemoryLayer,
    /// Lifecycle status.
    pub status: MemoryStatus,
    /// Default recall intent.
    pub recall_default: MemoryRecallDefault,
}

impl Default for MemoryLifecycleFacts {
    fn default() -> Self {
        Self {
            layer: MemoryLayer::Episodic,
            status: MemoryStatus::Active,
            recall_default: MemoryRecallDefault::Scoped,
        }
    }
}

impl MemoryLifecycleFacts {
    /// Evaluate recall and projection eligibility for these lifecycle facts.
    #[must_use]
    pub fn evaluate(self) -> MemoryLifecycleDecision {
        let projection_allowed = self.layer != MemoryLayer::Cache
            && !self.status.blocks_projection()
            && self.recall_default != MemoryRecallDefault::No;
        let scoped_recall_allowed = projection_allowed;
        let default_recall_allowed = scoped_recall_allowed
            && self.recall_default == MemoryRecallDefault::Yes
            && matches!(self.status, MemoryStatus::Open | MemoryStatus::Active);
        let recall_prior = if scoped_recall_allowed {
            (self.layer.recall_prior()
                * self.status.recall_multiplier()
                * self.recall_default.recall_multiplier())
            .clamp(0.0, 1.0)
        } else {
            0.0
        };
        MemoryLifecycleDecision {
            projection_allowed,
            default_recall_allowed,
            scoped_recall_allowed,
            recall_prior,
            reason_code: lifecycle_reason_code(self, projection_allowed, default_recall_allowed),
        }
    }
}

/// Deterministic lifecycle decision for projection and recall.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct MemoryLifecycleDecision {
    /// Whether adapters may materialize typed memory objects for this record.
    pub projection_allowed: bool,
    /// Whether the record can enter unprompted/default recall.
    pub default_recall_allowed: bool,
    /// Whether the record can enter explicit scoped recall.
    pub scoped_recall_allowed: bool,
    /// Deterministic prior to combine with semantic similarity, Q-value, and
    /// graph evidence.
    pub recall_prior: f32,
    /// Stable explanation code for audit and tests.
    pub reason_code: &'static str,
}

/// Infer lifecycle facts from structured properties.
#[must_use]
pub fn infer_memory_lifecycle_facts_from_properties<'a>(
    properties: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> MemoryLifecycleFacts {
    let mut facts = MemoryLifecycleFacts::default();
    for (key, value) in properties {
        match normalize_property_key(key).as_str() {
            "MEMORY_LAYER" | "MEMORY_TIER" => {
                if let Some(layer) = infer_memory_layer(value) {
                    facts.layer = layer;
                }
            }
            "MEMORY_STATUS" | "MEMORY_STATE" | "MEMORY_LIFECYCLE" | "MEMORY_OBJECT_STATUS" => {
                if let Some(status) = infer_memory_status(value) {
                    facts.status = status;
                }
            }
            "RECALL_DEFAULT" | "MEMORY_RECALL_DEFAULT" => {
                if let Some(recall_default) = infer_memory_recall_default(value) {
                    facts.recall_default = recall_default;
                }
            }
            _ => {}
        }
    }
    facts
}

/// Infer a lifecycle layer from host metadata text.
#[must_use]
pub fn infer_memory_layer(value: impl AsRef<str>) -> Option<MemoryLayer> {
    match normalize_property_value(value.as_ref()).as_str() {
        "cache" | "artifact-cache" | "projection-cache" => Some(MemoryLayer::Cache),
        "temporary" | "temp" | "working" | "working-memory" | "short-term" => {
            Some(MemoryLayer::Temporary)
        }
        "scheduled" | "schedule" | "time-bound" | "deadline" => Some(MemoryLayer::Scheduled),
        "episodic" | "episode" | "history" | "historical" => Some(MemoryLayer::Episodic),
        "knowledge"
        | "long-term"
        | "long-term-knowledge"
        | "durable-knowledge"
        | "working-knowledge" => Some(MemoryLayer::Knowledge),
        _ => None,
    }
}

/// Infer a lifecycle status from host metadata text.
#[must_use]
pub fn infer_memory_status(value: impl AsRef<str>) -> Option<MemoryStatus> {
    match normalize_property_value(value.as_ref()).as_str() {
        "open" | "draft" => Some(MemoryStatus::Open),
        "active" | "ready" => Some(MemoryStatus::Active),
        "closed" | "done" | "completed" => Some(MemoryStatus::Closed),
        "superseded" | "obsolete" => Some(MemoryStatus::Superseded),
        "archived" | "archive" => Some(MemoryStatus::Archived),
        "expired" | "outdated" => Some(MemoryStatus::Expired),
        "stale" => Some(MemoryStatus::Stale),
        "rejected" | "reject" => Some(MemoryStatus::Rejected),
        "blocked" | "block" => Some(MemoryStatus::Blocked),
        _ => None,
    }
}

/// Infer default recall intent from host metadata text.
#[must_use]
pub fn infer_memory_recall_default(value: impl AsRef<str>) -> Option<MemoryRecallDefault> {
    match normalize_property_value(value.as_ref()).as_str() {
        "yes" | "true" | "default" | "always" => Some(MemoryRecallDefault::Yes),
        "scoped" | "scope" | "query" | "explicit" => Some(MemoryRecallDefault::Scoped),
        "no" | "false" | "never" | "off" => Some(MemoryRecallDefault::No),
        _ => None,
    }
}

fn lifecycle_reason_code(
    facts: MemoryLifecycleFacts,
    projection_allowed: bool,
    default_recall_allowed: bool,
) -> &'static str {
    if facts.layer == MemoryLayer::Cache {
        return "cache_not_memory";
    }
    if facts.status.blocks_projection() {
        return "status_blocks_projection";
    }
    if facts.recall_default == MemoryRecallDefault::No {
        return "recall_default_no";
    }
    if default_recall_allowed {
        return "default_recall_allowed";
    }
    if projection_allowed {
        return "scoped_recall_only";
    }
    "not_recallable"
}

fn normalize_property_key(key: &str) -> String {
    key.trim()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
}

fn normalize_property_value(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
}
