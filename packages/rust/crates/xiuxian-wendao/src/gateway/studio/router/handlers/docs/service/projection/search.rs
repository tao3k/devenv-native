use std::sync::Arc;

use crate::analyzers::{
    DocsSearchQuery, DocsSearchResult, RepoIntelligenceError, build_docs_search,
};
use crate::gateway::studio::router::handlers::docs::service::runtime::run_docs_analysis;
use crate::gateway::studio::router::{GatewayState, StudioApiError};

pub(crate) async fn run_docs_search(
    state: Arc<GatewayState>,
    query: DocsSearchQuery,
) -> Result<DocsSearchResult, StudioApiError> {
    run_docs_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "DOCS_SEARCH_PANIC",
        "Docs search task failed unexpectedly",
        move |analysis| Ok::<_, RepoIntelligenceError>(build_docs_search(&query, &analysis)),
    )
    .await
}
