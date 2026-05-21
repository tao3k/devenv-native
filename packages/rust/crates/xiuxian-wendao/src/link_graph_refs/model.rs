//! `link_graph_refs::model` owns Wendao link graph refs model behavior.

use serde::{Deserialize, Serialize};

/// Represents an entity reference extracted from note content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkGraphEntityRef {
    /// Entity name or note target without any address fragment.
    pub name: String,
    /// Optional Obsidian-style heading or block address.
    #[serde(default)]
    pub target_address: Option<String>,
    /// Original matched text
    #[serde(skip)]
    pub original: String,
}

impl LinkGraphEntityRef {
    /// Create a new entity reference.
    #[must_use]
    pub fn new(name: String, target_address: Option<String>, original: String) -> Self {
        Self {
            name,
            target_address,
            original,
        }
    }

    /// Get the wikilink format: `[[Name]]` or `[[Name#Heading]]`.
    #[must_use]
    pub fn to_wikilink(&self) -> String {
        match &self.target_address {
            Some(value) => format!("[[{}{}]]", self.name, value),
            None => format!("[[{}]]", self.name),
        }
    }

    /// Get a coarse structural tag for this extracted link.
    #[must_use]
    pub fn to_tag(&self) -> String {
        if self.target_address.is_some() {
            "#entity-addressed".to_string()
        } else {
            "#entity".to_string()
        }
    }
}
