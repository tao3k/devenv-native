use std::sync::Arc;

use crate::analyzers::{ImportSearchResult, import_fallback_contract};
use crate::gateway::studio::router::handlers::repo::analysis::search::publication::repo_entity_publication_ready;
use crate::gateway::studio::router::{GatewayState, StudioApiError};
use crate::query_core::query_repo_entity_import_results_if_published;

use super::execution::{
    RepoAnalysisFallbackSearchContract, RepoAnalysisSearchSpec, run_fallback_repo_analysis_search,
};

pub(crate) async fn run_repo_import_search(
    state: Arc<GatewayState>,
    repo_id: String,
    package: Option<String>,
    module: Option<String>,
    limit: usize,
) -> Result<ImportSearchResult, StudioApiError> {
    let publication_ready = repo_entity_publication_ready(&state, repo_id.as_str()).await;
    if let Some(result) = query_repo_entity_import_results_if_published(
        &state.studio.search_plane,
        repo_id.as_str(),
        package.clone(),
        module.clone(),
        limit,
        publication_ready,
    )
    .await
    .map_err(|error| {
        StudioApiError::internal(
            "REPO_IMPORT_SEARCH_FAILED",
            "Repo import search task failed",
            Some(error.to_string()),
        )
    })? {
        return Ok(result);
    }

    let fallback = import_fallback_contract(package, module);
    run_fallback_repo_analysis_search(
        Arc::clone(&state),
        repo_id,
        limit,
        RepoAnalysisFallbackSearchContract {
            spec: RepoAnalysisSearchSpec {
                scope: fallback.scope,
                panic_code: "REPO_IMPORT_SEARCH_PANIC",
                panic_message: "Repo import search task failed unexpectedly",
                fuzzy_options: fallback.fuzzy_options,
            },
            fallback,
        },
    )
    .await
}
