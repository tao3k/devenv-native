use std::time::Duration;

use crate::analyzers::RegisteredRepository;
use crate::gateway::studio::router::{
    StudioApiError, StudioState, configured_repositories, configured_repository,
    map_repo_intelligence_error,
};
use crate::gateway::studio::types::SearchResponse;
use crate::parsers::search::repo_code_query::{
    ParsedRepoCodeSearchQuery, parse_repo_code_search_query_with_repo_hint,
};
use crate::search::repo_search::{
    RepoCodeSearchExecutionError, search_repo_code_outcome_for_query,
};
use crate::search::{RepoSearchQueryCacheKeyInput, SearchCorpusKind, SearchPlaneCacheTtl};

use crate::gateway::studio::search::handlers::code_search::query::{
    infer_repo_hint_from_repositories, query_uses_redundant_repo_seed, repo_search_result_limits,
    repo_wide_code_search_timeout,
};

struct ResolvedCodeSearchScope {
    parsed: ParsedRepoCodeSearchQuery,
    effective_repo_hint: Option<String>,
    effective_repo_wide_budget: Option<Duration>,
    selected_repository: Option<RegisteredRepository>,
    repo_ids: Vec<String>,
}

/// Build one code-search response from the Studio search plane.
///
/// # Errors
///
/// Returns [`StudioApiError`] when repository configuration is invalid or the repo-backed search
/// plane encounters a failure while producing the response payload.
#[allow(clippy::too_many_lines)]
pub(crate) async fn build_code_search_response(
    studio: &StudioState,
    raw_query: String,
    repo_hint: Option<&str>,
    limit: usize,
) -> Result<SearchResponse, StudioApiError> {
    build_code_search_response_with_budget(studio, raw_query, repo_hint, limit, None).await
}

/// Build one code-search response with an optional repository-wide timeout budget.
///
/// # Errors
///
/// Returns [`StudioApiError`] when repository configuration is invalid or the repo-backed search
/// plane encounters a failure while producing the response payload.
#[allow(clippy::too_many_lines)]
pub(crate) async fn build_code_search_response_with_budget(
    studio: &StudioState,
    raw_query: String,
    repo_hint: Option<&str>,
    limit: usize,
    repo_wide_budget: Option<Duration>,
) -> Result<SearchResponse, StudioApiError> {
    let scope = resolve_code_search_scope(studio, raw_query.as_str(), repo_hint, repo_wide_budget)?;
    let cache_key =
        build_code_search_cache_key_for_scope(studio, raw_query.as_str(), limit, &scope).await;
    let ResolvedCodeSearchScope {
        parsed,
        effective_repo_hint,
        effective_repo_wide_budget,
        selected_repository,
        repo_ids,
    } = scope;
    if let Some(cache_key) = cache_key.as_ref()
        && let Some(cached) = studio
            .search_plane
            .cache_get_json::<SearchResponse>(cache_key)
            .await
    {
        return Ok(cached);
    }
    let outcome = search_repo_code_outcome_for_query(
        &studio.search_plane,
        selected_repository.as_ref(),
        repo_ids,
        raw_query.as_str(),
        &parsed,
        repo_search_result_limits(effective_repo_hint.as_deref(), limit),
        effective_repo_wide_budget,
    )
    .await
    .map_err(|error| match error {
        RepoCodeSearchExecutionError::MissingRepositoryScopeForAstGrep => {
            StudioApiError::bad_request(
                "MISSING_REPOSITORY",
                "ast-grep code search requires repo:<id> or an explicit repository hint",
            )
        }
        RepoCodeSearchExecutionError::Search(message) => StudioApiError::internal(
            "REPO_CODE_SEARCH_FAILED",
            "Failed to execute shared repo code search",
            Some(message),
        ),
    })?;
    let mut hits = outcome.hits;
    let partial_timeout = outcome.partial_timeout;
    let pending_repos = outcome.pending_repos;
    let skipped_repos = outcome.skipped_repos;

    hits.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.stem.cmp(&right.stem))
    });
    hits.truncate(limit);

    let hit_count = hits.len();
    let indexing_state = if partial_timeout {
        "partial".to_string()
    } else if pending_repos.is_empty() {
        "ready".to_string()
    } else if hit_count == 0 {
        "indexing".to_string()
    } else {
        "partial".to_string()
    };

    let response = SearchResponse {
        query: raw_query,
        hit_count,
        hits,
        graph_confidence_score: None,
        selected_mode: Some("code_search".to_string()),
        intent: Some("code_search".to_string()),
        intent_confidence: Some(1.0),
        search_mode: Some("code_search".to_string()),
        partial: partial_timeout || !pending_repos.is_empty() || !skipped_repos.is_empty(),
        indexing_state: Some(indexing_state),
        pending_repos,
        skipped_repos,
    };
    if let Some(cache_key) = cache_key.as_ref() {
        studio
            .search_plane
            .cache_set_json(cache_key, SearchPlaneCacheTtl::HotQuery, &response)
            .await;
    }
    Ok(response)
}

fn resolve_code_search_scope(
    studio: &StudioState,
    raw_query: &str,
    repo_hint: Option<&str>,
    repo_wide_budget: Option<Duration>,
) -> Result<ResolvedCodeSearchScope, StudioApiError> {
    let mut parsed = parse_repo_code_search_query_with_repo_hint(raw_query, repo_hint);
    let configured_repositories = configured_repositories(studio);
    if parsed.repo.is_none() {
        parsed.repo =
            infer_repo_hint_from_repositories(&parsed, configured_repositories.as_slice());
    }
    let effective_repo_hint = parsed.repo.clone();
    let effective_repo_wide_budget = if effective_repo_hint.is_some() {
        None
    } else {
        repo_wide_budget.or_else(|| repo_wide_code_search_timeout(None))
    };
    let selected_repository = if let Some(repo_id) = effective_repo_hint.as_deref() {
        Some(configured_repository(studio, repo_id).map_err(map_repo_intelligence_error)?)
    } else {
        None
    };
    if let Some(repository) = selected_repository.as_ref()
        && query_uses_redundant_repo_seed(&parsed, repository)
    {
        parsed.search_term = None;
    }
    let repo_ids = if let Some(repository) = selected_repository.as_ref() {
        vec![repository.id.clone()]
    } else {
        configured_repositories
            .into_iter()
            .map(|repository| repository.id)
            .collect()
    };
    if repo_ids.is_empty() {
        return Err(StudioApiError::bad_request(
            "UNKNOWN_REPOSITORY",
            "No configured repository is available for code search",
        ));
    }
    Ok(ResolvedCodeSearchScope {
        parsed,
        effective_repo_hint,
        effective_repo_wide_budget,
        selected_repository,
        repo_ids,
    })
}

async fn build_code_search_cache_key_for_scope(
    studio: &StudioState,
    raw_query: &str,
    limit: usize,
    scope: &ResolvedCodeSearchScope,
) -> Option<String> {
    studio
        .search_plane
        .repo_search_query_cache_key(RepoSearchQueryCacheKeyInput {
            scope: "code_search",
            corpora: &[],
            repo_corpora: &[
                SearchCorpusKind::RepoEntity,
                SearchCorpusKind::RepoContentChunk,
            ],
            repo_ids: scope.repo_ids.as_slice(),
            query: raw_query,
            limit,
            intent: Some("code_search"),
            repo_hint: scope.effective_repo_hint.as_deref(),
        })
        .await
}

#[cfg(test)]
pub(crate) async fn build_code_search_cache_key(
    studio: &StudioState,
    raw_query: &str,
    repo_hint: Option<&str>,
    limit: usize,
) -> Result<Option<String>, StudioApiError> {
    let scope = resolve_code_search_scope(studio, raw_query, repo_hint, None)?;
    Ok(build_code_search_cache_key_for_scope(studio, raw_query, limit, &scope).await)
}
