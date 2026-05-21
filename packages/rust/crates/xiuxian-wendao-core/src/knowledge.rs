//! Core knowledge-entry records used by Wendao storage and retrieval.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use xiuxian_types::KnowledgeCategory;

/// Stable identifier for one knowledge entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeEntryId(String);

impl KnowledgeEntryId {
    /// Consumes this id into its serialized representation.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for KnowledgeEntryId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for KnowledgeEntryId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// Stable knowledge payload shared across Wendao consumers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KnowledgeEntry {
    /// Unique identifier for the entry.
    pub id: String,
    /// Human-readable title.
    pub title: String,
    /// Main content/body of the knowledge entry.
    pub content: String,
    /// Classification category.
    pub category: KnowledgeCategory,
    /// Tags for filtering and search.
    pub tags: Vec<String>,
    /// Original source file path or URL.
    pub source: Option<String>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp.
    pub updated_at: DateTime<Utc>,
    /// Entry version for change tracking.
    pub version: i32,
    /// Additional metadata for extensibility.
    pub metadata: HashMap<String, serde_json::Value>,
}

impl KnowledgeEntry {
    /// Create a new `KnowledgeEntry` with required fields.
    #[must_use]
    pub fn new(
        id: impl Into<KnowledgeEntryId>,
        title: String,
        content: String,
        category: KnowledgeCategory,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: id.into().into_string(),
            title,
            content,
            category,
            tags: Vec::new(),
            source: None,
            created_at: now,
            updated_at: now,
            version: 1,
            metadata: HashMap::new(),
        }
    }

    /// Set tags for this entry.
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set source for this entry.
    #[must_use]
    pub fn with_source(mut self, source: Option<String>) -> Self {
        self.source = source;
        self
    }

    /// Add a tag to this entry.
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}
