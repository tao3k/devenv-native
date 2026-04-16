use std::sync::Arc;

use crate::analyzers::{
    DocCoverageQuery, DocCoverageResult, RepoIntelligenceError, RepositoryAnalysisOutput,
    build_doc_coverage,
};
use crate::gateway::studio::router::handlers::repo::shared::execution::with_repo_analysis;
use crate::gateway::studio::router::{GatewayState, StudioApiError};

pub(crate) async fn run_repo_doc_coverage(
    state: Arc<GatewayState>,
    repo_id: String,
    module_id: Option<String>,
) -> Result<DocCoverageResult, StudioApiError> {
    run_repo_analysis_summary(
        Arc::clone(&state),
        repo_id.clone(),
        "REPO_DOC_COVERAGE_PANIC",
        "Repo doc coverage task failed unexpectedly",
        move |analysis| {
            Ok::<_, RepoIntelligenceError>(build_doc_coverage(
                &DocCoverageQuery { repo_id, module_id },
                &analysis,
            ))
        },
    )
    .await
}

async fn run_repo_analysis_summary<T, F>(
    state: Arc<GatewayState>,
    repo_id: String,
    panic_code: &'static str,
    panic_message: &'static str,
    build: F,
) -> Result<T, StudioApiError>
where
    T: Send + 'static,
    F: FnOnce(RepositoryAnalysisOutput) -> Result<T, RepoIntelligenceError> + Send + 'static,
{
    with_repo_analysis(state, repo_id, panic_code, panic_message, build).await
}
