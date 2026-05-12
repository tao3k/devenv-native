//! `analyzers::service::search::symbol` owns Wendao service search symbol behavior.

use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
use crate::analyzers::cache::RepositorySearchArtifacts;
use crate::analyzers::compute_repository_saliency;
use crate::analyzers::{SymbolSearchHit, SymbolSearchQuery, SymbolSearchResult};

#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
use super::ranking::ranked_symbol_matches_with_artifacts;
use super::ranking::{RankedSearchRecord, ranked_symbol_matches};
use crate::analyzers::service::{
    analyze_repository_from_config_with_registry, bootstrap_builtin_registry,
};
use crate::analyzers::service::{
    backlinks_for, documents_backlink_lookup, hierarchy_segments_from_path, infer_ecosystem,
    projection_page_lookup, projection_pages_for, record_hierarchical_uri,
};

/// Build a symbol search result from normalized analysis records.
#[must_use]
pub fn build_symbol_search(
    query: &SymbolSearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> SymbolSearchResult {
    symbol_search_result_from_selected(
        query,
        analysis,
        ranked_symbol_matches(query.query.as_str(), &analysis.symbols, query.limit.max(1)),
    )
}
/// `build_symbol_search_with_artifacts` public function boundary for Wendao.

#[must_use]
#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
pub fn build_symbol_search_with_artifacts(
    query: &SymbolSearchQuery,
    analysis: &RepositoryAnalysisOutput,
    artifacts: &RepositorySearchArtifacts,
) -> SymbolSearchResult {
    symbol_search_result_from_selected(
        query,
        analysis,
        ranked_symbol_matches_with_artifacts(
            query.query.as_str(),
            &analysis.symbols,
            &artifacts.symbols_by_id,
            &artifacts.symbol_index,
            query.limit.max(1),
        ),
    )
}

fn symbol_search_result_from_selected(
    query: &SymbolSearchQuery,
    analysis: &RepositoryAnalysisOutput,
    selected: Vec<RankedSearchRecord<crate::analyzers::SymbolRecord>>,
) -> SymbolSearchResult {
    let backlink_lookup = documents_backlink_lookup(&analysis.relations, &analysis.docs);
    let projection_lookup = projection_page_lookup(analysis);
    let saliency_map = compute_repository_saliency(analysis);
    let symbols = selected
        .iter()
        .map(|candidate| candidate.item.clone())
        .collect::<Vec<_>>();
    let symbol_hits = selected
        .into_iter()
        .enumerate()
        .map(|(index, candidate)| {
            let normalized_score = candidate.score;
            let symbol = candidate.item;
            let audit_status = symbol.audit_status.clone();
            let verification_state = symbol.verification_state.clone().or_else(|| {
                audit_status
                    .as_ref()
                    .map(|status| match status.as_str() {
                        "verified" | "approved" => "verified".to_string(),
                        _ => "unverified".to_string(),
                    })
                    .map(Into::into)
            });
            let symbol_id = symbol.symbol_id.clone();
            let symbol_path = symbol.path.clone();
            let (implicit_backlinks, implicit_backlink_items) =
                backlinks_for(symbol_id.as_str(), &backlink_lookup);
            let saliency_score = saliency_map.get(symbol_id.as_str()).copied();

            SymbolSearchHit {
                symbol,
                score: Some(normalized_score),
                rank: Some(index + 1),
                saliency_score,
                hierarchical_uri: Some(record_hierarchical_uri(
                    query.repo_id.as_str(),
                    infer_ecosystem(query.repo_id.as_str()),
                    "api",
                    symbol_path.as_str(),
                    symbol_id.as_str(),
                )),
                hierarchy: hierarchy_segments_from_path(symbol_path.as_str()),
                implicit_backlinks,
                implicit_backlink_items,
                projection_page_ids: projection_pages_for(symbol_id.as_str(), &projection_lookup),
                audit_status: audit_status.as_ref().map(ToString::to_string),
                verification_state: verification_state.as_ref().map(ToString::to_string),
            }
        })
        .collect::<Vec<_>>();

    SymbolSearchResult {
        repo_id: query.repo_id.clone(),
        symbols,
        symbol_hits,
    }
}

/// Load configuration, analyze one repository, and return matching symbols.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn symbol_search_from_config_with_registry(
    query: &SymbolSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<SymbolSearchResult, RepoIntelligenceError> {
    let analysis =
        analyze_repository_from_config_with_registry(&query.repo_id, config_path, cwd, registry)?;
    Ok(build_symbol_search(query, &analysis))
}

/// Load configuration, analyze one repository, and return matching symbols.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn symbol_search_from_config(
    query: &SymbolSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<SymbolSearchResult, RepoIntelligenceError> {
    let registry = bootstrap_builtin_registry()?;
    symbol_search_from_config_with_registry(query, config_path, cwd, &registry)
}
