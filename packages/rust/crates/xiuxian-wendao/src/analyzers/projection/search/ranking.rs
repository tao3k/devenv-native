//! `analyzers::projection::search::ranking` owns Wendao projection search ranking behavior.

use crate::analyzers::RepositoryAnalysisOutput;
#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
use crate::analyzers::cache::RepositorySearchArtifacts;
use crate::analyzers::{
    ProjectedPageRecord, ProjectionPageKind, RepoProjectedPageSearchQuery,
    RepoProjectedPageSearchResult,
};

use super::heuristic::heuristic_projected_page_matches;
#[cfg(feature = "repo-lexical-index")]
use super::indexed::search_indexed_projected_pages;
#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
use super::indexed::search_projected_pages_with_index;
use super::lexical::lexical_projected_page_matches;
use super::options::projected_page_document_search_options;
use crate::analyzers::projection::pages::build_projected_pages;

/// Namespace boundary: this public name is scoped by its module owner.
/// Build projected-page search results for one repository query.
#[must_use]
pub fn build_repo_projected_page_search(
    query: &RepoProjectedPageSearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> RepoProjectedPageSearchResult {
    build_repo_projected_page_search_with_options(
        query,
        analysis,
        projected_page_document_search_options(),
    )
}
/// `build_repo_projected_page_search_with_options` public function boundary for Wendao.
#[must_use]
pub fn build_repo_projected_page_search_with_options(
    query: &RepoProjectedPageSearchQuery,
    analysis: &RepositoryAnalysisOutput,
    options: crate::search::FuzzySearchOptions,
) -> RepoProjectedPageSearchResult {
    RepoProjectedPageSearchResult {
        repo_id: query.repo_id.clone(),
        pages: ranked_projected_page_matches(
            query.query.as_str(),
            query.kind,
            analysis,
            query.limit,
            options,
        )
        .into_iter()
        .map(|(_, page)| page)
        .collect(),
    }
}
/// Namespace boundary: this public name is scoped by its module owner.
/// `build_repo_projected_page_search_with_artifacts` public function boundary for Wendao.
#[must_use]
#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
pub fn build_repo_projected_page_search_with_artifacts(
    query: &RepoProjectedPageSearchQuery,
    analysis: &RepositoryAnalysisOutput,
    artifacts: &RepositorySearchArtifacts,
) -> RepoProjectedPageSearchResult {
    let _ = analysis;
    RepoProjectedPageSearchResult {
        repo_id: query.repo_id.clone(),
        pages: ranked_projected_page_matches_with_artifacts(
            query.query.as_str(),
            query.kind,
            artifacts,
            query.limit,
            projected_page_document_search_options(),
        )
        .into_iter()
        .map(|(_, page)| page)
        .collect(),
    }
}
/// `scored_projected_page_matches` public function boundary for Wendao.
#[must_use]
pub fn scored_projected_page_matches(
    query: &str,
    kind_filter: Option<ProjectionPageKind>,
    analysis: &RepositoryAnalysisOutput,
) -> Vec<(u8, ProjectedPageRecord)> {
    ranked_projected_page_matches(
        query,
        kind_filter,
        analysis,
        usize::MAX,
        projected_page_document_search_options(),
    )
}

pub(super) fn ranked_projected_page_matches(
    query: &str,
    kind_filter: Option<ProjectionPageKind>,
    analysis: &RepositoryAnalysisOutput,
    limit: usize,
    options: crate::search::FuzzySearchOptions,
) -> Vec<(u8, ProjectedPageRecord)> {
    let query = query.trim();
    if query.is_empty() || limit == 0 {
        return Vec::new();
    }

    let pages = build_projected_pages(analysis);
    let limit = limit.min(pages.len());
    if limit == 0 {
        return Vec::new();
    }

    #[cfg(feature = "repo-lexical-index")]
    if let Some(indexed_matches) =
        search_indexed_projected_pages(query, kind_filter, pages.as_slice(), limit, options)
        && !indexed_matches.is_empty()
    {
        return indexed_matches;
    }

    let normalized_query = query.to_ascii_lowercase();
    let scored_matches =
        heuristic_projected_page_matches(normalized_query.as_str(), kind_filter, pages.as_slice());
    if !scored_matches.is_empty() {
        return scored_matches.into_iter().take(limit).collect();
    }

    lexical_projected_page_matches(query, kind_filter, pages.as_slice(), limit, options)
}
/// `ranked_projected_page_matches_with_artifacts` public function boundary for Wendao.
#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
/// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
pub fn ranked_projected_page_matches_with_artifacts(
    query: &str,
    kind_filter: Option<ProjectionPageKind>,
    artifacts: &RepositorySearchArtifacts,
    limit: usize,
    options: crate::search::FuzzySearchOptions,
) -> Vec<(u8, ProjectedPageRecord)> {
    let query = query.trim();
    if query.is_empty() || limit == 0 {
        return Vec::new();
    }

    let limit = limit.min(artifacts.projected_pages.len());
    if limit == 0 {
        return Vec::new();
    }

    let indexed_matches = search_projected_pages_with_index(
        query,
        kind_filter,
        &artifacts.projected_page_index,
        &artifacts.projected_pages_by_id,
        limit,
        options,
    );
    if !indexed_matches.is_empty() {
        return indexed_matches;
    }

    let normalized_query = query.to_ascii_lowercase();
    let scored_matches = heuristic_projected_page_matches(
        normalized_query.as_str(),
        kind_filter,
        artifacts.projected_pages.as_slice(),
    );
    if !scored_matches.is_empty() {
        return scored_matches.into_iter().take(limit).collect();
    }

    lexical_projected_page_matches(
        query,
        kind_filter,
        artifacts.projected_pages.as_slice(),
        limit,
        options,
    )
}
