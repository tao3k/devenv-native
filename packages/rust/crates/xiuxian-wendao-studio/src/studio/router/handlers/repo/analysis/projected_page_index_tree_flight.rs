use std::sync::Arc;

use crate::studio::arrow_types::{
    LanceDataType, LanceField, LanceRecordBatch, LanceSchema, LanceStringArray, LanceUInt64Array,
};
use async_trait::async_trait;
use tonic::Status;
use xiuxian_wendao_server::transport::{
    AnalysisFlightRouteResponse, RepoProjectedPageIndexTreeFlightRouteProvider,
};

use crate::studio::router::handlers::repo::projected_service::pages::run_repo_projected_page_index_tree;
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::RepoProjectedPageIndexTreeResult;

#[derive(Clone)]
pub(crate) struct StudioRepoProjectedPageIndexTreeFlightRouteProvider {
    state: Arc<GatewayState>,
}

impl StudioRepoProjectedPageIndexTreeFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self { state }
    }
}

impl std::fmt::Debug for StudioRepoProjectedPageIndexTreeFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioRepoProjectedPageIndexTreeFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl RepoProjectedPageIndexTreeFlightRouteProvider
    for StudioRepoProjectedPageIndexTreeFlightRouteProvider
{
    async fn repo_projected_page_index_tree_batch(
        &self,
        repo_key: &str,
        page_key: &str,
    ) -> Result<AnalysisFlightRouteResponse, Status> {
        let response = run_repo_projected_page_index_tree(
            Arc::clone(&self.state),
            xiuxian_wendao::analyzers::RepoProjectedPageIndexTreeQuery {
                repo_id: repo_key.to_string(),
                page_id: page_key.to_string(),
            },
        )
        .await
        .map_err(studio_api_error_to_tonic_status)?;
        let batch = repo_projected_page_index_tree_batch(&response).map_err(Status::internal)?;
        let metadata =
            repo_projected_page_index_tree_metadata(&response).map_err(Status::internal)?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(metadata))
    }
}

pub(crate) fn repo_projected_page_index_tree_batch(
    response: &RepoProjectedPageIndexTreeResult,
) -> Result<LanceRecordBatch, String> {
    let tree = response
        .tree
        .as_ref()
        .ok_or_else(|| "repo projected page-index tree payload is missing `tree`".to_string())?;
    let roots_json = serde_json::to_string(tree.roots.as_slice())
        .map_err(|error| format!("failed to encode projected page-index roots: {error}"))?;
    let root_count = u64::try_from(tree.root_count)
        .map_err(|error| format!("failed to represent projected page-index root count: {error}"))?;

    LanceRecordBatch::try_new(
        Arc::new(LanceSchema::new(vec![
            LanceField::new("repoId", LanceDataType::Utf8, false),
            LanceField::new("pageId", LanceDataType::Utf8, false),
            LanceField::new("kind", LanceDataType::Utf8, false),
            LanceField::new("path", LanceDataType::Utf8, false),
            LanceField::new("docId", LanceDataType::Utf8, false),
            LanceField::new("title", LanceDataType::Utf8, false),
            LanceField::new("rootCount", LanceDataType::UInt64, false),
            LanceField::new("rootsJson", LanceDataType::Utf8, false),
        ])),
        vec![
            Arc::new(LanceStringArray::from(vec![tree.repo_id.as_str()])),
            Arc::new(LanceStringArray::from(vec![tree.page_id.as_str()])),
            Arc::new(LanceStringArray::from(vec![projection_page_kind_token(
                tree.kind,
            )])),
            Arc::new(LanceStringArray::from(vec![tree.path.as_str()])),
            Arc::new(LanceStringArray::from(vec![tree.doc_id.as_str()])),
            Arc::new(LanceStringArray::from(vec![tree.title.as_str()])),
            Arc::new(LanceUInt64Array::from(vec![root_count])),
            Arc::new(LanceStringArray::from(vec![roots_json.as_str()])),
        ],
    )
    .map_err(|error| error.to_string())
}

pub(crate) fn repo_projected_page_index_tree_metadata(
    response: &RepoProjectedPageIndexTreeResult,
) -> Result<Vec<u8>, String> {
    let tree = response
        .tree
        .as_ref()
        .ok_or_else(|| "repo projected page-index tree payload is missing `tree`".to_string())?;
    serde_json::to_vec(&serde_json::json!({
        "repoId": tree.repo_id,
        "pageId": tree.page_id,
        "kind": projection_page_kind_token(tree.kind),
        "path": tree.path,
        "docId": tree.doc_id,
        "title": tree.title,
        "rootCount": tree.root_count,
    }))
    .map_err(|error| error.to_string())
}

fn studio_api_error_to_tonic_status(error: StudioApiError) -> Status {
    match error.status() {
        axum::http::StatusCode::BAD_REQUEST => Status::invalid_argument(error.error.message),
        axum::http::StatusCode::NOT_FOUND => Status::not_found(error.error.message),
        axum::http::StatusCode::CONFLICT => Status::failed_precondition(error.error.message),
        _ => Status::internal(error.error.message),
    }
}

fn projection_page_kind_token(kind: xiuxian_wendao::analyzers::ProjectionPageKind) -> &'static str {
    match kind {
        xiuxian_wendao::analyzers::ProjectionPageKind::Reference => "reference",
        xiuxian_wendao::analyzers::ProjectionPageKind::HowTo => "howto",
        xiuxian_wendao::analyzers::ProjectionPageKind::Tutorial => "tutorial",
        xiuxian_wendao::analyzers::ProjectionPageKind::Explanation => "explanation",
    }
}

#[cfg(test)]
#[path = "../../../../../../tests/unit/gateway/studio/router/handlers/repo/analysis/projected_page_index_tree_flight.rs"]
mod tests;
