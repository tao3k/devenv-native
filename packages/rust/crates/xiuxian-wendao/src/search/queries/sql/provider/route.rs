use async_trait::async_trait;
use xiuxian_db_store::engine_batches_to_lance_batches;
use xiuxian_wendao_runtime::transport::{SqlFlightRouteProvider, SqlFlightRouteResponse};

use crate::search::queries::SearchQueryService;

use super::metadata::StudioSqlFlightMetadata;
use crate::search::queries::sql::execute_sql_query;

#[derive(Clone)]
pub(crate) struct StudioSqlFlightRouteProvider {
    service: SearchQueryService,
}

impl StudioSqlFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(service: impl Into<SearchQueryService>) -> Self {
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
        let batches =
            engine_batches_to_lance_batches(engine_batches.as_slice()).map_err(|error| {
                format!(
                    "studio SQL Flight provider failed to convert SQL response batches for `{query_text}`: {error}"
                )
            })?;
        let app_metadata = serde_json::to_vec(&StudioSqlFlightMetadata {
            result_batch_count: batches.len(),
            ..metadata
        })
        .map_err(|error| {
            format!("studio SQL Flight provider failed to encode app metadata: {error}")
        })?;

        Ok(SqlFlightRouteResponse::new(batches).with_app_metadata(app_metadata))
    }
}
