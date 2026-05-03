use std::sync::Arc;

use crate::studio::router::handlers::docs::service::runtime::run_docs_analysis;
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::{
    DocsSearchQuery, DocsSearchResult, RepoIntelligenceError, build_docs_search,
};

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
