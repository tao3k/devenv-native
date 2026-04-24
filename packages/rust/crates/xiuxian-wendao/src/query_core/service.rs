//! Thin query-core facade helpers for internal Wendao callers.

use std::collections::{BTreeSet, HashSet};
use std::future::Future;
use std::hash::BuildHasher;
use std::pin::Pin;
use std::sync::Arc;

use crate::analyzers::{
    ExampleSearchResult, ImportSearchQuery, ImportSearchResult, ModuleSearchResult,
    SymbolSearchResult,
};
use crate::link_graph::LinkGraphIndex;
use crate::query_core::WendaoExecutionContext;
use crate::query_core::execute::{
    LinkGraphNeighborsBackend, SearchPlaneRetrievalBackend, execute_column_mask,
    execute_graph_neighbors, execute_payload_fetch, execute_vector_search,
};
use crate::query_core::graph::{WendaoGraphProjection, graph_projection_from_relation};
use crate::query_core::operators::{
    ColumnMaskOp, ColumnMaskPredicate, GraphDirection, GraphNeighborsOp, PayloadFetchOp,
    RetrievalCorpus, VectorSearchOp,
};
use crate::query_core::telemetry::WendaoExplainSink;
use crate::query_core::types::{WendaoQueryCoreError, WendaoRelation};
use crate::search::{
    SearchPlaneService, search_repo_entity_example_results, search_repo_entity_import_results,
    search_repo_entity_module_results, search_repo_entity_symbol_results,
};

/// Result of one repo-scoped code query routed through the Phase-1 query core.
pub struct RepoCodeQueryRelation {
    /// The corpus that actually produced the returned relation.
    pub corpus: RetrievalCorpus,
    /// The returned Arrow-native relation.
    pub relation: WendaoRelation,
}

type RepoEntityQueryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, WendaoQueryCoreError>> + Send + 'a>>;

/// Immutable repo-scoped retrieval parameters for one query-core lookup.
#[derive(Clone, Debug, Default)]
pub struct RepoRetrievalQuery {
    /// Stable repo identifier.
    pub repo_id: String,
    /// User-provided search term.
    pub search_term: String,
    /// Optional language filters.
    pub language_filters: BTreeSet<String>,
    /// Optional kind filters.
    pub kind_filters: BTreeSet<String>,
    /// Maximum number of rows to return.
    pub limit: usize,
}

impl RepoRetrievalQuery {
    /// Build one repo-scoped retrieval query from generic set-like filters.
    pub fn new<LS, KS>(
        repo_id: impl Into<String>,
        search_term: impl Into<String>,
        language_filters: &HashSet<String, LS>,
        kind_filters: &HashSet<String, KS>,
        limit: usize,
    ) -> Self
    where
        LS: BuildHasher,
        KS: BuildHasher,
    {
        Self {
            repo_id: repo_id.into(),
            search_term: search_term.into(),
            language_filters: language_filters.iter().cloned().collect::<BTreeSet<_>>(),
            kind_filters: kind_filters.iter().cloned().collect::<BTreeSet<_>>(),
            limit,
        }
    }

    fn language_filter_set(&self) -> HashSet<String> {
        self.language_filters
            .iter()
            .cloned()
            .collect::<HashSet<_>>()
    }

    fn kind_filter_set(&self) -> HashSet<String> {
        self.kind_filters.iter().cloned().collect::<HashSet<_>>()
    }
}

/// Repo-code lane policy that decides which corpora may execute.
#[derive(Clone, Copy, Debug, Default)]
pub struct RepoCodeQueryPolicy {
    /// Whether the repo-entity lane may execute first.
    pub allow_entity: bool,
    /// Whether the repo-content lane may execute as fallback.
    pub allow_content: bool,
}

/// Combined repo-code query-core request.
#[derive(Clone, Debug, Default)]
pub struct RepoCodeQueryRequest {
    /// Shared repo-scoped retrieval parameters.
    pub query: RepoRetrievalQuery,
    /// Repo-code lane routing policy.
    pub policy: RepoCodeQueryPolicy,
}

impl RepoCodeQueryRequest {
    /// Build one repo-code request from set-like filters and lane policy.
    pub fn new<LS, KS>(
        repo_id: impl Into<String>,
        search_term: impl Into<String>,
        language_filters: &HashSet<String, LS>,
        kind_filters: &HashSet<String, KS>,
        allow_entity: bool,
        allow_content: bool,
        limit: usize,
    ) -> Self
    where
        LS: BuildHasher,
        KS: BuildHasher,
    {
        Self {
            query: RepoRetrievalQuery::new(
                repo_id,
                search_term,
                language_filters,
                kind_filters,
                limit,
            ),
            policy: RepoCodeQueryPolicy {
                allow_entity,
                allow_content,
            },
        }
    }
}

#[derive(Clone, Copy)]
/// Typed contract for one repo-entity fast-path query surface.
pub struct RepoEntityTypedResultsContract<T> {
    execute:
        for<'a> fn(&'a SearchPlaneService, &'a str, &'a str, usize) -> RepoEntityQueryFuture<'a, T>,
}

/// Execute a repo-scoped content query through the Phase-1 query core.
///
/// # Errors
///
/// Returns [`WendaoQueryCoreError`] when the retrieval adapter or execution layer fails.
pub async fn query_repo_content_relation(
    search_plane: &SearchPlaneService,
    query: &RepoRetrievalQuery,
    explain_sink: Option<Arc<dyn WendaoExplainSink>>,
) -> Result<WendaoRelation, WendaoQueryCoreError> {
    let ctx = build_repo_content_context(search_plane, explain_sink);
    let relation = execute_vector_search(
        &ctx,
        &VectorSearchOp {
            corpus: RetrievalCorpus::RepoContent,
            repo_id: query.repo_id.clone(),
            search_term: query.search_term.clone(),
            language_filters: query.language_filter_set(),
            kind_filters: HashSet::new(),
            limit: query.limit,
        },
    )
    .await?;
    finalize_repo_retrieval_relation(&ctx, query.repo_id.as_str(), relation, query.limit).await
}

/// Execute a repo-scoped code query through the entity-first, content-fallback policy.
///
/// # Errors
///
/// Returns [`WendaoQueryCoreError`] when the retrieval adapter or execution layer fails.
pub async fn query_repo_code_relation(
    search_plane: &SearchPlaneService,
    request: &RepoCodeQueryRequest,
    explain_sink: Option<Arc<dyn WendaoExplainSink>>,
) -> Result<RepoCodeQueryRelation, WendaoQueryCoreError> {
    if request.policy.allow_entity {
        let entity_relation =
            query_repo_entity_relation(search_plane, &request.query, explain_sink.clone()).await?;
        if entity_relation.row_count() > 0 {
            return Ok(RepoCodeQueryRelation {
                corpus: RetrievalCorpus::RepoEntity,
                relation: entity_relation,
            });
        }
        if !request.policy.allow_content {
            return Ok(RepoCodeQueryRelation {
                corpus: RetrievalCorpus::RepoEntity,
                relation: entity_relation,
            });
        }
    }

    let content_allowed =
        request.query.kind_filters.is_empty() || request.query.kind_filters.contains("file");
    if !request.policy.allow_content || !content_allowed {
        return Ok(RepoCodeQueryRelation {
            corpus: RetrievalCorpus::RepoEntity,
            relation: WendaoRelation::new(xiuxian_db_store::retrieval_result_schema(), Vec::new()),
        });
    }

    let content_relation =
        query_repo_content_relation(search_plane, &request.query, explain_sink).await?;
    Ok(RepoCodeQueryRelation {
        corpus: RetrievalCorpus::RepoContent,
        relation: content_relation,
    })
}

/// Execute a repo-scoped entity query through the Phase-1 query core.
///
/// # Errors
///
/// Returns [`WendaoQueryCoreError`] when the retrieval adapter or execution layer fails.
pub async fn query_repo_entity_relation(
    search_plane: &SearchPlaneService,
    query: &RepoRetrievalQuery,
    explain_sink: Option<Arc<dyn WendaoExplainSink>>,
) -> Result<WendaoRelation, WendaoQueryCoreError> {
    let ctx = build_repo_content_context(search_plane, explain_sink);
    let relation = execute_vector_search(
        &ctx,
        &VectorSearchOp {
            corpus: RetrievalCorpus::RepoEntity,
            repo_id: query.repo_id.clone(),
            search_term: query.search_term.clone(),
            language_filters: query.language_filter_set(),
            kind_filters: query.kind_filter_set(),
            limit: query.limit,
        },
    )
    .await?;
    finalize_repo_retrieval_relation(&ctx, query.repo_id.as_str(), relation, query.limit).await
}

/// Execute a repo-entity module query when publication is ready.
///
/// # Errors
///
/// Returns [`WendaoQueryCoreError`] when the typed repo-entity query fails.
pub async fn query_repo_entity_module_results_if_published(
    search_plane: &SearchPlaneService,
    repo_id: &str,
    query: &str,
    limit: usize,
    publication_ready: bool,
) -> Result<Option<ModuleSearchResult>, WendaoQueryCoreError> {
    query_repo_entity_results_if_published(
        search_plane,
        repo_id,
        query,
        limit,
        publication_ready,
        repo_entity_module_results_contract(),
    )
    .await
}

/// Execute a repo-entity symbol query when publication is ready.
///
/// # Errors
///
/// Returns [`WendaoQueryCoreError`] when the typed repo-entity query fails.
pub async fn query_repo_entity_symbol_results_if_published(
    search_plane: &SearchPlaneService,
    repo_id: &str,
    query: &str,
    limit: usize,
    publication_ready: bool,
) -> Result<Option<SymbolSearchResult>, WendaoQueryCoreError> {
    query_repo_entity_results_if_published(
        search_plane,
        repo_id,
        query,
        limit,
        publication_ready,
        repo_entity_symbol_results_contract(),
    )
    .await
}

/// Execute a repo-entity example query when publication is ready.
///
/// # Errors
///
/// Returns [`WendaoQueryCoreError`] when the typed repo-entity query fails.
pub async fn query_repo_entity_example_results_if_published(
    search_plane: &SearchPlaneService,
    repo_id: &str,
    query: &str,
    limit: usize,
    publication_ready: bool,
) -> Result<Option<ExampleSearchResult>, WendaoQueryCoreError> {
    query_repo_entity_results_if_published(
        search_plane,
        repo_id,
        query,
        limit,
        publication_ready,
        repo_entity_example_results_contract(),
    )
    .await
}

/// Execute a repo-entity import query when publication is ready.
///
/// # Errors
///
/// Returns [`WendaoQueryCoreError`] when the typed repo-entity query fails.
pub async fn query_repo_entity_import_results_if_published(
    search_plane: &SearchPlaneService,
    repo_id: &str,
    package: Option<String>,
    module: Option<String>,
    limit: usize,
    publication_ready: bool,
) -> Result<Option<ImportSearchResult>, WendaoQueryCoreError> {
    if !publication_ready {
        return Ok(None);
    }

    search_repo_entity_import_results(
        search_plane,
        &ImportSearchQuery {
            repo_id: repo_id.to_string(),
            package,
            module,
            limit,
        },
    )
    .await
    .map(Some)
    .map_err(|error| WendaoQueryCoreError::Backend(error.to_string()))
}

/// Execute one typed repo-entity query when publication is ready.
///
/// # Errors
///
/// Returns [`WendaoQueryCoreError`] when the typed repo-entity query fails.
pub async fn query_repo_entity_results_if_published<T>(
    search_plane: &SearchPlaneService,
    repo_id: &str,
    query: &str,
    limit: usize,
    publication_ready: bool,
    contract: RepoEntityTypedResultsContract<T>,
) -> Result<Option<T>, WendaoQueryCoreError> {
    query_repo_entity_typed_results_if_published(
        publication_ready,
        (contract.execute)(search_plane, repo_id, query, limit),
    )
    .await
}

/// Repo-entity module fast-path contract.
#[must_use]
pub fn repo_entity_module_results_contract() -> RepoEntityTypedResultsContract<ModuleSearchResult> {
    RepoEntityTypedResultsContract {
        execute: |search_plane, repo_id, query, limit| {
            Box::pin(async move {
                search_repo_entity_module_results(search_plane, repo_id, query, limit)
                    .await
                    .map_err(|error| WendaoQueryCoreError::Backend(error.to_string()))
            })
        },
    }
}

/// Repo-entity symbol fast-path contract.
#[must_use]
pub fn repo_entity_symbol_results_contract() -> RepoEntityTypedResultsContract<SymbolSearchResult> {
    RepoEntityTypedResultsContract {
        execute: |search_plane, repo_id, query, limit| {
            Box::pin(async move {
                search_repo_entity_symbol_results(search_plane, repo_id, query, limit)
                    .await
                    .map_err(|error| WendaoQueryCoreError::Backend(error.to_string()))
            })
        },
    }
}

/// Repo-entity example fast-path contract.
#[must_use]
pub fn repo_entity_example_results_contract() -> RepoEntityTypedResultsContract<ExampleSearchResult>
{
    RepoEntityTypedResultsContract {
        execute: |search_plane, repo_id, query, limit| {
            Box::pin(async move {
                search_repo_entity_example_results(search_plane, repo_id, query, limit)
                    .await
                    .map_err(|error| WendaoQueryCoreError::Backend(error.to_string()))
            })
        },
    }
}

/// Execute a graph-neighbor query through the Phase-1 query core.
///
/// # Errors
///
/// Returns [`WendaoQueryCoreError`] when the graph adapter or execution layer fails.
pub async fn query_graph_neighbors_relation(
    index: Arc<LinkGraphIndex>,
    node_id: &str,
    direction: GraphDirection,
    hops: usize,
    limit: usize,
    explain_sink: Option<Arc<dyn WendaoExplainSink>>,
) -> Result<WendaoRelation, WendaoQueryCoreError> {
    let ctx = build_graph_context(index, explain_sink);
    execute_graph_neighbors(
        &ctx,
        &GraphNeighborsOp {
            node_id: node_id.to_string(),
            direction,
            hops,
            limit,
        },
    )
    .await
}

/// Execute a graph-neighbor query and project it into the query-core-native graph shape.
///
/// # Errors
///
/// Returns [`WendaoQueryCoreError`] when graph execution or projection fails.
pub async fn query_graph_neighbors_projection(
    index: Arc<LinkGraphIndex>,
    node_id: &str,
    direction: GraphDirection,
    hops: usize,
    limit: usize,
    explain_sink: Option<Arc<dyn WendaoExplainSink>>,
) -> Result<WendaoGraphProjection, WendaoQueryCoreError> {
    let relation = query_graph_neighbors_relation(
        Arc::clone(&index),
        node_id,
        direction,
        hops,
        limit,
        explain_sink,
    )
    .await?;
    graph_projection_from_relation(index.as_ref(), &relation)
}

fn build_repo_content_context(
    search_plane: &SearchPlaneService,
    explain_sink: Option<Arc<dyn WendaoExplainSink>>,
) -> WendaoExecutionContext {
    let mut ctx = WendaoExecutionContext::default().with_retrieval_backend(Arc::new(
        SearchPlaneRetrievalBackend::new(Arc::new(search_plane.clone())),
    ));
    if let Some(explain_sink) = explain_sink {
        ctx = ctx.with_explain_sink(explain_sink);
    }
    ctx
}

fn build_graph_context(
    index: Arc<LinkGraphIndex>,
    explain_sink: Option<Arc<dyn WendaoExplainSink>>,
) -> WendaoExecutionContext {
    let mut ctx = WendaoExecutionContext::default()
        .with_graph_backend(Arc::new(LinkGraphNeighborsBackend::new(index)));
    if let Some(explain_sink) = explain_sink {
        ctx = ctx.with_explain_sink(explain_sink);
    }
    ctx
}

async fn finalize_repo_retrieval_relation(
    ctx: &WendaoExecutionContext,
    repo_id: &str,
    relation: WendaoRelation,
    limit: usize,
) -> Result<WendaoRelation, WendaoQueryCoreError> {
    let masked = execute_column_mask(
        ctx,
        &ColumnMaskOp {
            relation,
            predicates: vec![ColumnMaskPredicate::RepoEquals(repo_id.to_string())],
            limit: Some(limit),
        },
    )?;
    execute_payload_fetch(
        ctx,
        &PayloadFetchOp {
            relation: masked,
            columns: xiuxian_db_store::retrieval_result_columns(),
            ids: None,
        },
    )
    .await
}

async fn query_repo_entity_typed_results_if_published<T, Fut>(
    publication_ready: bool,
    future: Fut,
) -> Result<Option<T>, WendaoQueryCoreError>
where
    Fut: Future<Output = Result<T, WendaoQueryCoreError>>,
{
    if !publication_ready {
        return Ok(None);
    }

    future.await.map(Some)
}
