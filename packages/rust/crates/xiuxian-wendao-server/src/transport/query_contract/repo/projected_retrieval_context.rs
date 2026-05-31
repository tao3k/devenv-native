//! Projected retrieval-context route contract and metadata validation.

/// Canonical projected retrieval-context repository metadata header for Wendao
/// Flight requests.
pub const WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER: &str =
    "x-wendao-repo-projected-retrieval-context-repo";
/// Canonical projected retrieval-context page metadata header for Wendao
/// Flight requests.
pub const WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER: &str =
    "x-wendao-repo-projected-retrieval-context-page-id";
/// Canonical projected retrieval-context node metadata header for Wendao
/// Flight requests.
pub const WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER: &str =
    "x-wendao-repo-projected-retrieval-context-node-id";
/// Canonical projected retrieval-context related-limit metadata header for
/// Wendao Flight requests.
pub const WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER: &str =
    "x-wendao-repo-projected-retrieval-context-related-limit";
/// Stable route for the repo projected retrieval-context analysis contract.
pub const ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE: &str =
    "/analysis/repo-projected-retrieval-context";
/// Stable default related page limit for projected retrieval-context requests.
pub const REPO_PROJECTED_RETRIEVAL_CONTEXT_DEFAULT_RELATED_LIMIT: usize = 5;

use crate::transport::query_contract::{NodeIdRef, PageIdRef, RepoIdRef};

macro_rules! retrieval_context_token {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name(String);

        impl $name {
            /// Build a normalized projected retrieval-context token.
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            /// Borrow this token as a string slice.
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            /// Return the owned string.
            #[must_use]
            pub fn into_string(self) -> String {
                self.0
            }
        }
    };
}

retrieval_context_token!(
    RepoProjectedRetrievalContextRepoId,
    "Normalized repository identifier for projected retrieval-context requests."
);
retrieval_context_token!(
    RepoProjectedRetrievalContextPageId,
    "Normalized page identifier for projected retrieval-context requests."
);
retrieval_context_token!(
    RepoProjectedRetrievalContextNodeId,
    "Normalized node identifier for projected retrieval-context requests."
);

/// Projected retrieval-context request input.
#[derive(Debug, Clone, Copy)]
pub struct RepoProjectedRetrievalContextInput<'a> {
    /// Repository identifier from request metadata.
    pub repo_id: RepoIdRef<'a>,
    /// Page identifier from request metadata.
    pub page_id: PageIdRef<'a>,
    /// Optional node identifier from request metadata.
    pub node_id: Option<NodeIdRef<'a>>,
    /// Optional related-page limit.
    pub related_limit: Option<usize>,
}

/// Normalized projected retrieval-context request metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoProjectedRetrievalContextRequest {
    /// Normalized repository identifier.
    pub repo_id: RepoProjectedRetrievalContextRepoId,
    /// Normalized page identifier.
    pub page_id: RepoProjectedRetrievalContextPageId,
    /// Optional normalized node identifier.
    pub node_id: Option<RepoProjectedRetrievalContextNodeId>,
    /// Normalized related-page limit.
    pub related_limit: usize,
}

/// Validate the stable projected retrieval-context request contract.
///
/// # Errors
///
/// Returns an error when the repository identifier or page identifier is
/// blank, or when the related-limit value is zero.
pub fn validate_repo_projected_retrieval_context_request(
    input: RepoProjectedRetrievalContextInput<'_>,
) -> Result<RepoProjectedRetrievalContextRequest, String> {
    let normalized_repo_id = input.repo_id.trim();
    if normalized_repo_id.is_empty() {
        return Err("repo projected retrieval-context repo must not be blank".to_string());
    }
    let normalized_page_id = input.page_id.trim();
    if normalized_page_id.is_empty() {
        return Err("repo projected retrieval-context page id must not be blank".to_string());
    }
    let normalized_node_id = input
        .node_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(RepoProjectedRetrievalContextNodeId::new);
    let related_limit = input
        .related_limit
        .unwrap_or(REPO_PROJECTED_RETRIEVAL_CONTEXT_DEFAULT_RELATED_LIMIT);
    if related_limit == 0 {
        return Err(
            "repo projected retrieval-context related_limit must be greater than zero".to_string(),
        );
    }
    Ok(RepoProjectedRetrievalContextRequest {
        repo_id: RepoProjectedRetrievalContextRepoId::new(normalized_repo_id),
        page_id: RepoProjectedRetrievalContextPageId::new(normalized_page_id),
        node_id: normalized_node_id,
        related_limit,
    })
}
