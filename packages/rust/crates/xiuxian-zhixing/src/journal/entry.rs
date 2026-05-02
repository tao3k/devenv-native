//! Journal entry record for stream-of-consciousness reflections.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A daily journal entry for Xiuxian-Zhixing.
///
/// Captures the "Stream of Consciousness" for later alchemy/structuring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Unique identifier for the journal entry.
    pub id: Uuid,
    /// Exact time when this entry was recorded.
    pub timestamp: DateTime<Utc>,
    /// Raw unstructured text from the user.
    pub content: String,
    /// Tags for categorizing reflection (e.g., "Meditation", "Insight").
    pub tags: Vec<String>,
    /// Optional state snapshots (e.g., mental state, current activity).
    pub state_snapshot: serde_json::Value,
    /// Whether this entry has been processed by the LLM into the Knowledge Graph.
    pub processed: bool,
    /// Related entity IDs in the Knowledge Graph (Wendao).
    pub related_entities: Vec<String>,
}

impl JournalEntry {
    /// Creates a new journal entry with the current timestamp.
    #[must_use]
    pub fn new(content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            content,
            tags: Vec::new(),
            state_snapshot: serde_json::Value::Null,
            processed: false,
            related_entities: Vec::new(),
        }
    }
}
