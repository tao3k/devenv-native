use serde::Deserialize;

/// Query parameters for one docs-facing deterministic planner item.
#[derive(Debug, Deserialize)]
pub struct DocsPlannerItemApiQuery {
    /// The repository identifier.
    pub(crate) repo: Option<String>,
    /// Stable projected gap identifier.
    pub(crate) gap_id: Option<String>,
    /// Optional projected-page family to include as a deterministic cluster.
    pub(crate) family_kind: Option<String>,
    /// Maximum number of related projected pages to return.
    pub(crate) related_limit: Option<usize>,
    /// Maximum number of related projected pages to return in the requested family cluster.
    pub(crate) family_limit: Option<usize>,
}

/// Query parameters for docs-facing deterministic planner discovery.
#[derive(Debug, Deserialize)]
pub struct DocsPlannerSearchApiQuery {
    /// The repository identifier.
    pub(crate) repo: Option<String>,
    /// Planner search string.
    pub(crate) query: Option<String>,
    /// Optional projected gap kind filter.
    pub(crate) gap_kind: Option<String>,
    /// Optional projected-page family filter.
    pub(crate) page_kind: Option<String>,
    /// Maximum number of planner hits to return.
    pub(crate) limit: Option<usize>,
}

/// Query parameters for docs-facing deterministic planner queue shaping.
#[derive(Debug, Deserialize)]
pub struct DocsPlannerQueueApiQuery {
    /// The repository identifier.
    pub(crate) repo: Option<String>,
    /// Optional projected gap kind filter.
    pub(crate) gap_kind: Option<String>,
    /// Optional projected-page family filter.
    pub(crate) page_kind: Option<String>,
    /// Maximum number of preview gaps to return for each gap kind.
    pub(crate) per_kind_limit: Option<usize>,
}

/// Query parameters for docs-facing deterministic planner ranking.
#[derive(Debug, Deserialize)]
pub struct DocsPlannerRankApiQuery {
    /// The repository identifier.
    pub(crate) repo: Option<String>,
    /// Optional projected gap kind filter.
    pub(crate) gap_kind: Option<String>,
    /// Optional projected-page family filter.
    pub(crate) page_kind: Option<String>,
    /// Maximum number of ranked planner gaps to return.
    pub(crate) limit: Option<usize>,
}

/// Query parameters for docs-facing deterministic planner workset opening.
#[derive(Debug, Deserialize)]
pub struct DocsPlannerWorksetApiQuery {
    /// The repository identifier.
    pub(crate) repo: Option<String>,
    /// Optional projected gap kind filter.
    pub(crate) gap_kind: Option<String>,
    /// Optional projected-page family filter.
    pub(crate) page_kind: Option<String>,
    /// Maximum number of preview gaps to keep for each gap kind.
    pub(crate) per_kind_limit: Option<usize>,
    /// Maximum number of planner items to open across the queue preview.
    pub(crate) limit: Option<usize>,
    /// Optional projected-page family to include as a deterministic cluster.
    pub(crate) family_kind: Option<String>,
    /// Maximum number of related projected pages to return.
    pub(crate) related_limit: Option<usize>,
    /// Maximum number of related projected pages to return in the requested family cluster.
    pub(crate) family_limit: Option<usize>,
}
