use std::future::Future;
use std::sync::Arc;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::analyzers::{
    CachedRepositoryAnalysis, RepoAnalysisFallbackContract, RepoIntelligenceError,
    repository_search_artifacts,
};
use crate::gateway::studio::router::handlers::repo::analysis::search::cache::{
    repository_search_key, with_cached_repo_search_result,
};
use crate::gateway::studio::router::handlers::repo::analysis::search::publication::repo_entity_publication_ready;
use crate::gateway::studio::router::handlers::repo::shared::execution::with_repo_cached_analysis_bundle;
use crate::gateway::studio::router::{GatewayState, StudioApiError};
use crate::query_core::{RepoEntityTypedResultsContract, query_repo_entity_results_if_published};
use crate::search::FuzzySearchOptions;

pub(super) struct RepoAnalysisSearchSpec {
    pub(super) scope: &'static str,
    pub(super) panic_code: &'static str,
    pub(super) panic_message: &'static str,
    pub(super) fuzzy_options: FuzzySearchOptions,
}

pub(super) struct RepoAnalysisTypedSearchContract<Q, T> {
    pub(super) spec: RepoAnalysisSearchSpec,
    pub(super) error_code: &'static str,
    pub(super) error_message: &'static str,
    pub(super) fast_path: RepoEntityTypedResultsContract<T>,
    pub(super) fallback: RepoAnalysisFallbackContract<Q, T>,
}

pub(super) struct RepoAnalysisFallbackSearchContract<Q, T> {
    pub(super) spec: RepoAnalysisSearchSpec,
    pub(super) fallback: RepoAnalysisFallbackContract<Q, T>,
}

pub(super) async fn run_repo_analysis_search<T, FastFn, FastFut, FallbackFn>(
    state: Arc<GatewayState>,
    repo_id: String,
    search_query: String,
    limit: usize,
    spec: RepoAnalysisSearchSpec,
    fast_path: FastFn,
    fallback: FallbackFn,
) -> Result<T, StudioApiError>
where
    T: Serialize + DeserializeOwned + Send + 'static,
    FastFn: FnOnce(Arc<GatewayState>, String, String, usize) -> FastFut,
    FastFut: Future<Output = Result<Option<T>, StudioApiError>>,
    FallbackFn: FnOnce(String, String, usize, CachedRepositoryAnalysis) -> Result<T, RepoIntelligenceError>
        + Send
        + 'static,
{
    let search_plane = state.studio.search_plane.clone();
    let cache_repo_id = repo_id.clone();
    let cache_query = search_query.clone();
    with_cached_repo_search_result(
        &search_plane,
        spec.scope,
        cache_repo_id.as_str(),
        cache_query.as_str(),
        limit,
        {
            let state = Arc::clone(&state);
            move || async move {
                if let Some(result) = fast_path(
                    Arc::clone(&state),
                    repo_id.clone(),
                    search_query.clone(),
                    limit,
                )
                .await?
                {
                    return Ok(result);
                }

                with_repo_cached_analysis_bundle(
                    Arc::clone(&state),
                    repo_id.clone(),
                    spec.panic_code,
                    spec.panic_message,
                    move |cached| {
                        let cache_key = repository_search_key(
                            &cached.cache_key,
                            spec.scope,
                            search_query.as_str(),
                            limit,
                            spec.fuzzy_options,
                        );
                        if let Some(result) =
                            crate::analyzers::load_cached_repository_search_result(&cache_key)?
                        {
                            return Ok(result);
                        }

                        let result = fallback(repo_id, search_query, limit, cached.clone())?;
                        crate::analyzers::store_cached_repository_search_result(
                            &cache_key, &result,
                        )?;
                        Ok(result)
                    },
                )
                .await
            }
        },
    )
    .await
}

pub(super) async fn run_typed_repo_analysis_search<Q, T>(
    state: Arc<GatewayState>,
    repo_id: String,
    search_query: String,
    limit: usize,
    contract: RepoAnalysisTypedSearchContract<Q, T>,
) -> Result<T, StudioApiError>
where
    T: Serialize + DeserializeOwned + Send + 'static,
    Q: Send + 'static,
{
    let publication_ready = repo_entity_publication_ready(&state, repo_id.as_str()).await;
    let RepoAnalysisTypedSearchContract {
        spec,
        error_code,
        error_message,
        fast_path,
        fallback,
    } = contract;
    let RepoAnalysisSearchSpec {
        scope: _,
        panic_code,
        panic_message,
        fuzzy_options: _,
    } = spec;
    let query = (fallback.build_query)(repo_id.clone(), search_query.clone(), limit);
    let fallback_scope = fallback.scope;
    let fallback_fuzzy_options = fallback.fuzzy_options;
    let fallback_query_text = fallback.query_text;
    let fallback_query_limit = fallback.query_limit;
    let fallback_build_result = fallback.build_result;
    run_repo_analysis_search(
        Arc::clone(&state),
        repo_id,
        search_query,
        limit,
        RepoAnalysisSearchSpec {
            scope: fallback_scope,
            panic_code,
            panic_message,
            fuzzy_options: fallback_fuzzy_options,
        },
        move |state, repo_id, search_query, limit| async move {
            query_repo_entity_results_if_published(
                &state.studio.search_plane,
                repo_id.as_str(),
                search_query.as_str(),
                limit,
                publication_ready,
                fast_path,
            )
            .await
            .map_err(|error| {
                StudioApiError::internal(error_code, error_message, Some(error.to_string()))
            })
        },
        move |_repo_id, _search_query, _limit, cached| {
            let query_text = fallback_query_text(&query);
            load_or_build_repo_analysis_result(
                &cached,
                fallback_scope,
                query_text.as_str(),
                fallback_query_limit(&query),
                fallback_fuzzy_options,
                |analysis, artifacts| fallback_build_result(&query, analysis, artifacts),
            )
        },
    )
    .await
}

fn load_or_build_repo_analysis_result<T, BuildFn>(
    cached: &CachedRepositoryAnalysis,
    scope: &'static str,
    query: &str,
    limit: usize,
    fuzzy_options: FuzzySearchOptions,
    build: BuildFn,
) -> Result<T, RepoIntelligenceError>
where
    T: Serialize + DeserializeOwned,
    BuildFn: FnOnce(
        &crate::analyzers::RepositoryAnalysisOutput,
        &crate::analyzers::RepositorySearchArtifacts,
    ) -> T,
{
    let cache_key = repository_search_key(&cached.cache_key, scope, query, limit, fuzzy_options);
    if let Some(result) = crate::analyzers::load_cached_repository_search_result(&cache_key)? {
        return Ok(result);
    }

    let artifacts = repository_search_artifacts(&cached.cache_key, &cached.analysis)?;
    let result = build(&cached.analysis, artifacts.as_ref());
    crate::analyzers::store_cached_repository_search_result(&cache_key, &result)?;
    Ok(result)
}

pub(super) async fn run_fallback_repo_analysis_search<Q, T>(
    state: Arc<GatewayState>,
    repo_id: String,
    limit: usize,
    contract: RepoAnalysisFallbackSearchContract<Q, T>,
) -> Result<T, StudioApiError>
where
    T: Serialize + DeserializeOwned + Send + 'static,
    Q: Send + 'static,
{
    let RepoAnalysisFallbackSearchContract { spec, fallback } = contract;
    let query = (fallback.build_query)(repo_id.clone(), String::new(), limit);
    let cache_query = (fallback.query_text)(&query);
    run_repo_analysis_search(
        Arc::clone(&state),
        repo_id,
        cache_query,
        limit,
        spec,
        |_state, _repo_id, _search_query, _limit| async move { Ok(None) },
        move |_repo_id, _search_query, _limit, cached| {
            let query_text = (fallback.query_text)(&query);
            load_or_build_repo_analysis_result(
                &cached,
                fallback.scope,
                query_text.as_str(),
                (fallback.query_limit)(&query),
                fallback.fuzzy_options,
                |analysis, artifacts| (fallback.build_result)(&query, analysis, artifacts),
            )
        },
    )
    .await
}
