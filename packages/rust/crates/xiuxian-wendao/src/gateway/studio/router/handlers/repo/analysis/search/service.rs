use std::future::Future;
use std::sync::Arc;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::analyzers::{
    CachedRepositoryAnalysis, RepoAnalysisFallbackContract, example_fallback_contract,
    import_fallback_contract, module_fallback_contract, repository_search_artifacts,
    symbol_fallback_contract,
};
use crate::analyzers::{
    ExampleSearchResult, ImportSearchResult, ModuleSearchResult, RepoIntelligenceError,
    SymbolSearchResult,
};
use crate::gateway::studio::router::GatewayState;
use crate::gateway::studio::router::StudioApiError;
use crate::gateway::studio::router::handlers::repo::analysis::search::cache::{
    repository_search_key, with_cached_repo_search_result,
};
use crate::gateway::studio::router::handlers::repo::analysis::search::publication::repo_entity_publication_ready;
use crate::gateway::studio::router::handlers::repo::shared::with_repo_cached_analysis_bundle;
use crate::query_core::{
    RepoEntityTypedResultsContract, query_repo_entity_import_results_if_published,
    query_repo_entity_results_if_published, repo_entity_example_results_contract,
    repo_entity_module_results_contract, repo_entity_symbol_results_contract,
};
use crate::search::FuzzySearchOptions;

pub(crate) struct RepoAnalysisSearchSpec {
    pub(crate) scope: &'static str,
    pub(crate) panic_code: &'static str,
    pub(crate) panic_message: &'static str,
    pub(crate) fuzzy_options: FuzzySearchOptions,
}

struct RepoAnalysisTypedSearchContract<Q, T> {
    spec: RepoAnalysisSearchSpec,
    error_code: &'static str,
    error_message: &'static str,
    fast_path: RepoEntityTypedResultsContract<T>,
    fallback: RepoAnalysisFallbackContract<Q, T>,
}

struct RepoAnalysisFallbackSearchContract<Q, T> {
    spec: RepoAnalysisSearchSpec,
    fallback: RepoAnalysisFallbackContract<Q, T>,
}

pub(crate) async fn run_repo_module_search(
    state: Arc<GatewayState>,
    repo_id: String,
    search_query: String,
    limit: usize,
) -> Result<ModuleSearchResult, StudioApiError> {
    run_typed_repo_analysis_search(
        Arc::clone(&state),
        repo_id,
        search_query,
        limit,
        RepoAnalysisTypedSearchContract {
            spec: RepoAnalysisSearchSpec {
                scope: module_fallback_contract().scope,
                panic_code: "REPO_MODULE_SEARCH_PANIC",
                panic_message: "Repo module search task failed unexpectedly",
                fuzzy_options: module_fallback_contract().fuzzy_options,
            },
            error_code: "REPO_MODULE_SEARCH_FAILED",
            error_message: "Repo module search task failed",
            fast_path: repo_entity_module_results_contract(),
            fallback: module_fallback_contract(),
        },
    )
    .await
}

pub(crate) async fn run_repo_symbol_search(
    state: Arc<GatewayState>,
    repo_id: String,
    search_query: String,
    limit: usize,
) -> Result<SymbolSearchResult, StudioApiError> {
    run_typed_repo_analysis_search(
        Arc::clone(&state),
        repo_id,
        search_query,
        limit,
        RepoAnalysisTypedSearchContract {
            spec: RepoAnalysisSearchSpec {
                scope: symbol_fallback_contract().scope,
                panic_code: "REPO_SYMBOL_SEARCH_PANIC",
                panic_message: "Repo symbol search task failed unexpectedly",
                fuzzy_options: symbol_fallback_contract().fuzzy_options,
            },
            error_code: "REPO_SYMBOL_SEARCH_FAILED",
            error_message: "Repo symbol search task failed",
            fast_path: repo_entity_symbol_results_contract(),
            fallback: symbol_fallback_contract(),
        },
    )
    .await
}

pub(crate) async fn run_repo_example_search(
    state: Arc<GatewayState>,
    repo_id: String,
    search_query: String,
    limit: usize,
) -> Result<ExampleSearchResult, StudioApiError> {
    run_typed_repo_analysis_search(
        Arc::clone(&state),
        repo_id,
        search_query,
        limit,
        RepoAnalysisTypedSearchContract {
            spec: RepoAnalysisSearchSpec {
                scope: example_fallback_contract().scope,
                panic_code: "REPO_EXAMPLE_SEARCH_PANIC",
                panic_message: "Repo example search task failed unexpectedly",
                fuzzy_options: example_fallback_contract().fuzzy_options,
            },
            error_code: "REPO_EXAMPLE_SEARCH_FAILED",
            error_message: "Repo example search task failed",
            fast_path: repo_entity_example_results_contract(),
            fallback: example_fallback_contract(),
        },
    )
    .await
}

pub(crate) async fn run_repo_import_search(
    state: Arc<GatewayState>,
    repo_id: String,
    package: Option<String>,
    module: Option<String>,
    limit: usize,
) -> Result<ImportSearchResult, StudioApiError> {
    let publication_ready = repo_entity_publication_ready(&state, repo_id.as_str()).await;
    if let Some(result) = query_repo_entity_import_results_if_published(
        &state.studio.search_plane,
        repo_id.as_str(),
        package.clone(),
        module.clone(),
        limit,
        publication_ready,
    )
    .await
    .map_err(|error| {
        StudioApiError::internal(
            "REPO_IMPORT_SEARCH_FAILED",
            "Repo import search task failed",
            Some(error.to_string()),
        )
    })? {
        return Ok(result);
    }

    let fallback = import_fallback_contract(package, module);
    run_fallback_repo_analysis_search(
        Arc::clone(&state),
        repo_id,
        limit,
        RepoAnalysisFallbackSearchContract {
            spec: RepoAnalysisSearchSpec {
                scope: fallback.scope,
                panic_code: "REPO_IMPORT_SEARCH_PANIC",
                panic_message: "Repo import search task failed unexpectedly",
                fuzzy_options: fallback.fuzzy_options,
            },
            fallback,
        },
    )
    .await
}

pub(crate) async fn run_repo_analysis_search<T, FastFn, FastFut, FallbackFn>(
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

async fn run_typed_repo_analysis_search<Q, T>(
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

async fn run_fallback_repo_analysis_search<Q, T>(
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

#[cfg(test)]
#[path = "../../../../../../../../tests/unit/gateway/studio/router/handlers/repo/analysis/search/service.rs"]
mod tests;
