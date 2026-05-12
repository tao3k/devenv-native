//! Docs governance module for semantic checking.
//!
//! This module provides document governance validation for package-local
//! crate documentation, ensuring proper identity protocols, index structure,
//! and relation tracking.
/// Public Wendao boundary.

#[path = "collection/mod.rs"]
pub mod collection;
/// Public Wendao boundary.
#[path = "rendering/mod.rs"]
pub mod rendering;
#[path = "scope.rs"]
mod scope;
/// Public Wendao boundary.
#[path = "types.rs"]
pub mod types;

pub use collection::{collect_doc_governance_issues, collect_workspace_doc_governance_issues};
pub use types::{
    CANONICAL_DOC_HIDDEN_PATH_LINK_ISSUE_TYPE, DOC_IDENTITY_PROTOCOL_ISSUE_TYPE,
    INCOMPLETE_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_INDEX_FOOTER_BLOCK_ISSUE_TYPE, MISSING_PACKAGE_DOCS_INDEX_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_INDEX_RELATIONS_BLOCK_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_INDEX_SECTION_LINK_ISSUE_TYPE,
    MISSING_PACKAGE_DOCS_SECTION_LANDING_ISSUE_TYPE, MISSING_PACKAGE_DOCS_TREE_ISSUE_TYPE,
    STALE_PACKAGE_DOCS_INDEX_FOOTER_STANDARDS_ISSUE_TYPE,
    STALE_PACKAGE_DOCS_INDEX_RELATION_LINK_ISSUE_TYPE,
};

#[cfg(test)]
#[path = "../../../../../tests/unit/zhenfa_router/native/semantic_check/docs_governance/mod.rs"]
mod tests;
