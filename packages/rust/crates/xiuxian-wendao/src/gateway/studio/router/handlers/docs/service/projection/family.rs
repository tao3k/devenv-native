use std::sync::Arc;

use crate::analyzers::{
    DocsFamilyClusterQuery, DocsFamilyClusterResult, DocsFamilyContextQuery,
    DocsFamilyContextResult, DocsFamilySearchQuery, DocsFamilySearchResult, RepoIntelligenceError,
    build_docs_family_cluster, build_docs_family_context, build_docs_family_search,
};
use crate::gateway::studio::router::handlers::docs::service::runtime::run_docs_analysis;
use crate::gateway::studio::router::{GatewayState, StudioApiError};

pub(crate) async fn run_docs_family_context(
    state: Arc<GatewayState>,
    query: DocsFamilyContextQuery,
) -> Result<DocsFamilyContextResult, StudioApiError> {
    run_docs_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "DOCS_FAMILY_CONTEXT_PANIC",
        "Docs family context task failed unexpectedly",
        move |analysis| build_docs_family_context(&query, &analysis),
    )
    .await
}

pub(crate) async fn run_docs_family_search(
    state: Arc<GatewayState>,
    query: DocsFamilySearchQuery,
) -> Result<DocsFamilySearchResult, StudioApiError> {
    run_docs_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "DOCS_FAMILY_SEARCH_PANIC",
        "Docs family search task failed unexpectedly",
        move |analysis| Ok::<_, RepoIntelligenceError>(build_docs_family_search(&query, &analysis)),
    )
    .await
}

pub(crate) async fn run_docs_family_cluster(
    state: Arc<GatewayState>,
    query: DocsFamilyClusterQuery,
) -> Result<DocsFamilyClusterResult, StudioApiError> {
    run_docs_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "DOCS_FAMILY_CLUSTER_PANIC",
        "Docs family cluster task failed unexpectedly",
        move |analysis| build_docs_family_cluster(&query, &analysis),
    )
    .await
}
