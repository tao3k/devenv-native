use std::sync::Arc;

use crate::studio::router::{GatewayState, StudioApiError};
use xiuxian_wendao::analyzers::{
    DocsPlannerItemQuery, DocsPlannerItemResult, DocsPlannerQueueQuery, DocsPlannerQueueResult,
    DocsPlannerRankQuery, DocsPlannerRankResult, DocsPlannerSearchQuery, DocsPlannerSearchResult,
    DocsPlannerWorksetQuery, DocsPlannerWorksetResult, RepoIntelligenceError,
    build_docs_planner_item, build_docs_planner_queue, build_docs_planner_rank,
    build_docs_planner_search, build_docs_planner_workset,
};

use crate::studio::router::handlers::docs::service::runtime::run_docs_analysis;

pub(crate) async fn run_docs_planner_item(
    state: Arc<GatewayState>,
    query: DocsPlannerItemQuery,
) -> Result<DocsPlannerItemResult, StudioApiError> {
    run_docs_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "DOCS_PLANNER_ITEM_PANIC",
        "Docs planner item task failed unexpectedly",
        move |analysis| build_docs_planner_item(&query, &analysis),
    )
    .await
}

pub(crate) async fn run_docs_planner_search(
    state: Arc<GatewayState>,
    query: DocsPlannerSearchQuery,
) -> Result<DocsPlannerSearchResult, StudioApiError> {
    run_docs_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "DOCS_PLANNER_SEARCH_PANIC",
        "Docs planner search task failed unexpectedly",
        move |analysis| {
            Ok::<_, RepoIntelligenceError>(build_docs_planner_search(&query, &analysis))
        },
    )
    .await
}

pub(crate) async fn run_docs_planner_queue(
    state: Arc<GatewayState>,
    query: DocsPlannerQueueQuery,
) -> Result<DocsPlannerQueueResult, StudioApiError> {
    run_docs_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "DOCS_PLANNER_QUEUE_PANIC",
        "Docs planner queue task failed unexpectedly",
        move |analysis| Ok::<_, RepoIntelligenceError>(build_docs_planner_queue(&query, &analysis)),
    )
    .await
}

pub(crate) async fn run_docs_planner_rank(
    state: Arc<GatewayState>,
    query: DocsPlannerRankQuery,
) -> Result<DocsPlannerRankResult, StudioApiError> {
    run_docs_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "DOCS_PLANNER_RANK_PANIC",
        "Docs planner rank task failed unexpectedly",
        move |analysis| Ok::<_, RepoIntelligenceError>(build_docs_planner_rank(&query, &analysis)),
    )
    .await
}

pub(crate) async fn run_docs_planner_workset(
    state: Arc<GatewayState>,
    query: DocsPlannerWorksetQuery,
) -> Result<DocsPlannerWorksetResult, StudioApiError> {
    run_docs_analysis(
        Arc::clone(&state),
        query.repo_id.clone(),
        "DOCS_PLANNER_WORKSET_PANIC",
        "Docs planner workset task failed unexpectedly",
        move |analysis| build_docs_planner_workset(&query, &analysis),
    )
    .await
}
