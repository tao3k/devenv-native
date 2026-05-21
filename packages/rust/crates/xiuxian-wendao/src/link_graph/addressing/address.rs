//! `link_graph::addressing::address` owns Wendao link graph addressing address behavior.

use crate::link_graph::models::BlockAddress;
use crate::link_graph::models::PageIndexNode;

/// Semantic address for node resolution following the Triple-A protocol.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Address {
    /// Explicit anchor ID from `:ID:` property drawer.
    /// Highest priority - directly identifies a node.
    Id(String),
    /// Structural path through heading hierarchy.
    /// Medium priority - e.g., `["Architecture", "Storage"]`.
    Path(Vec<String>),
    /// Content fingerprint (Blake3 hash).
    /// Lowest priority - used for self-healing when content moved.
    Hash(String),
    /// Block-level address within a section.
    ///
    /// Enables fine-grained addressing like `/Section/Paragraph[2]`.
    Block {
        /// Path to the containing section.
        section_path: Vec<String>,
        /// Block address within the section.
        block_addr: BlockAddress,
    },
}

impl Address {
    /// Create an ID-based address.
    #[must_use]
    pub fn id(id: impl Into<String>) -> Self {
        Self::Id(id.into())
    }

    /// Create a path-based address.
    #[must_use]
    pub fn path(components: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Path(components.into_iter().map(Into::into).collect())
    }

    /// Create a hash-based address.
    #[must_use]
    pub fn hash(hash: impl Into<String>) -> Self {
        Self::Hash(hash.into())
    }

    /// Parse an address string.
    ///
    /// Formats:
    /// - `#id` - explicit ID (e.g., `#arch-v1`)
    /// - `/path/to/heading` - structural path
    /// - `@hash` - content hash
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }

        if let Some(id) = trimmed.strip_prefix('#') {
            if id.is_empty() {
                return None;
            }
            return Some(Self::Id(id.to_string()));
        }

        if let Some(hash) = trimmed.strip_prefix('@') {
            if hash.is_empty() {
                return None;
            }
            return Some(Self::Hash(hash.to_string()));
        }

        let path_str = trimmed.strip_prefix('/').unwrap_or(trimmed);
        let components: Vec<String> = path_str
            .split('/')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if components.is_empty() {
            return None;
        }

        Some(Self::Path(components))
    }

    /// Format address as a human-readable string.
    #[must_use]
    pub fn to_display_string(&self) -> String {
        match self {
            Self::Id(id) => format!("#{id}"),
            Self::Path(components) => format!("/{}", components.join("/")),
            Self::Hash(hash) => format!("@{hash}"),
            Self::Block {
                section_path,
                block_addr,
            } => format!(
                "/{}{}",
                section_path.join("/"),
                block_addr.to_path_component()
            ),
        }
    }
}

/// Resolution result containing the found node and any path migration info.
#[derive(Debug, Clone)]
pub struct ResolvedNode {
    /// The resolved node.
    pub node: PageIndexNode,
    /// Document ID containing the node.
    pub doc_id: String,
    /// Whether the address was found via a different addressing mode.
    /// E.g., ID not found, but found via path.
    pub migrated_from: Option<Address>,
}

/// Enhanced resolution result with detailed match information.
#[derive(Debug, Clone)]
pub struct EnhancedResolvedNode {
    /// The resolved node.
    pub node: PageIndexNode,
    /// Document ID containing the node.
    pub doc_id: String,
    /// Actual path matched (may differ from request if fuzzy).
    pub resolved_path: Vec<String>,
    /// Stable ID for future anchoring.
    pub resolved_id: Option<String>,
    /// How the match was found.
    pub match_type: crate::link_graph::addressing::MatchType,
    /// Fuzzy match score (1.0 = exact).
    pub similarity: f32,
}

/// Resolution mode for dual-index addressing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveMode {
    /// Exact ID lookup only (O(1)).
    Anchor,
    /// Fuzzy path discovery with path drift tolerance.
    Discover {
        /// Enable fuzzy matching.
        fuzzy: bool,
        /// Maximum number of results to return.
        max_results: usize,
    },
}
