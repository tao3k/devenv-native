use serde::Deserialize;

/// Query parameters for repository-wide search.
#[derive(Debug, Deserialize)]
pub struct RepoSearchApiQuery {
    /// The repository identifier.
    pub repo: Option<String>,
    /// The search query string.
    pub query: Option<String>,
    /// Maximum number of hits to return.
    pub limit: Option<usize>,
}

/// Query parameters for repository import search.
#[derive(Debug, Deserialize)]
pub struct RepoImportSearchApiQuery {
    /// The repository identifier.
    pub repo: Option<String>,
    /// Optional target package filter.
    pub package: Option<String>,
    /// Optional source-module filter.
    pub module: Option<String>,
    /// Maximum number of hits to return.
    pub limit: Option<usize>,
}

/// Query parameters for documentation coverage inspection.
#[derive(Debug, Deserialize)]
pub struct RepoDocCoverageApiQuery {
    /// The repository identifier.
    pub repo: Option<String>,
    /// Optional module identifier filter.
    #[serde(rename = "module")]
    pub module_id: Option<String>,
}

/// Query parameters for repository source synchronization.
#[derive(Debug, Deserialize)]
pub struct RepoSyncApiQuery {
    /// The repository identifier.
    pub repo: Option<String>,
    /// The synchronization mode ("ensure", "refresh", or "status").
    pub mode: Option<String>,
}

/// Query parameters for repo index status.
#[derive(Debug, Deserialize)]
pub struct RepoIndexStatusApiQuery {
    /// Optional repository identifier filter.
    pub repo: Option<String>,
}
