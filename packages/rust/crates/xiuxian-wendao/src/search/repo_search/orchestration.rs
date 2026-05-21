//! `search::repo_search::orchestration` owns Wendao search repo search orchestration behavior.

use crate::analyzers::RegisteredRepository;
use crate::parsers::search::repo_code_query::ParsedRepoCodeSearchQuery;
use crate::search::SearchPlaneService;
use crate::search::contracts::SearchHit;

use super::ast::{
    ast_pattern_requests_generic_analysis, has_generic_ast_language_filters,
    repository_supports_generic_ast_analysis, search_repo_ast_analysis_hits,
    search_repo_ast_pattern_hits,
};
use super::buffered::search_repo_intent_hits_buffered;
use super::buffered::{RepoSearchResultLimits, search_repo_code_hits_buffered};
use super::dispatch::{RepoSearchDispatch, collect_repo_search_targets, repo_search_parallelism};

use std::time::Duration;

#[derive(Debug, Default)]
/// Repository intent-search result plus repositories that were not searchable yet.
pub struct RepoIntentSearchOutcome {
    /// Search hits produced by searchable repositories.
    pub hits: Vec<SearchHit>,
    /// Repository ids whose publications are still pending.
    pub pending_repos: Vec<String>,
    /// Repository ids skipped because they are unsupported or unavailable.
    pub skipped_repos: Vec<String>,
    #[cfg(any(test, feature = "test-support"))]
    /// Whether at least one repository content publication was available.
    pub repo_content_available: bool,
}

#[derive(Debug, Default)]
/// Repository code-search result plus dispatch state for unavailable repositories.
pub struct RepoCodeSearchOutcome {
    /// Search hits produced by searchable repositories.
    pub hits: Vec<SearchHit>,
    /// Repository ids whose publications are still pending.
    pub pending_repos: Vec<String>,
    /// Repository ids skipped because they are unsupported or unavailable.
    pub skipped_repos: Vec<String>,
    /// Whether buffered repository search stopped after a partial timeout.
    pub partial_timeout: bool,
}

#[derive(Debug, thiserror::Error)]
/// Errors returned while executing repository code search.
pub enum RepoCodeSearchExecutionError {
    /// Ast-grep search was requested without exactly one repository scope.
    #[error("ast-grep code search requires one explicit repository scope")]
    MissingRepositoryScopeForAstGrep,
    /// Repository search failed after dispatching to the selected backend.
    #[error("{0}")]
    Search(String),
}

/// Search repository intent across selected repositories and report dispatch gaps.
///
/// # Errors
///
/// Returns a search error string when one of the repository search workers fails.
pub async fn search_repo_intent_outcome(
    search_plane: &SearchPlaneService,
    repo_ids: Vec<String>,
    raw_query: &str,
    limit: usize,
) -> Result<RepoIntentSearchOutcome, String> {
    let dispatch = prepare_repo_search_dispatch(search_plane, repo_ids).await;
    #[cfg(any(test, feature = "test-support"))]
    let repo_content_available = dispatch
        .searchable
        .iter()
        .any(|target| target.publication_state.content_published);
    let hits = search_repo_intent_hits_buffered(
        search_plane.clone(),
        dispatch.searchable,
        raw_query,
        limit,
    )
    .await?;

    Ok(RepoIntentSearchOutcome {
        hits,
        pending_repos: dispatch.pending,
        skipped_repos: dispatch.skipped,
        #[cfg(any(test, feature = "test-support"))]
        repo_content_available,
    })
}

pub(crate) async fn search_repo_code_outcome(
    search_plane: &SearchPlaneService,
    repo_ids: Vec<String>,
    raw_query: &str,
    per_repo_limits: RepoSearchResultLimits,
    repo_wide_budget: Option<Duration>,
) -> Result<RepoCodeSearchOutcome, String> {
    let dispatch = prepare_repo_search_dispatch(search_plane, repo_ids).await;
    let buffered = search_repo_code_hits_buffered(
        search_plane.clone(),
        dispatch.searchable,
        raw_query,
        per_repo_limits,
        repo_wide_budget,
    )
    .await?;

    Ok(RepoCodeSearchOutcome {
        hits: buffered.hits,
        pending_repos: dispatch.pending,
        skipped_repos: dispatch.skipped,
        partial_timeout: buffered.partial_timeout,
    })
}

/// Search repository code with parsed filters and report dispatch gaps.
///
/// # Errors
///
/// Returns a typed execution error when the selected AST or content backend
/// cannot satisfy the query.
/// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
pub async fn search_repo_code_outcome_for_query(
    search_plane: &SearchPlaneService,
    selected_repository: Option<&RegisteredRepository>,
    repo_ids: Vec<String>,
    raw_query: &str,
    parsed_query: &ParsedRepoCodeSearchQuery,
    per_repo_limits: RepoSearchResultLimits,
    repo_wide_budget: Option<Duration>,
) -> Result<RepoCodeSearchOutcome, RepoCodeSearchExecutionError> {
    if let Some(ast_pattern) = parsed_query.ast_pattern.as_deref() {
        let repository = selected_repository
            .ok_or(RepoCodeSearchExecutionError::MissingRepositoryScopeForAstGrep)?;
        let language_filters = parsed_query
            .language_filters
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        if ast_pattern_requests_generic_analysis(ast_pattern) {
            let hits = search_repo_ast_analysis_hits(
                search_plane,
                repository,
                parsed_query.search_term(),
                language_filters.as_slice(),
                per_repo_limits.entity_limit,
            )
            .await
            .map_err(RepoCodeSearchExecutionError::Search)?;
            return Ok(RepoCodeSearchOutcome {
                hits,
                pending_repos: Vec::new(),
                skipped_repos: Vec::new(),
                partial_timeout: false,
            });
        }
        let hits = search_repo_ast_pattern_hits(
            search_plane,
            repository,
            ast_pattern,
            language_filters.as_slice(),
            per_repo_limits.entity_limit,
        )
        .await
        .map_err(RepoCodeSearchExecutionError::Search)?;
        return Ok(RepoCodeSearchOutcome {
            hits,
            pending_repos: Vec::new(),
            skipped_repos: Vec::new(),
            partial_timeout: false,
        });
    }

    if let Some(repository) = selected_repository
        && should_use_generic_ast_analysis(repository, parsed_query)
    {
        let language_filters = parsed_query
            .language_filters
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let hits = search_repo_ast_analysis_hits(
            search_plane,
            repository,
            parsed_query.search_term(),
            language_filters.as_slice(),
            per_repo_limits.entity_limit,
        )
        .await
        .map_err(RepoCodeSearchExecutionError::Search)?;
        return Ok(RepoCodeSearchOutcome {
            hits,
            pending_repos: Vec::new(),
            skipped_repos: Vec::new(),
            partial_timeout: false,
        });
    }

    search_repo_code_outcome(
        search_plane,
        repo_ids,
        raw_query,
        per_repo_limits,
        repo_wide_budget,
    )
    .await
    .map_err(RepoCodeSearchExecutionError::Search)
}

fn should_use_generic_ast_analysis(
    repository: &RegisteredRepository,
    parsed_query: &ParsedRepoCodeSearchQuery,
) -> bool {
    repository_supports_generic_ast_analysis(repository)
        && parsed_query.kind_filters.is_empty()
        && (!repository.has_repo_intelligence_plugins()
            || has_generic_ast_language_filters(repository, &parsed_query.language_filters))
}

async fn prepare_repo_search_dispatch(
    search_plane: &SearchPlaneService,
    repo_ids: Vec<String>,
) -> RepoSearchDispatch {
    let publication_states = search_plane
        .repo_search_publication_states(repo_ids.as_slice())
        .await;
    let dispatch = collect_repo_search_targets(repo_ids, &publication_states);
    search_plane.record_repo_search_dispatch(
        dispatch.pending.len() + dispatch.skipped.len() + dispatch.searchable.len(),
        dispatch.searchable.len(),
        repo_search_parallelism(search_plane, dispatch.searchable.len()),
    );
    dispatch
}
