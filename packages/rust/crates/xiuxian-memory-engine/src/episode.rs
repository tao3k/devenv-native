//! Episode data structures for self-evolving memory.
//!
//! An Episode represents a single interaction experience in the memory system,
//! storing intent, experience, outcome, and Q-learning metadata.

use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Global scope constant for episodes without a specific scope.
pub const GLOBAL_EPISODE_SCOPE: &str = "_global";

/// Stable episode identifier used by memory APIs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EpisodeId(String);

impl EpisodeId {
    /// Borrows the raw episode identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for EpisodeId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl From<String> for EpisodeId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for EpisodeId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<&String> for EpisodeId {
    fn from(value: &String) -> Self {
        Self(value.clone())
    }
}

impl PartialEq<&str> for EpisodeId {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Input payload for creating an episode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpisodeDraft {
    /// Unique identifier for this episode.
    pub id: EpisodeId,
    /// User intent captured for the episode.
    pub intent: String,
    /// Semantic embedding for the intent.
    pub intent_embedding: Vec<f32>,
    /// Stored experience or action result.
    pub experience: String,
    /// Outcome label or detail.
    pub outcome: String,
    /// Optional logical scope.
    #[serde(default)]
    pub scope: Option<String>,
}

impl EpisodeDraft {
    /// Assign a logical scope to this draft.
    #[must_use]
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }
}

/// A single experience episode in the memory system.
///
/// Each episode represents a stored interaction with:
/// - Intent (what the user wanted)
/// - Experience (the actual experience/response)
/// - Outcome (success/failure result)
/// - Q-value (learned utility from Q-learning)
/// - Usage statistics (retrieval and feedback counts)
/// - Scope (logical grouping for recall)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    /// Unique identifier for this episode
    pub id: String,
    /// The user's intent (query/goal)
    pub intent: String,
    /// Semantic embedding of the intent
    pub intent_embedding: Vec<f32>,
    /// The actual experience (response/action taken)
    pub experience: String,
    /// The outcome (success indicator, error message, etc.)
    pub outcome: String,
    /// Current Q-value (learned utility, initialized to 0.5)
    pub q_value: f32,
    /// Number of times the episode was retrieved or used.
    #[serde(default)]
    pub retrieval_count: u32,
    /// Number of successful retrievals
    pub success_count: u32,
    /// Number of failed retrievals
    pub failure_count: u32,
    /// Creation timestamp (Unix milliseconds)
    pub created_at: i64,
    /// Last update timestamp (Unix milliseconds)
    #[serde(default)]
    pub updated_at: i64,
    /// Logical scope for grouping episodes
    pub scope: String,
}

impl Episode {
    /// Create a new episode with default Q-value (0.5).
    #[must_use]
    pub fn new(draft: EpisodeDraft) -> Self {
        Self::from_draft(draft)
    }

    /// Create a new episode with a specific scope.
    #[must_use]
    pub fn new_scoped(draft: EpisodeDraft) -> Self {
        Self::from_draft(draft)
    }

    fn from_draft(draft: EpisodeDraft) -> Self {
        let now = Utc::now().timestamp_millis();
        Self {
            id: draft.id.as_str().to_string(),
            intent: draft.intent,
            intent_embedding: draft.intent_embedding,
            experience: draft.experience,
            outcome: draft.outcome,
            q_value: 0.5, // Initial Q-value (neutral)
            retrieval_count: 0,
            success_count: 0,
            failure_count: 0,
            created_at: now,
            updated_at: now,
            scope: draft
                .scope
                .as_deref()
                .map_or_else(|| GLOBAL_EPISODE_SCOPE.to_string(), Self::normalize_scope),
        }
    }

    fn touch(&mut self) {
        self.updated_at = Utc::now().timestamp_millis();
    }

    /// Normalize derived episode fields after deserialization or migration.
    pub fn normalize_tracking_fields(&mut self) {
        if self.updated_at == 0 {
            self.updated_at = self.created_at;
        }
        let feedback_count = self.feedback_count();
        if self.retrieval_count < feedback_count {
            self.retrieval_count = feedback_count;
        }
    }

    /// Normalize a scope string.
    ///
    /// Returns `GLOBAL_EPISODE_SCOPE` for empty or whitespace-only strings.
    #[must_use]
    pub fn normalize_scope(scope: &str) -> String {
        let trimmed = scope.trim();
        if trimmed.is_empty() {
            GLOBAL_EPISODE_SCOPE.to_string()
        } else {
            trimmed.to_string()
        }
    }

    /// Get the normalized scope key for this episode.
    #[must_use]
    pub fn scope_key(&self) -> &str {
        self.scope.trim()
    }

    /// Calculate the utility of this episode.
    ///
    /// Utility is computed as: `success_rate * q_value`
    /// - `success_rate = success / (success + failure + 1)` to avoid division by zero
    /// - This gives higher weight to episodes with more successes
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn utility(&self) -> f32 {
        let total = self.feedback_count() as f32 + 1.0;
        let success_rate = (self.success_count as f32 + 1.0) / total;
        success_rate * self.q_value
    }

    /// Get the total number of feedback outcomes.
    #[must_use]
    pub fn feedback_count(&self) -> u32 {
        self.success_count + self.failure_count
    }

    /// Update success count and recalculate Q-value.
    pub fn mark_success(&mut self) {
        self.retrieval_count = self.retrieval_count.saturating_add(1);
        self.success_count = self.success_count.saturating_add(1);
        self.touch();
    }

    /// Update failure count and recalculate Q-value.
    pub fn mark_failure(&mut self) {
        self.retrieval_count = self.retrieval_count.saturating_add(1);
        self.failure_count = self.failure_count.saturating_add(1);
        self.touch();
    }

    /// Mark the episode as retrieved without assigning outcome feedback.
    pub fn mark_accessed(&mut self) {
        self.retrieval_count = self.retrieval_count.saturating_add(1);
        self.touch();
    }

    /// Get the total number of uses.
    #[must_use]
    pub fn total_uses(&self) -> u32 {
        self.retrieval_count
    }

    /// Check if this episode has been validated (used at least once).
    #[must_use]
    pub fn is_validated(&self) -> bool {
        self.feedback_count() > 0
    }

    /// Apply time-based decay to the Q-value.
    ///
    /// `Q_decay = Q * decay_factor^(age_hours)`.
    ///
    /// Args:
    /// - `decay_factor`: Decay per hour (e.g., 0.95 means 5% decay per hour)
    /// - `current_time`: Current timestamp in milliseconds
    ///
    /// Returns:
    /// - Decayed Q-value (moves towards 0.5 over time)
    #[allow(clippy::cast_precision_loss)]
    pub fn apply_time_decay(&mut self, decay_factor: f32, current_time: i64) {
        let age_hours = (current_time - self.created_at) as f32 / (1000.0 * 60.0 * 60.0);
        if age_hours > 0.0 {
            let decay = decay_factor.powf(age_hours);
            // Decay towards 0.5 (neutral value)
            self.q_value = 0.5 + (self.q_value - 0.5) * decay;
        }
    }

    /// Get the age of this episode in hours.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn age_hours(&self, current_time: i64) -> f32 {
        (current_time - self.created_at) as f32 / (1000.0 * 60.0 * 60.0)
    }
}
