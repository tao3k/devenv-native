use std::sync::Arc;

use crate::studio::arrow_types::{
    LanceDataType, LanceField, LanceRecordBatch, LanceSchema, LanceStringArray, LanceUInt64Array,
};
use async_trait::async_trait;
use tonic::Status;
use xiuxian_wendao::analyzers::RepoProjectedRetrievalContextResult;
use xiuxian_wendao_server::transport::{
    AnalysisFlightRouteResponse, RepoProjectedRetrievalContextFlightRouteProvider,
};

use crate::studio::router::handlers::repo::projected_service::retrieval::run_repo_projected_retrieval_context;
use crate::studio::router::{GatewayState, StudioApiError};

#[derive(Clone)]
pub(crate) struct StudioRepoProjectedRetrievalContextFlightRouteProvider {
    state: Arc<GatewayState>,
}

impl StudioRepoProjectedRetrievalContextFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self { state }
    }
}

impl std::fmt::Debug for StudioRepoProjectedRetrievalContextFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioRepoProjectedRetrievalContextFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl RepoProjectedRetrievalContextFlightRouteProvider
    for StudioRepoProjectedRetrievalContextFlightRouteProvider
{
    async fn repo_projected_retrieval_context_batch(
        &self,
        repo_id: &str,
        page_id: &str,
        node_id: Option<&str>,
        related_limit: usize,
    ) -> Result<AnalysisFlightRouteResponse, Status> {
        let response = run_repo_projected_retrieval_context(
            Arc::clone(&self.state),
            xiuxian_wendao::analyzers::RepoProjectedRetrievalContextQuery {
                repo_id: repo_id.to_string(),
                page_id: page_id.to_string(),
                node_id: node_id.map(ToString::to_string),
                related_limit,
            },
        )
        .await
        .map_err(studio_api_error_to_tonic_status)?;
        let batch = repo_projected_retrieval_context_batch_with_requested_node(&response, node_id)
            .map_err(Status::internal)?;
        let metadata =
            repo_projected_retrieval_context_metadata_with_requested_node(&response, node_id)
                .map_err(Status::internal)?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(metadata))
    }
}

#[cfg(test)]
pub(crate) fn repo_projected_retrieval_context_batch(
    response: &RepoProjectedRetrievalContextResult,
) -> Result<LanceRecordBatch, String> {
    repo_projected_retrieval_context_batch_with_requested_node(response, None)
}

fn repo_projected_retrieval_context_batch_with_requested_node(
    response: &RepoProjectedRetrievalContextResult,
    requested_node_id: Option<&str>,
) -> Result<LanceRecordBatch, String> {
    let center_json = serde_json::to_string(&response.center)
        .map_err(|error| format!("failed to encode retrieval-context center: {error}"))?;
    let related_pages_json = serde_json::to_string(response.related_pages.as_slice())
        .map_err(|error| format!("failed to encode retrieval-context related pages: {error}"))?;
    let node_context_json = response
        .node_context
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|error| format!("failed to encode retrieval-context node context: {error}"))?;
    let related_count = u64::try_from(response.related_pages.len())
        .map_err(|error| format!("failed to represent related page count: {error}"))?;
    let node_id = response_node_id(response, requested_node_id);

    LanceRecordBatch::try_new(
        Arc::new(LanceSchema::new(vec![
            LanceField::new("repoId", LanceDataType::Utf8, false),
            LanceField::new("pageId", LanceDataType::Utf8, false),
            LanceField::new("nodeId", LanceDataType::Utf8, true),
            LanceField::new("centerJson", LanceDataType::Utf8, false),
            LanceField::new("relatedCount", LanceDataType::UInt64, false),
            LanceField::new("relatedPagesJson", LanceDataType::Utf8, false),
            LanceField::new("nodeContextJson", LanceDataType::Utf8, true),
        ])),
        vec![
            Arc::new(LanceStringArray::from(vec![response.repo_id.as_str()])),
            Arc::new(LanceStringArray::from(vec![
                response.center.page.page_id.as_str(),
            ])),
            Arc::new(LanceStringArray::from(vec![node_id])),
            Arc::new(LanceStringArray::from(vec![center_json.as_str()])),
            Arc::new(LanceUInt64Array::from(vec![related_count])),
            Arc::new(LanceStringArray::from(vec![related_pages_json.as_str()])),
            Arc::new(LanceStringArray::from(vec![node_context_json.as_deref()])),
        ],
    )
    .map_err(|error| error.to_string())
}

#[cfg(test)]
pub(crate) fn repo_projected_retrieval_context_metadata(
    response: &RepoProjectedRetrievalContextResult,
) -> Result<Vec<u8>, String> {
    repo_projected_retrieval_context_metadata_with_requested_node(response, None)
}

fn repo_projected_retrieval_context_metadata_with_requested_node(
    response: &RepoProjectedRetrievalContextResult,
    requested_node_id: Option<&str>,
) -> Result<Vec<u8>, String> {
    let node_id = response_node_id(response, requested_node_id);

    serde_json::to_vec(&serde_json::json!({
        "repoId": response.repo_id,
        "pageId": response.center.page.page_id,
        "nodeId": node_id,
        "relatedCount": response.related_pages.len(),
        "hasNodeContext": response.node_context.is_some(),
    }))
    .map_err(|error| error.to_string())
}

fn response_node_id<'a>(
    response: &'a RepoProjectedRetrievalContextResult,
    requested_node_id: Option<&'a str>,
) -> Option<&'a str> {
    response
        .center
        .node
        .as_ref()
        .map(|node| node.node_id.as_str())
        .or_else(|| response.node_context.as_ref().and(requested_node_id))
}

fn studio_api_error_to_tonic_status(error: StudioApiError) -> Status {
    match error.status() {
        axum::http::StatusCode::BAD_REQUEST => Status::invalid_argument(error.error.message),
        axum::http::StatusCode::NOT_FOUND => Status::not_found(error.error.message),
        axum::http::StatusCode::CONFLICT => Status::failed_precondition(error.error.message),
        _ => Status::internal(error.error.message),
    }
}

#[cfg(test)]
#[path = "../../../../../../tests/unit/gateway/studio/router/handlers/repo/analysis/projected_retrieval_context_flight.rs"]
mod tests;
