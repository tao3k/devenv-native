//! Canonical parser families for Wendao domain-core document understanding.

/// Docs-governance parser helpers.
#[path = "parsers/docs_governance.rs"]
pub(crate) mod docs_governance;
/// Graph persistence parsing.
#[path = "parsers/graph.rs"]
pub mod graph;
/// Language-specific parser families.
#[path = "parsers/languages.rs"]
pub mod languages;
#[path = "parsers/link_graph.rs"]
pub mod link_graph;
#[path = "parsers/markdown.rs"]
pub mod markdown;
/// Search query parsing.
#[path = "parsers/search.rs"]
pub mod search;
/// Semantic-check grammar helpers.
#[path = "parsers/semantic_check/mod.rs"]
pub(crate) mod semantic_check;
#[path = "parsers/zhixing.rs"]
pub mod zhixing;
