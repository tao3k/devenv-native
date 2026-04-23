#[cfg(not(feature = "duckdb"))]
use xiuxian_db_store::SearchEngineContext;

#[cfg(not(feature = "duckdb"))]
use crate::search::SearchPlaneService;
#[cfg(not(feature = "duckdb"))]
use crate::search::queries::sql::registration::{
    SqlQuerySurface, register_datafusion_sql_query_surface,
};

/// Shared request-scoped `DataFusion` query core over the visible search-plane data.
#[cfg(not(feature = "duckdb"))]
pub(crate) struct DataFusionQueryCore {
    datafusion_query_engine: SearchEngineContext,
    surface: SqlQuerySurface,
}

#[cfg(not(feature = "duckdb"))]
impl DataFusionQueryCore {
    fn new(datafusion_query_engine: SearchEngineContext, surface: SqlQuerySurface) -> Self {
        Self {
            datafusion_query_engine,
            surface,
        }
    }

    pub(crate) fn datafusion_engine(&self) -> &SearchEngineContext {
        &self.datafusion_query_engine
    }

    pub(crate) fn surface(&self) -> &SqlQuerySurface {
        &self.surface
    }
}

#[cfg(not(feature = "duckdb"))]
pub(crate) async fn open_datafusion_query_core(
    service: &SearchPlaneService,
) -> Result<DataFusionQueryCore, String> {
    let (datafusion_query_engine, surface) = register_datafusion_sql_query_surface(service).await?;
    Ok(DataFusionQueryCore::new(datafusion_query_engine, surface))
}
