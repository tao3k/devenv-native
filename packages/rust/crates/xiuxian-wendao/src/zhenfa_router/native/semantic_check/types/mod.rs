//! Shared types for semantic checking.

/// Standard property drawer attribute keys (Blueprint v2.0).
#[path = "attrs.rs"]
pub(super) mod attrs;
mod data;

/// Parser-owned hash-aligned reference extracted from semantic-check wiki-link syntax.
pub use crate::parsers::semantic_check::HashReference;
pub use data::{
    CheckType, FileAuditReport, FuzzySuggestionData, IssueLocation, NodeStatus,
    SemanticCheckResult, SemanticIssue, WendaoSemanticCheckArgs,
};
