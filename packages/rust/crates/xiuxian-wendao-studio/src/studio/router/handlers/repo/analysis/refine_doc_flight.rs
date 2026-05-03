use std::sync::Arc;

use crate::studio::arrow_types::{
    LanceDataType, LanceField, LanceRecordBatch, LanceSchema, LanceStringArray,
};
use async_trait::async_trait;
use tonic::Status;
use xiuxian_wendao_web::transport::{AnalysisFlightRouteResponse, RefineDocFlightRouteProvider};

use crate::studio::router::handlers::repo::command_service::run_refine_entity_doc;
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::{RefineEntityDocRequest, RefineEntityDocResponse};

#[derive(Clone)]
pub(crate) struct StudioRefineDocFlightRouteProvider {
    state: Arc<GatewayState>,
}

impl StudioRefineDocFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self { state }
    }
}

impl std::fmt::Debug for StudioRefineDocFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioRefineDocFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl RefineDocFlightRouteProvider for StudioRefineDocFlightRouteProvider {
    async fn refine_doc_batch(
        &self,
        repo_id: &str,
        entity_id: &str,
        user_hints: Option<&str>,
    ) -> Result<AnalysisFlightRouteResponse, Status> {
        let response = run_refine_entity_doc(
            Arc::clone(&self.state),
            RefineEntityDocRequest {
                repo_id: repo_id.to_string(),
                entity_id: entity_id.to_string(),
                user_hints: user_hints.map(ToString::to_string),
            },
        )
        .await
        .map_err(studio_api_error_to_tonic_status)?;
        let batch = refine_doc_batch(&response).map_err(Status::internal)?;
        let metadata = refine_doc_metadata(&response).map_err(Status::internal)?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(metadata))
    }
}

pub(crate) fn refine_doc_batch(
    response: &RefineEntityDocResponse,
) -> Result<LanceRecordBatch, String> {
    LanceRecordBatch::try_new(
        Arc::new(LanceSchema::new(vec![
            LanceField::new("repoId", LanceDataType::Utf8, false),
            LanceField::new("entityId", LanceDataType::Utf8, false),
            LanceField::new("refinedContent", LanceDataType::Utf8, false),
            LanceField::new("verificationState", LanceDataType::Utf8, false),
        ])),
        vec![
            Arc::new(LanceStringArray::from(vec![response.repo_id.as_str()])),
            Arc::new(LanceStringArray::from(vec![response.entity_id.as_str()])),
            Arc::new(LanceStringArray::from(vec![
                response.refined_content.as_str(),
            ])),
            Arc::new(LanceStringArray::from(vec![
                response.verification_state.as_str(),
            ])),
        ],
    )
    .map_err(|error| error.to_string())
}

pub(crate) fn refine_doc_metadata(response: &RefineEntityDocResponse) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&serde_json::json!({
        "repoId": response.repo_id,
        "entityId": response.entity_id,
        "refinedContent": response.refined_content,
        "verificationState": response.verification_state,
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

#[cfg(test)]
#[path = "../../../../../../tests/unit/gateway/studio/router/handlers/repo/analysis/refine_doc_flight.rs"]
mod tests;
