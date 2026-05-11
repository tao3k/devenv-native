//! Frontmatter DTOs for Markdown note metadata.

use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// Parser-owned frontmatter category token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NoteCategory(String);

impl NoteCategory {
    /// Build a frontmatter category token.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Borrow the category as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for NoteCategory {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for NoteCategory {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl Deref for NoteCategory {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

/// Parser-owned YAML frontmatter extracted from a Markdown document.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteFrontmatter {
    /// Document title from frontmatter.
    pub title: Option<String>,
    /// Human-readable description.
    pub description: Option<String>,
    /// Skill name or semantic document name.
    pub name: Option<String>,
    /// Document category.
    pub category: Option<NoteCategory>,
    /// Tags for discovery and filtering.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Routing keywords from `metadata.routing_keywords`.
    #[serde(default)]
    pub routing_keywords: Vec<String>,
    /// Intent descriptions from `metadata.intents`.
    #[serde(default)]
    pub intents: Vec<String>,
}
