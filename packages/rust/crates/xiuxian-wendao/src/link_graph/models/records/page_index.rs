//! `link_graph::models::records::page_index` owns Wendao models records page index behavior.

use std::collections::HashMap;
use std::sync::Arc;

use super::MarkdownBlock;
use crate::parsers::markdown::{CodeObservation, LogbookEntry};

/// One hierarchical `PageIndex` node derived from a markdown heading section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageIndexNode {
    /// Stable semantic node identifier scoped to a single document.
    pub node_id: String,
    /// Stable parent anchor identifier when this node is nested under another heading.
    pub parent_id: Option<String>,
    /// Display title for this node.
    pub title: String,
    /// Heading depth normalized to the range `1..=6`.
    pub level: usize,
    /// Content chunk owned by this node after optional thinning.
    pub text: Arc<str>,
    /// Optional downstream summary payload.
    pub summary: Option<String>,
    /// Child headings nested under this node.
    pub children: Vec<PageIndexNode>,
    /// Structural metadata collected during build/thinning.
    pub metadata: PageIndexMeta,
    /// Block-level children for fine-grained addressing.
    ///
    /// Populated during index build by parsing section content into
    /// paragraphs, code fences, lists, etc. Empty if not yet parsed.
    pub blocks: Vec<MarkdownBlock>,
}

/// Metadata describing a `PageIndex` node's source span, identity, and pruning state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageIndexMeta {
    /// Inclusive 1-based source line range within the markdown body.
    /// NOTE: This is treated as a "viewport" and may be outdated after edits.
    pub line_range: (usize, usize),
    /// Optional byte range for precise AST-level mutations.
    pub byte_range: Option<(usize, usize)>,
    /// Hierarchical structural path (e.g., `["Heading", "Architecture", "Storage"]`).
    pub structural_path: Vec<String>,
    /// Content-based fingerprint (Blake3 hash) for self-healing and deduplication.
    pub content_hash: Option<String>,
    /// Property drawer attributes extracted from the heading (e.g., :ID:).
    pub attributes: HashMap<String, String>,
    /// Best-effort whitespace token count for the current node text.
    pub token_count: usize,
    /// Whether descendant content was folded into this node.
    pub is_thinned: bool,
    /// Execution log entries from :LOGBOOK: drawer (Blueprint v2.4).
    pub logbook: Vec<LogbookEntry>,
    /// Code observations from :OBSERVE: property drawer (Blueprint v2.7).
    pub observations: Vec<CodeObservation>,
}
