use serde::Deserialize;

/// Basic repository query parameters.
#[derive(Debug, Deserialize)]
pub struct RepoApiQuery {
    /// The repository identifier.
    pub repo: Option<String>,
}

/// Query parameters for projected page lookup.
#[derive(Debug, Deserialize)]
pub struct RepoProjectedPageApiQuery {
    /// The repository identifier.
    pub repo: Option<String>,
    /// The projected page identifier.
    pub page_id: Option<String>,
}

/// Query parameters for projected page-index node lookup.
#[derive(Debug, Deserialize)]
pub struct RepoProjectedPageIndexNodeApiQuery {
    /// The repository identifier.
    pub repo: Option<String>,
    /// The projected page identifier.
    pub page_id: Option<String>,
    /// The page-index node identifier.
    pub node_id: Option<String>,
}
