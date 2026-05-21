//! `search::queries::core::service` owns Wendao queries core service behavior.

use std::path::PathBuf;

use crate::search::SearchPlaneService;
#[cfg(not(feature = "duckdb"))]
use crate::search::queries::core::scope::{DataFusionQueryCore, open_datafusion_query_core};
use crate::search::queries::sql::registration::{SqlQuerySurface, build_sql_query_surface};

/// Canonical shared-query service owner above the residual request-scoped `DataFusion` query core.
#[derive(Clone)]
pub struct SearchQueryService {
    search_plane: SearchPlaneService,
}

impl SearchQueryService {
    /// Create a shared query service over the provided search plane.
    #[must_use]
    pub fn new(search_plane: SearchPlaneService) -> Self {
        Self { search_plane }
    }

    /// Create a shared query service from one project root.
    #[must_use]
    pub fn from_project_root(project_root: impl Into<PathBuf>) -> Self {
        Self::new(SearchPlaneService::new(project_root.into()))
    }

    pub(crate) fn search_plane(&self) -> &SearchPlaneService {
        &self.search_plane
    }

    #[cfg(not(feature = "duckdb"))]
    pub(crate) async fn open_datafusion_core(&self) -> Result<DataFusionQueryCore, String> {
        open_datafusion_query_core(self.search_plane()).await
    }

    pub(crate) async fn open_sql_surface(&self) -> Result<SqlQuerySurface, String> {
        build_sql_query_surface(self.search_plane()).await
    }
}

impl From<SearchPlaneService> for SearchQueryService {
    fn from(search_plane: SearchPlaneService) -> Self {
        Self::new(search_plane)
    }
}
