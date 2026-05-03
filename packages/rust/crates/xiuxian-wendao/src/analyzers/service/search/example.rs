use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
use crate::analyzers::cache::RepositorySearchArtifacts;
use crate::analyzers::compute_repository_saliency;
use crate::analyzers::{ExampleSearchHit, ExampleSearchQuery, ExampleSearchResult};

use super::documents::build_example_metadata_lookup;
#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
use super::ranking::ranked_example_matches_with_artifacts;
use super::ranking::{RankedSearchRecord, ranked_example_matches};
use crate::analyzers::service::{
    analyze_repository_from_config_with_registry, bootstrap_builtin_registry,
};
use crate::analyzers::service::{
    backlinks_for, documents_backlink_lookup, hierarchy_segments_from_path, infer_ecosystem,
    projection_page_lookup, projection_pages_for, record_hierarchical_uri,
};

/// Build an example search result from normalized analysis records.
#[must_use]
pub fn build_example_search(
    query: &ExampleSearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> ExampleSearchResult {
    let metadata_lookup = build_example_metadata_lookup(analysis);
    example_search_result_from_selected(
        query,
        analysis,
        ranked_example_matches(
            query.query.as_str(),
            &analysis.examples,
            &metadata_lookup,
            query.limit.max(1),
        ),
    )
}

#[must_use]
#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
pub fn build_example_search_with_artifacts(
    query: &ExampleSearchQuery,
    analysis: &RepositoryAnalysisOutput,
    artifacts: &RepositorySearchArtifacts,
) -> ExampleSearchResult {
    example_search_result_from_selected(
        query,
        analysis,
        ranked_example_matches_with_artifacts(
            query.query.as_str(),
            &analysis.examples,
            &artifacts.example_metadata,
            &artifacts.examples_by_id,
            &artifacts.example_index,
            query.limit.max(1),
        ),
    )
}

fn example_search_result_from_selected(
    query: &ExampleSearchQuery,
    analysis: &RepositoryAnalysisOutput,
    selected: Vec<RankedSearchRecord<crate::analyzers::ExampleRecord>>,
) -> ExampleSearchResult {
    let backlink_lookup = documents_backlink_lookup(&analysis.relations, &analysis.docs);
    let projection_lookup = projection_page_lookup(analysis);
    let saliency_map = compute_repository_saliency(analysis);
    let examples = selected
        .iter()
        .map(|candidate| candidate.item.clone())
        .collect::<Vec<_>>();
    let example_hits = selected
        .into_iter()
        .enumerate()
        .map(|(index, candidate)| {
            let normalized_score = candidate.score;
            let example = candidate.item;
            let example_id = example.example_id.clone();
            let example_path = example.path.clone();
            let (implicit_backlinks, implicit_backlink_items) =
                backlinks_for(example_id.as_str(), &backlink_lookup);
            let saliency_score = saliency_map.get(example_id.as_str()).copied();

            ExampleSearchHit {
                example,
                score: Some(normalized_score),
                rank: Some(index + 1),
                saliency_score,
                hierarchical_uri: Some(record_hierarchical_uri(
                    query.repo_id.as_str(),
                    infer_ecosystem(query.repo_id.as_str()),
                    "examples",
                    example_path.as_str(),
                    example_id.as_str(),
                )),
                hierarchy: hierarchy_segments_from_path(example_path.as_str()),
                implicit_backlinks,
                implicit_backlink_items,
                projection_page_ids: projection_pages_for(example_id.as_str(), &projection_lookup),
            }
        })
        .collect::<Vec<_>>();

    ExampleSearchResult {
        repo_id: query.repo_id.clone(),
        examples,
        example_hits,
    }
}

/// Load configuration, analyze one repository, and return matching examples.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn example_search_from_config_with_registry(
    query: &ExampleSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<ExampleSearchResult, RepoIntelligenceError> {
    let analysis =
        analyze_repository_from_config_with_registry(&query.repo_id, config_path, cwd, registry)?;
    Ok(build_example_search(query, &analysis))
}

/// Load configuration, analyze one repository, and return matching examples.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails.
pub fn example_search_from_config(
    query: &ExampleSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<ExampleSearchResult, RepoIntelligenceError> {
    let registry = bootstrap_builtin_registry()?;
    example_search_from_config_with_registry(query, config_path, cwd, &registry)
}
