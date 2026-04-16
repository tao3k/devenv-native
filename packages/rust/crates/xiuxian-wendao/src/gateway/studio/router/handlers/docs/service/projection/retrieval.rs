use std::sync::Arc;

use crate::analyzers::{
    DocsRetrievalContextOptions, DocsRetrievalContextQuery, DocsRetrievalContextResult,
    DocsRetrievalHitQuery, DocsRetrievalHitResult, DocsRetrievalQuery, DocsRetrievalResult,
    RepoIntelligenceError, build_docs_retrieval, build_docs_retrieval_hit,
};
use crate::gateway::studio::router::handlers::docs::service::runtime::{
    run_docs_analysis, run_docs_tool_service,
};
use crate::gateway::studio::router::{GatewayState, StudioApiError, map_repo_intelligence_error};

pub(crate) async fn run_docs_retrieval(
    state: Arc<GatewayState>,
    query: DocsRetrievalQuery,
) -> Result<DocsRetrievalResult, StudioApiError> {
    run_docs_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "DOCS_RETRIEVAL_PANIC",
        "Docs retrieval task failed unexpectedly",
        move |analysis| Ok::<_, RepoIntelligenceError>(build_docs_retrieval(&query, &analysis)),
    )
    .await
}

pub(crate) async fn run_docs_retrieval_context(
    state: Arc<GatewayState>,
    query: DocsRetrievalContextQuery,
) -> Result<DocsRetrievalContextResult, StudioApiError> {
    run_docs_tool_service(
        Arc::clone(&state),
        query.repo_id.clone(),
        "DOCS_RETRIEVAL_CONTEXT_PANIC",
        "Docs retrieval context task failed unexpectedly",
        move |service, repository, registry| {
            service.get_retrieval_context_with_options_for_registered_repository(
                &query.page_id,
                &repository,
                registry,
                DocsRetrievalContextOptions {
                    node_id: query.node_id,
                    related_limit: query.related_limit,
                },
            )
        },
    )
    .await
}

pub(crate) async fn run_docs_retrieval_hit(
    state: Arc<GatewayState>,
    query: DocsRetrievalHitQuery,
) -> Result<DocsRetrievalHitResult, StudioApiError> {
    let result = run_docs_analysis(
        Arc::clone(&state),
        query.repo.clone(),
        "DOCS_RETRIEVAL_HIT_PANIC",
        "Docs retrieval hit task failed unexpectedly",
        move |analysis| Ok::<_, RepoIntelligenceError>(build_docs_retrieval_hit(&query, &analysis)),
    )
    .await?;
    result.map_err(map_repo_intelligence_error)
}
