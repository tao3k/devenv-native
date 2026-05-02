//! Serializable event payload model and constructors.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

/// Core event model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmniEvent {
    /// Unique event identifier.
    pub id: String,
    /// Event source (e.g., "watcher", "tool:filesystem", "kernel").
    pub source: String,
    /// Event topic/category (e.g., "file/changed", "agent/thought").
    pub topic: String,
    /// Flexible JSON payload.
    pub payload: Value,
    /// Event timestamp.
    pub timestamp: DateTime<Utc>,
}

impl OmniEvent {
    /// Create a new event.
    #[must_use]
    pub fn new(source: impl Into<String>, topic: impl Into<String>, payload: Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source: source.into(),
            topic: topic.into(),
            payload,
            timestamp: Utc::now(),
        }
    }

    /// Create a simple string payload event.
    #[must_use]
    pub fn with_string(source: &str, topic: &str, message: &str) -> Self {
        Self::new(source, topic, json!({ "message": message }))
    }

    /// Create a file-related event.
    #[must_use]
    pub fn file_event(source: &str, topic: &str, path: &str, is_dir: bool) -> Self {
        Self::new(source, topic, json!({ "path": path, "is_dir": is_dir }))
    }
}

impl std::fmt::Display for OmniEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} -> {}: {}",
            self.timestamp.format("%H:%M:%S"),
            self.source,
            self.topic,
            self.payload
        )
    }
}
