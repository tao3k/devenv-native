//! Provider adapters for the repo search Flight host.

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tonic::Status;
use xiuxian_db_store::LanceRecordBatch;
use xiuxian_wendao_runtime::transport::{
    AnalysisFlightRouteResponse, GraphNeighborsFlightRouteProvider,
    GraphNeighborsFlightRouteResponse, RepoProjectedPageIndexTreeFlightRouteProvider,
    RepoProjectedRetrievalContextFlightRouteProvider, RepoSearchFlightRequest,
    RepoSearchFlightRouteProvider, RerankScoreWeights, WendaoFlightRouteProviders,
    WendaoFlightService,
};

use crate::LinkGraphIndex;
use crate::analyzers::{
    ProjectedPageIndexNode, ProjectedPageIndexNodeContext, ProjectedPageIndexNodeHit,
    ProjectedPageIndexTree, ProjectedPageRecord, ProjectedRetrievalHit, ProjectedRetrievalHitKind,
    RepoIntelligenceError, RepoProjectedPageIndexTreeQuery, RepoProjectedPageIndexTreeResult,
    RepoProjectedRetrievalContextQuery, RepoProjectedRetrievalContextResult,
    RepositoryAnalysisOutput, build_projected_page_index_trees, build_projected_pages,
    build_repo_projected_page_index_tree, build_repo_projected_retrieval_context,
    repo_projected_page_index_tree_from_config, repo_projected_retrieval_context_from_config,
};
use crate::query_core::{GraphDirection, query_graph_neighbors_projection};
use crate::search::SearchPlaneService;
use crate::search::repo_search::search_repo_content_batch;

#[path = "batches.rs"]
mod batches;

use batches::{
    graph_neighbors_projection_batch, repo_projected_page_index_tree_batch,
    repo_projected_page_index_tree_metadata, repo_projected_retrieval_context_batch,
    repo_projected_retrieval_context_metadata,
};

/// Runtime parts needed to build the live repo-search Flight route provider.
pub(super) struct SearchStrategyFlowFlightHostParts {
    pub(super) repo_id: String,
    pub(super) project_root: PathBuf,
    pub(super) config_path: Option<PathBuf>,
    pub(super) search_plane: Arc<SearchPlaneService>,
    pub(super) link_graph_index: Arc<LinkGraphIndex>,
    pub(super) bootstrap_analysis: Option<Arc<RepositoryAnalysisOutput>>,
}

struct RepoSearchFlightHostProvider {
    repo_id: String,
    project_root: PathBuf,
    config_path: Option<PathBuf>,
    search_plane: Arc<SearchPlaneService>,
    link_graph_index: Arc<LinkGraphIndex>,
    bootstrap_analysis: Option<Arc<RepositoryAnalysisOutput>>,
    bootstrap_projection_cache: Arc<Mutex<Option<Arc<BootstrapProjectionCache>>>>,
}

struct BootstrapProjectionCache {
    pages: Vec<ProjectedPageRecord>,
    pages_by_id: HashMap<String, ProjectedPageRecord>,
    trees_by_id: HashMap<String, ProjectedPageIndexTree>,
}

impl fmt::Debug for RepoSearchFlightHostProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RepoSearchFlightHostProvider")
            .field("repo_id", &self.repo_id)
            .finish_non_exhaustive()
    }
}

impl RepoSearchFlightHostProvider {
    fn bootstrap_projection_cache(&self) -> Result<Option<Arc<BootstrapProjectionCache>>, Status> {
        let Some(analysis) = self.bootstrap_analysis.as_deref() else {
            return Ok(None);
        };
        let mut guard = self.bootstrap_projection_cache.lock().map_err(|error| {
            Status::internal(format!("lock bootstrap projection cache: {error}"))
        })?;
        if let Some(cache) = guard.as_ref() {
            return Ok(Some(Arc::clone(cache)));
        }

        let cache = Arc::new(
            BootstrapProjectionCache::build(analysis)
                .map_err(|error| Status::internal(error.to_string()))?,
        );
        *guard = Some(Arc::clone(&cache));
        Ok(Some(cache))
    }
}

impl BootstrapProjectionCache {
    fn build(analysis: &RepositoryAnalysisOutput) -> Result<Self, RepoIntelligenceError> {
        let pages = build_projected_pages(analysis);
        let pages_by_id = pages
            .iter()
            .map(|page| (page.page_id.clone(), page.clone()))
            .collect::<HashMap<_, _>>();
        let trees_by_id = build_projected_page_index_trees(analysis)?
            .into_iter()
            .map(|tree| (tree.page_id.clone(), tree))
            .collect::<HashMap<_, _>>();

        Ok(Self {
            pages,
            pages_by_id,
            trees_by_id,
        })
    }

    fn page_index_tree(
        &self,
        query: &RepoProjectedPageIndexTreeQuery,
    ) -> Result<RepoProjectedPageIndexTreeResult, RepoIntelligenceError> {
        let tree = self
            .trees_by_id
            .get(query.page_id.as_str())
            .cloned()
            .ok_or_else(|| RepoIntelligenceError::UnknownProjectedPage {
                repo_id: query.repo_id.clone().into(),
                page_id: query.page_id.clone().into(),
            })?;

        Ok(RepoProjectedPageIndexTreeResult {
            repo_id: query.repo_id.clone(),
            tree: Some(tree),
        })
    }

    fn retrieval_context(
        &self,
        query: &RepoProjectedRetrievalContextQuery,
    ) -> Result<RepoProjectedRetrievalContextResult, RepoIntelligenceError> {
        let page = self
            .pages_by_id
            .get(query.page_id.as_str())
            .cloned()
            .ok_or_else(|| RepoIntelligenceError::UnknownProjectedPage {
                repo_id: query.repo_id.clone().into(),
                page_id: query.page_id.clone().into(),
            })?;
        let center = ProjectedRetrievalHit {
            kind: ProjectedRetrievalHitKind::Page,
            page: page.clone(),
            node: None,
        };
        let related_pages = self.related_pages(&page, query.related_limit);
        let node_context = query
            .node_id
            .as_deref()
            .map(|node_id| self.node_context(&page, node_id))
            .transpose()?;

        Ok(RepoProjectedRetrievalContextResult {
            repo_id: query.repo_id.clone(),
            center,
            related_pages,
            node_context,
        })
    }

    fn related_pages(&self, page: &ProjectedPageRecord, limit: usize) -> Vec<ProjectedPageRecord> {
        let mut matches = self
            .pages
            .iter()
            .filter(|candidate| candidate.page_id != page.page_id)
            .filter_map(|candidate| {
                let score = relation_score(page, candidate);
                (score > 0).then_some((score, candidate.clone()))
            })
            .collect::<Vec<_>>();
        matches.sort_by(
            |(left_score, left_page): &(usize, ProjectedPageRecord),
             (right_score, right_page): &(usize, ProjectedPageRecord)| {
                right_score
                    .cmp(left_score)
                    .then_with(|| left_page.title.cmp(&right_page.title))
                    .then_with(|| left_page.page_id.cmp(&right_page.page_id))
            },
        );
        matches
            .into_iter()
            .take(limit)
            .map(|(_, page)| page)
            .collect()
    }

    fn node_context(
        &self,
        page: &ProjectedPageRecord,
        node_id: &str,
    ) -> Result<ProjectedPageIndexNodeContext, RepoIntelligenceError> {
        let tree = self.trees_by_id.get(page.page_id.as_str()).ok_or_else(|| {
            RepoIntelligenceError::UnknownProjectedPage {
                repo_id: page.repo_id.clone().into(),
                page_id: page.page_id.clone().into(),
            }
        })?;
        let raw = find_node_context(tree.roots.as_slice(), node_id, &[]).ok_or_else(|| {
            RepoIntelligenceError::UnknownProjectedPageIndexNode {
                repo_id: page.repo_id.clone().into(),
                page_id: page.page_id.clone().into(),
                node_id: node_id.to_string().into(),
            }
        })?;

        Ok(ProjectedPageIndexNodeContext {
            ancestors: raw
                .ancestors
                .into_iter()
                .map(|node| page_index_node_hit(page, node))
                .collect(),
            previous_sibling: raw
                .previous_sibling
                .map(|node| page_index_node_hit(page, node)),
            next_sibling: raw.next_sibling.map(|node| page_index_node_hit(page, node)),
            children: raw
                .children
                .into_iter()
                .map(|node| page_index_node_hit(page, node))
                .collect(),
        })
    }
}

struct RawNodeContext<'a> {
    ancestors: Vec<&'a ProjectedPageIndexNode>,
    previous_sibling: Option<&'a ProjectedPageIndexNode>,
    next_sibling: Option<&'a ProjectedPageIndexNode>,
    children: Vec<&'a ProjectedPageIndexNode>,
}

fn relation_score(page: &ProjectedPageRecord, candidate: &ProjectedPageRecord) -> usize {
    shared_count(page.module_ids.as_slice(), candidate.module_ids.as_slice())
        + shared_count(page.symbol_ids.as_slice(), candidate.symbol_ids.as_slice())
        + shared_count(page.doc_ids.as_slice(), candidate.doc_ids.as_slice())
        + shared_count(
            page.example_ids.as_slice(),
            candidate.example_ids.as_slice(),
        )
}

fn shared_count(left: &[String], right: &[String]) -> usize {
    left.iter()
        .filter(|item| right.iter().any(|candidate| candidate == *item))
        .count()
}

fn find_node_context<'a>(
    nodes: &'a [ProjectedPageIndexNode],
    node_id: &str,
    ancestors: &[&'a ProjectedPageIndexNode],
) -> Option<RawNodeContext<'a>> {
    for (index, node) in nodes.iter().enumerate() {
        if node.node_id == node_id {
            return Some(RawNodeContext {
                ancestors: ancestors.to_vec(),
                previous_sibling: index.checked_sub(1).and_then(|left| nodes.get(left)),
                next_sibling: nodes.get(index + 1),
                children: node.children.iter().collect(),
            });
        }
        let mut child_ancestors = ancestors.to_vec();
        child_ancestors.push(node);
        if let Some(context) =
            find_node_context(node.children.as_slice(), node_id, &child_ancestors)
        {
            return Some(context);
        }
    }
    None
}

fn page_index_node_hit(
    page: &ProjectedPageRecord,
    node: &ProjectedPageIndexNode,
) -> ProjectedPageIndexNodeHit {
    ProjectedPageIndexNodeHit {
        repo_id: page.repo_id.clone(),
        page_id: page.page_id.clone(),
        page_title: page.title.clone(),
        page_kind: page.kind,
        path: page.path.clone(),
        doc_id: page.doc_id.clone(),
        node_id: node.node_id.clone(),
        node_title: node.title.clone(),
        structural_path: node.structural_path.clone(),
        line_range: node.line_range,
        text: node.text.clone(),
    }
}

#[async_trait]
impl RepoSearchFlightRouteProvider for RepoSearchFlightHostProvider {
    async fn repo_search_batch(
        &self,
        request: &RepoSearchFlightRequest,
    ) -> Result<LanceRecordBatch, String> {
        let request = request_with_default_repo(request, self.repo_id.as_str());
        search_repo_content_batch(self.search_plane.as_ref(), &request).await
    }
}

#[async_trait]
impl RepoProjectedPageIndexTreeFlightRouteProvider for RepoSearchFlightHostProvider {
    async fn repo_projected_page_index_tree_batch(
        &self,
        repo_id: &str,
        page_id: &str,
    ) -> Result<AnalysisFlightRouteResponse, Status> {
        let effective_repo_id = effective_repo_id(repo_id, self.repo_id.as_str());
        let bootstrap_projection_cache = self.bootstrap_projection_cache()?;
        let response = resolve_projected_page_index_tree(
            effective_repo_id.as_str(),
            page_id,
            self.config_path.as_deref(),
            self.project_root.as_path(),
            self.bootstrap_analysis.as_deref(),
            bootstrap_projection_cache.as_deref(),
        )?;
        let batch = repo_projected_page_index_tree_batch(&response).map_err(Status::internal)?;
        let metadata =
            repo_projected_page_index_tree_metadata(&response).map_err(Status::internal)?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(metadata))
    }
}

#[async_trait]
impl RepoProjectedRetrievalContextFlightRouteProvider for RepoSearchFlightHostProvider {
    async fn repo_projected_retrieval_context_batch(
        &self,
        repo_id: &str,
        page_id: &str,
        node_id: Option<&str>,
        related_limit: usize,
    ) -> Result<AnalysisFlightRouteResponse, Status> {
        let effective_repo_id = effective_repo_id(repo_id, self.repo_id.as_str());
        let bootstrap_projection_cache = self.bootstrap_projection_cache()?;
        let response = resolve_projected_retrieval_context(
            effective_repo_id.as_str(),
            page_id,
            node_id,
            related_limit,
            self.config_path.as_deref(),
            self.project_root.as_path(),
            self.bootstrap_analysis.as_deref(),
            bootstrap_projection_cache.as_deref(),
        )?;
        let batch =
            repo_projected_retrieval_context_batch(&response, node_id).map_err(Status::internal)?;
        let metadata = repo_projected_retrieval_context_metadata(&response, node_id)
            .map_err(Status::internal)?;
        Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(metadata))
    }
}

#[async_trait]
impl GraphNeighborsFlightRouteProvider for RepoSearchFlightHostProvider {
    async fn graph_neighbors_batch(
        &self,
        node_id: &str,
        direction: &str,
        hops: usize,
        limit: usize,
    ) -> Result<GraphNeighborsFlightRouteResponse, Status> {
        let direction = graph_direction_from_token(direction);
        let mut attempted_errors = Vec::new();
        for node_id in graph_node_id_variants(self.repo_id.as_str(), node_id) {
            match query_graph_neighbors_projection(
                Arc::clone(&self.link_graph_index),
                node_id.as_str(),
                direction,
                hops,
                limit,
                None,
            )
            .await
            {
                Ok(projection) => {
                    let batch =
                        graph_neighbors_projection_batch(self.repo_id.as_str(), &projection)
                            .map_err(Status::internal)?;
                    return Ok(GraphNeighborsFlightRouteResponse::new(batch));
                }
                Err(error) if is_graph_node_not_found(error.to_string().as_str()) => {
                    attempted_errors.push(format!("{node_id}: {error}"));
                }
                Err(error) => return Err(internal_status(error)),
            }
        }
        Err(Status::internal(format!(
            "graph node `{node_id}` not found after repo-id normalization: {}",
            attempted_errors.join("; ")
        )))
    }
}

pub(super) fn build_search_strategy_flow_flight_service(
    parts: SearchStrategyFlowFlightHostParts,
    expected_schema_version: impl Into<String>,
    rerank_dimension: usize,
    rerank_weights: RerankScoreWeights,
) -> Result<WendaoFlightService, String> {
    let bootstrap_projection_cache = parts
        .bootstrap_analysis
        .as_deref()
        .map(BootstrapProjectionCache::build)
        .transpose()
        .map_err(|error| format!("build bootstrap projection cache: {error}"))?
        .map(Arc::new);
    let provider = Arc::new(RepoSearchFlightHostProvider {
        repo_id: parts.repo_id,
        project_root: parts.project_root,
        config_path: parts.config_path,
        search_plane: parts.search_plane,
        link_graph_index: parts.link_graph_index,
        bootstrap_analysis: parts.bootstrap_analysis,
        bootstrap_projection_cache: Arc::new(Mutex::new(bootstrap_projection_cache)),
    });
    let mut route_providers = WendaoFlightRouteProviders::new(provider.clone());
    route_providers.repo_projected_page_index_tree = Some(provider.clone());
    route_providers.repo_projected_retrieval_context = Some(provider.clone());
    route_providers.graph_neighbors = Some(provider);
    WendaoFlightService::new_with_route_providers(
        expected_schema_version,
        route_providers,
        rerank_dimension,
        rerank_weights,
    )
}

fn request_with_default_repo(
    request: &RepoSearchFlightRequest,
    default_repo_id: &str,
) -> RepoSearchFlightRequest {
    if !request.repo_id.trim().is_empty() {
        return request.clone();
    }

    let mut request = request.clone();
    request.repo_id = default_repo_id.to_string();
    request
}

fn effective_repo_id(candidate: &str, default_repo_id: &str) -> String {
    if candidate.trim().is_empty() {
        default_repo_id.to_string()
    } else {
        candidate.to_string()
    }
}

fn graph_direction_from_token(direction: &str) -> GraphDirection {
    match direction.trim().to_ascii_lowercase().as_str() {
        "incoming" | "in" => GraphDirection::Incoming,
        "outgoing" | "out" => GraphDirection::Outgoing,
        _ => GraphDirection::Both,
    }
}

fn graph_node_id_variants(repo_id: &str, node_id: &str) -> Vec<String> {
    let trimmed = node_id.trim().trim_matches('/');
    let mut variants = vec![trimmed.to_string()];
    let repo_prefix = format!("{}/", repo_id.trim().trim_matches('/'));
    push_graph_node_id_variant(
        &mut variants,
        trimmed
            .strip_prefix(repo_prefix.as_str())
            .map(str::to_string),
    );
    let doc_prefix = format!("repo:{repo_id}:doc:");
    push_graph_node_id_variant(
        &mut variants,
        trimmed
            .strip_prefix(doc_prefix.as_str())
            .map(str::to_string),
    );
    variants
}

fn push_graph_node_id_variant(variants: &mut Vec<String>, candidate: Option<String>) {
    let Some(candidate) = candidate.filter(|value| !value.trim().is_empty()) else {
        return;
    };
    if !variants.iter().any(|variant| variant == &candidate) {
        variants.push(candidate);
    }
}

fn is_graph_node_not_found(error: &str) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("graph node") && error.contains("not found")
}

fn resolve_projected_page_index_tree(
    repo_id: &str,
    page_id: &str,
    config_path: Option<&std::path::Path>,
    project_root: &std::path::Path,
    bootstrap_analysis: Option<&RepositoryAnalysisOutput>,
    bootstrap_projection_cache: Option<&BootstrapProjectionCache>,
) -> Result<crate::analyzers::RepoProjectedPageIndexTreeResult, Status> {
    let mut last_unknown_page = None;
    for page_id in projected_page_id_variants(repo_id, page_id) {
        let query = RepoProjectedPageIndexTreeQuery {
            repo_id: repo_id.to_string(),
            page_id,
        };
        if let Some(cache) = bootstrap_projection_cache {
            match cache.page_index_tree(&query) {
                Ok(response) => return Ok(response),
                Err(error) if is_unknown_projected_page(&error) => {
                    remember_unknown_page(&mut last_unknown_page, error);
                }
                Err(error) => return Err(internal_status(error)),
            }
        }
        match repo_projected_page_index_tree_from_config(&query, config_path, project_root) {
            Ok(response) => return Ok(response),
            Err(error) if should_try_bootstrap_projection(&error, bootstrap_analysis) => {
                remember_unknown_page(&mut last_unknown_page, error);
            }
            Err(error) => return Err(internal_status(error)),
        }
        if let Some(analysis) = bootstrap_analysis {
            match build_repo_projected_page_index_tree(&query, analysis) {
                Ok(response) => return Ok(response),
                Err(error) if is_unknown_projected_page(&error) => {
                    remember_unknown_page(&mut last_unknown_page, error);
                }
                Err(error) => return Err(internal_status(error)),
            }
        }
    }
    Err(internal_status(last_unknown_page.unwrap_or_else(|| {
        RepoIntelligenceError::UnknownProjectedPage {
            repo_id: repo_id.to_string().into(),
            page_id: page_id.to_string().into(),
        }
    })))
}

fn resolve_projected_retrieval_context(
    repo_id: &str,
    page_id: &str,
    node_id: Option<&str>,
    related_limit: usize,
    config_path: Option<&std::path::Path>,
    project_root: &std::path::Path,
    bootstrap_analysis: Option<&RepositoryAnalysisOutput>,
    bootstrap_projection_cache: Option<&BootstrapProjectionCache>,
) -> Result<crate::analyzers::RepoProjectedRetrievalContextResult, Status> {
    let mut last_unknown_page = None;
    for page_id in projected_page_id_variants(repo_id, page_id) {
        let query = RepoProjectedRetrievalContextQuery {
            repo_id: repo_id.to_string(),
            page_id,
            node_id: node_id.map(str::to_string),
            related_limit,
        };
        if let Some(cache) = bootstrap_projection_cache {
            match cache.retrieval_context(&query) {
                Ok(response) => return Ok(response),
                Err(error) if is_unknown_projected_page(&error) => {
                    remember_unknown_page(&mut last_unknown_page, error);
                }
                Err(error) => return Err(internal_status(error)),
            }
        }
        match repo_projected_retrieval_context_from_config(&query, config_path, project_root) {
            Ok(response) => return Ok(response),
            Err(error) if should_try_bootstrap_projection(&error, bootstrap_analysis) => {
                remember_unknown_page(&mut last_unknown_page, error);
            }
            Err(error) => return Err(internal_status(error)),
        }
        if let Some(analysis) = bootstrap_analysis {
            match build_repo_projected_retrieval_context(&query, analysis) {
                Ok(response) => return Ok(response),
                Err(error) if is_unknown_projected_page(&error) => {
                    remember_unknown_page(&mut last_unknown_page, error);
                }
                Err(error) => return Err(internal_status(error)),
            }
        }
    }
    Err(internal_status(last_unknown_page.unwrap_or_else(|| {
        RepoIntelligenceError::UnknownProjectedPage {
            repo_id: repo_id.to_string().into(),
            page_id: page_id.to_string().into(),
        }
    })))
}

fn remember_unknown_page(
    last_unknown_page: &mut Option<RepoIntelligenceError>,
    error: RepoIntelligenceError,
) {
    if last_unknown_page.is_none() {
        *last_unknown_page = Some(error);
    }
}

fn is_unknown_projected_page(error: &RepoIntelligenceError) -> bool {
    matches!(error, RepoIntelligenceError::UnknownProjectedPage { .. })
}

fn should_try_bootstrap_projection(
    error: &RepoIntelligenceError,
    bootstrap_analysis: Option<&RepositoryAnalysisOutput>,
) -> bool {
    bootstrap_analysis.is_some()
        && matches!(
            error,
            RepoIntelligenceError::UnknownProjectedPage { .. }
                | RepoIntelligenceError::UnknownRepository { .. }
                | RepoIntelligenceError::MissingRepoIntelligencePlugins { .. }
                | RepoIntelligenceError::ConfigLoad { .. }
        )
}

fn projected_page_id_variants(repo_id: &str, page_id: &str) -> Vec<String> {
    let mut variants = vec![page_id.to_string()];
    push_projected_page_id_variant(
        &mut variants,
        collapsed_projected_doc_page_id(repo_id, page_id),
    );
    push_projected_page_id_variant(
        &mut variants,
        expanded_projected_doc_page_id(repo_id, page_id),
    );
    variants
}

fn collapsed_projected_doc_page_id(repo_id: &str, page_id: &str) -> Option<String> {
    let nested_doc_marker = format!(":doc:repo:{repo_id}:doc:");
    let (prefix, suffix) = page_id.split_once(nested_doc_marker.as_str())?;
    Some(format!("{prefix}:doc:{suffix}"))
}

fn expanded_projected_doc_page_id(repo_id: &str, page_id: &str) -> Option<String> {
    if page_id.contains(&format!(":doc:repo:{repo_id}:doc:")) {
        return None;
    }
    let (prefix, suffix) = page_id.split_once(":doc:")?;
    if suffix.starts_with("repo:") {
        return None;
    }
    Some(format!("{prefix}:doc:repo:{repo_id}:doc:{suffix}"))
}

fn push_projected_page_id_variant(variants: &mut Vec<String>, candidate: Option<String>) {
    let Some(candidate) = candidate else {
        return;
    };
    if !variants.iter().any(|variant| variant == &candidate) {
        variants.push(candidate);
    }
}

fn internal_status(error: impl std::fmt::Display) -> Status {
    Status::internal(error.to_string())
}

#[cfg(test)]
#[path = "../../tests/unit/flight_host/providers.rs"]
mod tests;
