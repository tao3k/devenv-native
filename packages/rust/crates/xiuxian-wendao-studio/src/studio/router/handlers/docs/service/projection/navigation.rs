use std::sync::Arc;

use crate::studio::router::handlers::docs::service::runtime::{
    run_docs_analysis, run_docs_tool_service,
};
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::{
    DocsNavigationOptions, DocsNavigationQuery, DocsNavigationResult, DocsNavigationSearchQuery,
    DocsNavigationSearchResult, build_docs_navigation_search,
};

pub(crate) async fn run_docs_navigation(
    state: Arc<GatewayState>,
    query: DocsNavigationQuery,
) -> Result<DocsNavigationResult, StudioApiError> {
    run_docs_tool_service(
        Arc::clone(&state),
        query.repo_id.clone(),
        "DOCS_NAVIGATION_PANIC",
        "Docs navigation task failed unexpectedly",
        move |service, repository, registry| {
            service.get_navigation_with_options_for_registered_repository(
                &query.page_id,
                &repository,
                registry,
                DocsNavigationOptions {
                    node_id: query.node_id,
                    family_kind: query.family_kind,
                    related_limit: query.related_limit,
                    family_limit: query.family_limit,
                },
            )
        },
    )
    .await
}

pub(crate) async fn run_docs_navigation_search(
    state: Arc<GatewayState>,
    query: DocsNavigationSearchQuery,
) -> Result<DocsNavigationSearchResult, StudioApiError> {
    run_docs_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "DOCS_NAVIGATION_SEARCH_PANIC",
        "Docs navigation search task failed unexpectedly",
        move |analysis| build_docs_navigation_search(&query, &analysis),
    )
    .await
}
