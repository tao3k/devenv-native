use std::sync::Arc;

use async_trait::async_trait;
use xiuxian_wendao_runtime::transport::{
    AnalysisFlightRouteResponse, RepoIndexStatusFlightRouteProvider,
};

use crate::gateway::studio::router::GatewayState;
use crate::gateway::studio::router::handlers::repo::command_service::run_repo_index_status;

use super::diagnostics::repo_index_status_response_with_diagnostics;
use super::encoding::{
    build_repo_index_status_flight_batch, build_repo_index_status_flight_metadata,
};

#[derive(Clone)]
pub(crate) struct StudioRepoIndexStatusFlightRouteProvider {
    state: Arc<GatewayState>,
}

impl StudioRepoIndexStatusFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self { state }
    }
}

impl std::fmt::Debug for StudioRepoIndexStatusFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioRepoIndexStatusFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl RepoIndexStatusFlightRouteProvider for StudioRepoIndexStatusFlightRouteProvider {
    async fn repo_index_status_batch(
        &self,
        repo_id: Option<&str>,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        let response = run_repo_index_status(&self.state, repo_id);
        let response = repo_index_status_response_with_diagnostics(&response).await;
        let batch = build_repo_index_status_flight_batch(&response)?;
        let metadata = build_repo_index_status_flight_metadata(&response)?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(metadata))
    }
}
