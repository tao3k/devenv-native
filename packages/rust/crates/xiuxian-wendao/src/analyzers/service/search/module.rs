//! `analyzers::service::search::module` owns Wendao service search module behavior.

use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
use crate::analyzers::cache::RepositorySearchArtifacts;
use crate::analyzers::compute_repository_saliency;
use crate::analyzers::{ModuleSearchHit, ModuleSearchQuery, ModuleSearchResult};

#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
use super::ranking::ranked_module_matches_with_artifacts;
use super::ranking::{RankedSearchRecord, ranked_module_matches};
use crate::analyzers::service::{
    analyze_repository_from_config_with_registry, bootstrap_builtin_registry,
};
use crate::analyzers::service::{
    backlinks_for, documents_backlink_lookup, hierarchy_segments_from_path, infer_ecosystem,
    projection_page_lookup, projection_pages_for, record_hierarchical_uri,
};

/// Build a module search result from normalized analysis records.
#[must_use]
pub fn build_module_search(
    query: &ModuleSearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> ModuleSearchResult {
    module_search_result_from_selected(
        query,
        analysis,
        ranked_module_matches(query.query.as_str(), &analysis.modules, query.limit.max(1)),
    )
}
/// `build_module_search_with_artifacts` public function boundary for Wendao.
#[must_use]
#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
pub fn build_module_search_with_artifacts(
    query: &ModuleSearchQuery,
    analysis: &RepositoryAnalysisOutput,
    artifacts: &RepositorySearchArtifacts,
) -> ModuleSearchResult {
    module_search_result_from_selected(
        query,
        analysis,
        ranked_module_matches_with_artifacts(
            query.query.as_str(),
            &analysis.modules,
            &artifacts.modules_by_id,
            &artifacts.module_index,
            query.limit.max(1),
        ),
    )
}

fn module_search_result_from_selected(
    query: &ModuleSearchQuery,
    analysis: &RepositoryAnalysisOutput,
    selected: Vec<RankedSearchRecord<crate::analyzers::ModuleRecord>>,
) -> ModuleSearchResult {
    let backlink_lookup = documents_backlink_lookup(&analysis.relations, &analysis.docs);
    let projection_lookup = projection_page_lookup(analysis);
    let saliency_map = compute_repository_saliency(analysis);
    let modules = selected
        .iter()
        .map(|candidate| candidate.item.clone())
        .collect::<Vec<_>>();
    let module_hits = selected
        .into_iter()
        .enumerate()
        .map(|(index, candidate)| {
            let normalized_score = candidate.score;
            let module = candidate.item;
            let module_id = module.module_id.clone();
            let module_path = module.path.clone();
            let (implicit_backlinks, implicit_backlink_items) =
                backlinks_for(module_id.as_str(), &backlink_lookup);
            let saliency_score = saliency_map.get(module_id.as_str()).copied();

            ModuleSearchHit {
                module,
                score: Some(normalized_score),
                rank: Some(index + 1),
                saliency_score,
                hierarchical_uri: Some(record_hierarchical_uri(
                    query.repo_id.as_str(),
                    infer_ecosystem(query.repo_id.as_str()),
                    "api",
                    module_path.as_str(),
                    module_id.as_str(),
                )),
                hierarchy: hierarchy_segments_from_path(module_path.as_str()),
                implicit_backlinks,
                implicit_backlink_items,
                projection_page_ids: projection_pages_for(module_id.as_str(), &projection_lookup),
            }
        })
        .collect::<Vec<_>>();

    ModuleSearchResult {
        repo_id: query.repo_id.clone(),
        modules,
        module_hits,
    }
}

/// Load configuration, analyze one repository, and return matching modules.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn module_search_from_config_with_registry(
    query: &ModuleSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<ModuleSearchResult, RepoIntelligenceError> {
    let analysis =
        analyze_repository_from_config_with_registry(&query.repo_id, config_path, cwd, registry)?;
    Ok(build_module_search(query, &analysis))
}

/// Load configuration, analyze one repository, and return matching modules.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn module_search_from_config(
    query: &ModuleSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<ModuleSearchResult, RepoIntelligenceError> {
    let registry = bootstrap_builtin_registry()?;
    module_search_from_config_with_registry(query, config_path, cwd, &registry)
}
