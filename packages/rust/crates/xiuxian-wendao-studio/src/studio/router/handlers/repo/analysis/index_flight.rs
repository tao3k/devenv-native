use std::sync::Arc;

use async_trait::async_trait;
use xiuxian_wendao_server::transport::{AnalysisFlightRouteResponse, RepoIndexFlightRouteProvider};

use crate::studio::router::GatewayState;
use crate::studio::router::handlers::repo::analysis::index_status_flight::{
    build_repo_index_status_flight_batch, build_repo_index_status_flight_metadata,
};
use crate::studio::router::handlers::repo::command_service::run_repo_index;
use xiuxian_wendao::repo_index::RepoIndexRequest;

#[derive(Clone)]
pub(crate) struct StudioRepoIndexFlightRouteProvider {
    state: Arc<GatewayState>,
}

impl StudioRepoIndexFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self { state }
    }
}

impl std::fmt::Debug for StudioRepoIndexFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioRepoIndexFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl RepoIndexFlightRouteProvider for StudioRepoIndexFlightRouteProvider {
    async fn repo_index_batch(
        &self,
        repo_id: Option<&str>,
        refresh: bool,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        let response = run_repo_index(
            Arc::clone(&self.state),
            RepoIndexRequest {
                repo: repo_id.map(ToString::to_string),
                refresh,
            },
        )
        .await
        .map_err(|error| map_studio_api_error(&error))?;
        let batch = build_repo_index_status_flight_batch(&response)?;
        let metadata = build_repo_index_status_flight_metadata(&response)?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(metadata))
    }
}

fn map_studio_api_error(error: &crate::studio::router::StudioApiError) -> String {
    error
        .error
        .details
        .clone()
        .unwrap_or_else(|| format!("{}: {}", error.code(), error.error.message))
}
