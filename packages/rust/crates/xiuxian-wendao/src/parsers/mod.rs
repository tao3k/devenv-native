//! Canonical parser families for Wendao domain-core document understanding.

/// Docs-governance parser helpers.
#[cfg(any(test, feature = "zhenfa-router"))]
#[path = "docs_governance/mod.rs"]
pub(crate) mod docs_governance;
/// Graph persistence parsing.
#[path = "graph/mod.rs"]
pub mod graph;
/// Language-specific parser families.
#[path = "languages/mod.rs"]
pub mod languages;
#[path = "link_graph/mod.rs"]
pub mod link_graph;
#[path = "markdown/mod.rs"]
pub mod markdown;
/// Search query parsing.
#[path = "search/mod.rs"]
pub mod search;
/// Semantic-check grammar helpers.
#[cfg(any(test, feature = "zhenfa-router"))]
#[path = "semantic_check/mod.rs"]
pub(crate) mod semantic_check;
#[path = "zhixing/mod.rs"]
pub mod zhixing;
