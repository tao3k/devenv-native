use std::sync::Arc;

use crate::analyzers::{RepoIntelligenceError, RepositoryAnalysisOutput};
use crate::gateway::studio::router::handlers::repo::shared::execution::with_repo_analysis;
use crate::gateway::studio::router::{GatewayState, StudioApiError};

pub(crate) async fn run_repo_projected_analysis<T, F>(
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
