use std::sync::Arc;

use async_trait::async_trait;
use xiuxian_wendao_server::transport::{
    AnalysisFlightRouteResponse, CodeAstAnalysisFlightRouteProvider,
    MarkdownAnalysisFlightRouteProvider,
};

use crate::studio::router::GatewayState;
use crate::studio::router::StudioApiError;
use crate::studio::router::retrieval_arrow::build_retrieval_chunks_flight_batch;

use super::{load_code_ast_analysis_response, load_markdown_analysis_response};

#[derive(Clone)]
pub(crate) struct StudioMarkdownAnalysisFlightRouteProvider {
    state: Arc<GatewayState>,
}

impl StudioMarkdownAnalysisFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self { state }
    }
}

impl std::fmt::Debug for StudioMarkdownAnalysisFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioMarkdownAnalysisFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl MarkdownAnalysisFlightRouteProvider for StudioMarkdownAnalysisFlightRouteProvider {
    async fn markdown_analysis_batch(
        &self,
        path: &str,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        let response = load_markdown_analysis_response(self.state.as_ref(), path)
            .await
            .map_err(|error| map_studio_api_error(&error))?;
        let batch = build_retrieval_chunks_flight_batch(response.retrieval_atoms.as_slice())?;
        let metadata = serde_json::to_vec(&serde_json::json!({
            "path": response.path,
            "documentHash": response.document_hash,
            "nodeCount": response.node_count,
            "edgeCount": response.edge_count,
            "nodes": response.nodes,
            "edges": response.edges,
            "projections": response.projections,
            "diagnostics": response.diagnostics,
        }))
        .map_err(|error| error.to_string())?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(metadata))
    }
}

#[derive(Clone)]
pub(crate) struct StudioCodeAstAnalysisFlightRouteProvider {
    state: Arc<GatewayState>,
}

impl StudioCodeAstAnalysisFlightRouteProvider {
    #[must_use]
    pub(crate) fn new(state: Arc<GatewayState>) -> Self {
        Self { state }
    }
}

impl std::fmt::Debug for StudioCodeAstAnalysisFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioCodeAstAnalysisFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl CodeAstAnalysisFlightRouteProvider for StudioCodeAstAnalysisFlightRouteProvider {
    async fn code_ast_analysis_batch(
        &self,
        path: &str,
        repo_id: &str,
        line_hint: Option<usize>,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        let response =
            load_code_ast_analysis_response(self.state.as_ref(), path, repo_id, line_hint)
                .await
                .map_err(|error| map_studio_api_error(&error))?;
        let batch = build_retrieval_chunks_flight_batch(response.retrieval_atoms.as_slice())?;
        let metadata = serde_json::to_vec(&serde_json::json!({
            "repoId": response.repo_id,
            "path": response.path,
            "language": response.language,
            "nodeCount": response.nodes.len(),
            "edgeCount": response.edges.len(),
            "nodes": response.nodes,
            "edges": response.edges,
            "projections": response.projections,
            "focusNodeId": response.focus_node_id,
            "diagnostics": response.diagnostics,
        }))
        .map_err(|error| error.to_string())?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(metadata))
    }
}

fn map_studio_api_error(error: &StudioApiError) -> String {
    error
        .error
        .details
        .clone()
        .unwrap_or_else(|| format!("{}: {}", error.code(), error.error.message))
}
