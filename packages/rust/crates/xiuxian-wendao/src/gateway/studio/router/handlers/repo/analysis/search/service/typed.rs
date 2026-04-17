use std::sync::Arc;

use crate::analyzers::{
    ExampleSearchResult, ModuleSearchResult, SymbolSearchResult, example_fallback_contract,
    module_fallback_contract, symbol_fallback_contract,
};
use crate::gateway::studio::router::{GatewayState, StudioApiError};
use crate::query_core::{
    repo_entity_example_results_contract, repo_entity_module_results_contract,
    repo_entity_symbol_results_contract,
};

use super::execution::{
    RepoAnalysisSearchSpec, RepoAnalysisTypedSearchContract, run_typed_repo_analysis_search,
};

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
