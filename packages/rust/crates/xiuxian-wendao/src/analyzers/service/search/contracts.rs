#[cfg(any(feature = "search-runtime", feature = "search-runtime"))]
use crate::analyzers::ImportSearchQuery;
#[cfg(feature = "search-runtime")]
use crate::analyzers::RepositoryAnalysisOutput;
#[cfg(feature = "search-runtime")]
use crate::analyzers::{
    ExampleSearchQuery, ExampleSearchResult, ImportSearchResult, ModuleSearchQuery,
    ModuleSearchResult, SymbolSearchQuery, SymbolSearchResult,
};
#[cfg(feature = "search-runtime")]
use crate::search::FuzzySearchOptions;
#[cfg(feature = "search-runtime")]
use std::sync::Arc;

#[cfg(feature = "search-runtime")]
use super::{
    build_example_search_with_artifacts, build_import_search_with_artifacts,
    build_module_search_with_artifacts, build_symbol_search_with_artifacts,
};
#[cfg(feature = "search-runtime")]
use crate::analyzers::cache::RepositorySearchArtifacts;

#[cfg(feature = "search-runtime")]
type FallbackQueryBuilder<Q> = dyn Fn(String, String, usize) -> Q + Send + Sync;
#[cfg(feature = "search-runtime")]
type FallbackQueryText<Q> = dyn Fn(&Q) -> String + Send + Sync;
#[cfg(feature = "search-runtime")]
type FallbackQueryLimit<Q> = dyn Fn(&Q) -> usize + Send + Sync;
#[cfg(feature = "search-runtime")]
type FallbackResultBuilder<Q, T> =
    dyn Fn(&Q, &RepositoryAnalysisOutput, &RepositorySearchArtifacts) -> T + Send + Sync;

#[cfg(feature = "search-runtime")]
/// Fallback search contract for replaying repository analysis without live indexes.
pub struct RepoAnalysisFallbackContract<Q, T> {
    /// Stable fallback scope label used in telemetry and cache keys.
    pub scope: &'static str,
    /// Fuzzy search options used by the fallback index.
    pub fuzzy_options: FuzzySearchOptions,
    /// Builder that constructs a domain query from repo id, query text, and limit.
    pub build_query: Arc<FallbackQueryBuilder<Q>>,
    /// Extractor for the text that should be searched.
    pub query_text: Arc<FallbackQueryText<Q>>,
    /// Extractor for the maximum result limit.
    pub query_limit: Arc<FallbackQueryLimit<Q>>,
    /// Builder that renders fallback results from repository analysis artifacts.
    pub build_result: Arc<FallbackResultBuilder<Q, T>>,
}

#[cfg(feature = "search-runtime")]
/// Build the canonical normalized query text for import search.
#[must_use]
pub fn canonical_import_query_text(query: &ImportSearchQuery) -> String {
    let package = query.package.as_deref().unwrap_or("*");
    let module = query.module.as_deref().unwrap_or("*");
    format!("package={package};module={module}")
}

#[cfg(feature = "search-runtime")]
/// Build the module-search fallback contract.
pub fn module_fallback_contract()
-> RepoAnalysisFallbackContract<ModuleSearchQuery, ModuleSearchResult> {
    RepoAnalysisFallbackContract {
        scope: "repo.module-search",
        fuzzy_options: FuzzySearchOptions::path_search(),
        build_query: Arc::new(|repo_id, query, limit| ModuleSearchQuery {
            repo_id,
            query,
            limit,
        }),
        query_text: Arc::new(|query| query.query.clone()),
        query_limit: Arc::new(|query| query.limit),
        build_result: Arc::new(build_module_search_with_artifacts),
    }
}

#[cfg(feature = "search-runtime")]
/// Build the symbol-search fallback contract.
pub fn symbol_fallback_contract()
-> RepoAnalysisFallbackContract<SymbolSearchQuery, SymbolSearchResult> {
    RepoAnalysisFallbackContract {
        scope: "repo.symbol-search",
        fuzzy_options: FuzzySearchOptions::symbol_search(),
        build_query: Arc::new(|repo_id, query, limit| SymbolSearchQuery {
            repo_id,
            query,
            limit,
        }),
        query_text: Arc::new(|query| query.query.clone()),
        query_limit: Arc::new(|query| query.limit),
        build_result: Arc::new(build_symbol_search_with_artifacts),
    }
}

#[cfg(feature = "search-runtime")]
/// Build the example-search fallback contract.
pub fn example_fallback_contract()
-> RepoAnalysisFallbackContract<ExampleSearchQuery, ExampleSearchResult> {
    RepoAnalysisFallbackContract {
        scope: "repo.example-search",
        fuzzy_options: FuzzySearchOptions::document_search(),
        build_query: Arc::new(|repo_id, query, limit| ExampleSearchQuery {
            repo_id,
            query,
            limit,
        }),
        query_text: Arc::new(|query| query.query.clone()),
        query_limit: Arc::new(|query| query.limit),
        build_result: Arc::new(build_example_search_with_artifacts),
    }
}

#[cfg(feature = "search-runtime")]
/// Build the import-search fallback contract for optional package/module filters.
pub fn import_fallback_contract(
    package: Option<String>,
    module: Option<String>,
) -> RepoAnalysisFallbackContract<ImportSearchQuery, ImportSearchResult> {
    RepoAnalysisFallbackContract {
        scope: "repo.import-search",
        fuzzy_options: FuzzySearchOptions::symbol_search(),
        build_query: Arc::new(move |repo_id, _query, limit| ImportSearchQuery {
            repo_id,
            package: package.clone(),
            module: module.clone(),
            limit,
        }),
        query_text: Arc::new(canonical_import_query_text),
        query_limit: Arc::new(|query| query.limit),
        build_result: Arc::new(build_import_search_with_artifacts),
    }
}

#[cfg(all(test, feature = "search-runtime"))]
#[path = "../../../../tests/unit/analyzers/service/search/contracts.rs"]
mod tests;
