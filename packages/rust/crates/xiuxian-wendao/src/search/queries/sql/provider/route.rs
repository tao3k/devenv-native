//! `search::queries::sql::provider::route` owns Wendao sql provider route behavior.

use async_trait::async_trait;
use xiuxian_wendao_runtime::transport::{SqlFlightRouteProvider, SqlFlightRouteResponse};

use crate::search::queries::SearchQueryService;

use super::metadata::StudioSqlFlightMetadata;
use crate::search::queries::sql::execute_sql_query;

#[derive(Clone)]
/// SQL Flight route provider backed by the Wendao search query service.
pub struct StudioSqlFlightRouteProvider {
    service: SearchQueryService,
}

impl StudioSqlFlightRouteProvider {
    /// Create a SQL Flight route provider from a search query service.
    #[must_use]
    pub fn new(service: impl Into<SearchQueryService>) -> Self {
        Self {
            service: service.into(),
        }
    }
}

impl std::fmt::Debug for StudioSqlFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioSqlFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl SqlFlightRouteProvider for StudioSqlFlightRouteProvider {
    async fn sql_query_batches(&self, query_text: &str) -> Result<SqlFlightRouteResponse, String> {
        let result = execute_sql_query(&self.service, query_text).await?;
        let (metadata, engine_batches) = result.into_parts();
        let app_metadata = serde_json::to_vec(&StudioSqlFlightMetadata {
            result_batch_count: engine_batches.len(),
            ..metadata
        })
        .map_err(|error| {
            format!("studio SQL Flight provider failed to encode app metadata: {error}")
        })?;

        Ok(SqlFlightRouteResponse::new(engine_batches).with_app_metadata(app_metadata))
    }
}
