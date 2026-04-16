#[cfg(any(feature = "studio", feature = "search-runtime"))]
use crate::analyzers::ImportSearchQuery;
#[cfg(feature = "studio")]
use crate::analyzers::RepositoryAnalysisOutput;
#[cfg(feature = "studio")]
use crate::analyzers::{
    ExampleSearchQuery, ExampleSearchResult, ImportSearchResult, ModuleSearchQuery,
    ModuleSearchResult, SymbolSearchQuery, SymbolSearchResult,
};
#[cfg(feature = "studio")]
use crate::search::FuzzySearchOptions;
#[cfg(feature = "studio")]
use std::sync::Arc;

#[cfg(feature = "studio")]
use super::{
    build_example_search_with_artifacts, build_import_search_with_artifacts,
    build_module_search_with_artifacts, build_symbol_search_with_artifacts,
};
#[cfg(feature = "studio")]
use crate::analyzers::cache::RepositorySearchArtifacts;

#[cfg(feature = "studio")]
type FallbackQueryBuilder<Q> = dyn Fn(String, String, usize) -> Q + Send + Sync;
#[cfg(feature = "studio")]
type FallbackQueryText<Q> = dyn Fn(&Q) -> String + Send + Sync;
#[cfg(feature = "studio")]
type FallbackQueryLimit<Q> = dyn Fn(&Q) -> usize + Send + Sync;
#[cfg(feature = "studio")]
type FallbackResultBuilder<Q, T> =
    dyn Fn(&Q, &RepositoryAnalysisOutput, &RepositorySearchArtifacts) -> T + Send + Sync;

#[cfg(feature = "studio")]
pub(crate) struct RepoAnalysisFallbackContract<Q, T> {
    pub(crate) scope: &'static str,
    pub(crate) fuzzy_options: FuzzySearchOptions,
    pub(crate) build_query: Arc<FallbackQueryBuilder<Q>>,
    pub(crate) query_text: Arc<FallbackQueryText<Q>>,
    pub(crate) query_limit: Arc<FallbackQueryLimit<Q>>,
    pub(crate) build_result: Arc<FallbackResultBuilder<Q, T>>,
}

#[cfg(any(feature = "studio", feature = "search-runtime"))]
pub(crate) fn canonical_import_query_text(query: &ImportSearchQuery) -> String {
    let package = query.package.as_deref().unwrap_or("*");
    let module = query.module.as_deref().unwrap_or("*");
    format!("package={package};module={module}")
}

#[cfg(feature = "studio")]
pub(crate) fn module_fallback_contract()
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

#[cfg(feature = "studio")]
pub(crate) fn symbol_fallback_contract()
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

#[cfg(feature = "studio")]
pub(crate) fn example_fallback_contract()
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

#[cfg(feature = "studio")]
pub(crate) fn import_fallback_contract(
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

#[cfg(test)]
#[path = "../../../../tests/unit/analyzers/service/search/contracts.rs"]
mod tests;
