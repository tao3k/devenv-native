use std::sync::Arc;

use crate::studio::router::GatewayState;
use crate::studio::router::StudioApiError;
use crate::studio::router::handlers::repo::analysis_support::execution::{
    with_repo_analysis, with_repository,
};
use xiuxian_wendao::analyzers::{
    DocsToolService, PluginRegistry, RegisteredRepository, RepoIntelligenceError,
    RepositoryAnalysisOutput,
};

pub(super) async fn run_docs_analysis<T, F>(
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

pub(super) async fn run_docs_tool_service<T, F>(
    state: Arc<GatewayState>,
    repo_id: String,
    panic_code: &'static str,
    panic_message: &'static str,
    build: F,
) -> Result<T, StudioApiError>
where
    T: Send + 'static,
    F: FnOnce(
            DocsToolService,
            RegisteredRepository,
            &PluginRegistry,
        ) -> Result<T, RepoIntelligenceError>
        + Send
        + 'static,
{
    let plugin_registry = Arc::clone(&state.studio.plugin_registry);
    with_repository(
        state,
        repo_id,
        panic_code,
        panic_message,
        true,
        move |repository, cwd| {
            let service = DocsToolService::from_project_root(cwd, repository.id.clone());
            build(service, repository, plugin_registry.as_ref())
        },
    )
    .await
}
