//! `link_graph::addressing::topology::types` owns Wendao addressing topology types behavior.

use std::collections::HashMap;

/// A path entry representing a node's position in the document structure.
#[derive(Debug, Clone)]
pub struct PathEntry {
    /// Full structural path (e.g., `["Architecture", "Storage", "Configuration"]`).
    pub path: Vec<String>,
    /// Stable node ID for anchoring after discovery.
    pub node_id: String,
    /// Display title (may differ from path if title was renamed).
    pub title: String,
    /// Heading level (1-6).
    pub level: usize,
    /// Document ID containing this node.
    pub doc_id: String,
    /// Content hash for self-healing lookups.
    pub content_hash: Option<String>,
}

/// A match result from fuzzy path resolution.
#[derive(Debug, Clone)]
pub struct PathMatch {
    /// Document ID containing the match.
    pub doc_id: String,
    /// Full structural path to the matched node.
    pub path: Vec<String>,
    /// Similarity score (0.0-1.0, where 1.0 is exact match).
    pub similarity_score: f32,
    /// The matched path entry.
    pub entry: PathEntry,
    /// How the match was found.
    pub match_type: MatchType,
}

/// Classification of how a path match was discovered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchType {
    /// Exact path match.
    Exact,
    /// Path suffix match (partial path from the end).
    Suffix,
    /// Title substring match.
    TitleSubstring,
    /// Title edit-distance match.
    TitleFuzzy,
    /// Content hash match (self-healing).
    HashFallback,
    /// Case-insensitive match.
    CaseInsensitive,
}

/// Fuzzy path matching index for human discovery.
///
/// This index enables discovery of nodes when the exact path is not known,
/// supporting partial matches, title searches, and content hash fallbacks.
///
/// # Example
///
/// ```ignore
/// let topology = TopologyIndex::build_from_graph(&link_graph);
///
/// // Exact path lookup
/// if let Some(entry) = topology.exact_path("doc.md", &["Architecture", "Storage"]) {
///     println!("Found: {}", entry.title);
/// }
///
/// // Fuzzy discovery
/// let matches = topology.fuzzy_resolve("storage", 5);
/// for m in matches {
///     println!("{} (score: {})", m.path.join("/"), m.similarity_score);
/// }
/// ```
#[derive(Debug, Clone, Default)]
pub struct TopologyIndex {
    /// Structural paths for each document: `doc_id` → path entries.
    pub(crate) by_doc: HashMap<String, Vec<PathEntry>>,
    /// Lowercase title → possible matches (for fuzzy search).
    pub(crate) title_index: HashMap<String, Vec<PathMatch>>,
    /// Content hash → path entries (for self-healing).
    pub(crate) hash_index: HashMap<String, PathEntry>,
}

impl TopologyIndex {
    /// Create an empty topology index.
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_doc: HashMap::new(),
            title_index: HashMap::new(),
            hash_index: HashMap::new(),
        }
    }
}
