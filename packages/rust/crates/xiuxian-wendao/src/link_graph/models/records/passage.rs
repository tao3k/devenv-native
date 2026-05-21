//! `link_graph::models::records::passage` owns Wendao models records passage behavior.

use serde::{Deserialize, Serialize};

/// Represents a fine-grained content block (`Passage`) within a document.
/// Directly supports `HippoRAG 2`'s `Passage Nodes` for better grounding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkGraphPassage {
    /// Stable passage identifier (`doc_id#passage-<idx>-<heading-slug>`).
    pub id: String,
    /// Parent document identifier that owns this passage node.
    pub parent_doc_id: String,
    /// Original passage text extracted from a section.
    pub content: String,
    /// Lower-cased passage text for fast case-insensitive matching.
    pub content_lower: String,
    /// Entity identifiers referenced inside this passage.
    pub entities: Vec<String>,
}
