//! `analyzers::service::projection::docs_tool::options` owns Wendao projection docs tool options behavior.

use crate::analyzers::projection::ProjectionPageKind;

/// Default related-page limit for docs capability calls.
pub const DEFAULT_DOCS_RELATED_LIMIT: usize = 5;
/// Default family-cluster limit for docs navigation capability calls.
pub const DEFAULT_DOCS_FAMILY_LIMIT: usize = 3;

/// Optional parameters for docs navigation capability calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocsNavigationOptions {
    /// Optional stable page-index node identifier.
    pub node_id: Option<String>,
    /// Optional projected-page family to expand alongside navigation.
    pub family_kind: Option<ProjectionPageKind>,
    /// Maximum number of related projected pages to return.
    pub related_limit: usize,
    /// Maximum number of family-cluster entries to return.
    pub family_limit: usize,
}

impl Default for DocsNavigationOptions {
    fn default() -> Self {
        Self {
            node_id: None,
            family_kind: None,
            related_limit: DEFAULT_DOCS_RELATED_LIMIT,
            family_limit: DEFAULT_DOCS_FAMILY_LIMIT,
        }
    }
}

impl DocsNavigationOptions {
    #[must_use]
    pub(crate) fn normalized(self) -> Self {
        Self {
            family_limit: self.family_limit.max(1),
            ..self
        }
    }
}

/// Optional parameters for docs retrieval-context capability calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocsRetrievalContextOptions {
    /// Optional stable page-index node identifier.
    pub node_id: Option<String>,
    /// Maximum number of related projected pages to return.
    pub related_limit: usize,
}

impl Default for DocsRetrievalContextOptions {
    fn default() -> Self {
        Self {
            node_id: None,
            related_limit: DEFAULT_DOCS_RELATED_LIMIT,
        }
    }
}
