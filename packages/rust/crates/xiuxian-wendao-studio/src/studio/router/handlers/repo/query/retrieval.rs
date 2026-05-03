use serde::Deserialize;

/// Query parameters for projected retrieval hit lookup.
#[derive(Debug, Deserialize)]
pub struct RepoProjectedRetrievalHitApiQuery {
    /// The repository identifier.
    pub repo: Option<String>,
    /// The projected page identifier.
    pub page_id: Option<String>,
    /// The page-index node identifier.
    pub node_id: Option<String>,
}

/// Query parameters for projected retrieval context lookup.
#[derive(Debug, Deserialize)]
pub struct RepoProjectedRetrievalContextApiQuery {
    /// The repository identifier.
    pub repo: Option<String>,
    /// The projected page identifier.
    pub page_id: Option<String>,
    /// The page-index node identifier.
    pub node_id: Option<String>,
    /// Maximum number of related hits to return.
    pub related_limit: Option<usize>,
}

/// Query parameters for projected-page search.
#[derive(Debug, Deserialize)]
pub struct RepoProjectedPageSearchApiQuery {
    /// The repository identifier.
    pub repo: Option<String>,
    /// The search query string.
    pub query: Option<String>,
    /// The projected page kind filter.
    pub kind: Option<String>,
    /// Maximum number of hits to return.
    pub limit: Option<usize>,
}
