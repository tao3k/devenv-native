use std::sync::Arc;

use crate::studio::router::handlers::docs::service::runtime::run_docs_tool_service;
use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::{
    DocsPageIndexTreeQuery, DocsPageIndexTreeResult, DocsPageQuery, DocsPageResult,
};

pub(crate) async fn run_docs_page(
    state: Arc<GatewayState>,
    query: DocsPageQuery,
) -> Result<DocsPageResult, StudioApiError> {
    run_docs_tool_service(
        Arc::clone(&state),
        query.repo_id.clone(),
        "DOCS_PAGE_PANIC",
        "Docs page task failed unexpectedly",
        move |service, repository, registry| {
            service.get_document_for_registered_repository(&query.page_id, &repository, registry)
        },
    )
    .await
}

pub(crate) async fn run_docs_page_index_tree(
    state: Arc<GatewayState>,
    query: DocsPageIndexTreeQuery,
) -> Result<DocsPageIndexTreeResult, StudioApiError> {
    run_docs_tool_service(
        Arc::clone(&state),
        query.repo_id.clone(),
        "DOCS_PAGE_INDEX_TREE_PANIC",
        "Docs page-index tree task failed unexpectedly",
        move |service, repository, registry| {
            service.get_page_index_tree_for_registered_repository(
                &query.page_id,
                &repository,
                registry,
            )
        },
    )
    .await
}
