use std::path::Path;

use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::{
    build_projected_page_index_documents, build_projected_page_index_node,
    build_projected_page_index_tree, build_projected_page_index_tree_search,
    build_projected_page_index_trees,
};
use crate::analyzers::{
    DocsPageIndexDocumentsQuery, DocsPageIndexDocumentsResult, DocsPageIndexNodeQuery,
    DocsPageIndexNodeResult, DocsPageIndexTreeQuery, DocsPageIndexTreeResult,
    DocsPageIndexTreeSearchQuery, DocsPageIndexTreeSearchResult, DocsPageIndexTreesQuery,
    DocsPageIndexTreesResult, RepoProjectedPageIndexNodeQuery, RepoProjectedPageIndexNodeResult,
    RepoProjectedPageIndexTreeQuery, RepoProjectedPageIndexTreeResult,
    RepoProjectedPageIndexTreeSearchQuery, RepoProjectedPageIndexTreeSearchResult,
    RepoProjectedPageIndexTreesQuery, RepoProjectedPageIndexTreesResult,
};

use super::registry::{with_bootstrapped_repository_analysis, with_repository_analysis};

/// Build one docs-facing deterministic projected page-index tree from normalized analysis records.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page is
/// not present in the analysis output, or another [`RepoIntelligenceError`] when projected page
/// markdown cannot be parsed into page-index trees.
pub fn build_docs_page_index_tree(
    query: &DocsPageIndexTreeQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<DocsPageIndexTreeResult, RepoIntelligenceError> {
    build_repo_projected_page_index_tree(
        &RepoProjectedPageIndexTreeQuery {
            repo_id: query.repo_id.clone(),
            page_id: query.page_id.clone(),
        },
        analysis,
    )
}

/// Load configuration, analyze one repository, and return one docs-facing deterministic
/// projected page-index tree.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails, the requested projected page
/// identifier is not present for the repository, or projected page-index tree construction fails.
pub fn docs_page_index_tree_from_config_with_registry(
    query: &DocsPageIndexTreeQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsPageIndexTreeResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_docs_page_index_tree(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return one docs-facing deterministic
/// projected page-index tree.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails, the requested projected page
/// identifier is not present for the repository, or projected page-index tree construction fails.
pub fn docs_page_index_tree_from_config(
    query: &DocsPageIndexTreeQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsPageIndexTreeResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_docs_page_index_tree(query, analysis)
    })
}

/// Build deterministic docs-facing projected page-index documents from normalized analysis
/// records.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when projected page markdown cannot be parsed into
/// page-index-ready documents.
pub fn build_docs_page_index_documents(
    query: &DocsPageIndexDocumentsQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<DocsPageIndexDocumentsResult, RepoIntelligenceError> {
    Ok(DocsPageIndexDocumentsResult {
        repo_id: query.repo_id.clone(),
        documents: build_projected_page_index_documents(analysis)?,
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing projected
/// page-index documents.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis or projected page-index document
/// construction fails.
pub fn docs_page_index_documents_from_config_with_registry(
    query: &DocsPageIndexDocumentsQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsPageIndexDocumentsResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_docs_page_index_documents(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing projected
/// page-index documents.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis or projected page-index document
/// construction fails.
pub fn docs_page_index_documents_from_config(
    query: &DocsPageIndexDocumentsQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsPageIndexDocumentsResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_docs_page_index_documents(query, analysis)
    })
}

/// Build one docs-facing deterministic projected page-index node from normalized analysis records.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPageIndexNode`] when the requested projected
/// page-index node is not present in the analysis output.
pub fn build_docs_page_index_node(
    query: &DocsPageIndexNodeQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<DocsPageIndexNodeResult, RepoIntelligenceError> {
    build_repo_projected_page_index_node(
        &RepoProjectedPageIndexNodeQuery {
            repo_id: query.repo_id.clone(),
            page_id: query.page_id.clone(),
            node_id: query.node_id.clone(),
        },
        analysis,
    )
}

/// Load configuration, analyze one repository, and return one docs-facing deterministic
/// projected page-index node.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page-index node identifier is not present for the repository.
pub fn docs_page_index_node_from_config_with_registry(
    query: &DocsPageIndexNodeQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsPageIndexNodeResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_docs_page_index_node(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return one docs-facing deterministic
/// projected page-index node.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page-index node identifier is not present for the repository.
pub fn docs_page_index_node_from_config(
    query: &DocsPageIndexNodeQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsPageIndexNodeResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_docs_page_index_node(query, analysis)
    })
}

/// Build deterministic docs-facing projected page-index tree search results from normalized
/// analysis records.
#[must_use]
pub fn build_docs_page_index_tree_search(
    query: &DocsPageIndexTreeSearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> DocsPageIndexTreeSearchResult {
    build_repo_projected_page_index_tree_search(
        &RepoProjectedPageIndexTreeSearchQuery {
            repo_id: query.repo_id.clone(),
            query: query.query.clone(),
            kind: query.kind,
            limit: query.limit,
        },
        analysis,
    )
}

/// Load configuration, analyze one repository, and return deterministic docs-facing projected
/// page-index tree search results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or projected page-index tree
/// construction fails.
pub fn docs_page_index_tree_search_from_config_with_registry(
    query: &DocsPageIndexTreeSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsPageIndexTreeSearchResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        Ok(build_docs_page_index_tree_search(query, analysis))
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing projected
/// page-index tree search results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or projected page-index tree
/// construction fails.
pub fn docs_page_index_tree_search_from_config(
    query: &DocsPageIndexTreeSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsPageIndexTreeSearchResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        Ok(build_docs_page_index_tree_search(query, analysis))
    })
}

/// Build deterministic docs-facing projected page-index trees from normalized analysis records.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when projected page markdown cannot be parsed into
/// page-index trees.
pub fn build_docs_page_index_trees(
    query: &DocsPageIndexTreesQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<DocsPageIndexTreesResult, RepoIntelligenceError> {
    build_repo_projected_page_index_trees(
        &RepoProjectedPageIndexTreesQuery {
            repo_id: query.repo_id.clone(),
        },
        analysis,
    )
}

/// Load configuration, analyze one repository, and return deterministic docs-facing projected
/// page-index trees.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis or projected page-index tree
/// construction fails.
pub fn docs_page_index_trees_from_config_with_registry(
    query: &DocsPageIndexTreesQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<DocsPageIndexTreesResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_docs_page_index_trees(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return deterministic docs-facing projected
/// page-index trees.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis or projected page-index tree
/// construction fails.
pub fn docs_page_index_trees_from_config(
    query: &DocsPageIndexTreesQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<DocsPageIndexTreesResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_docs_page_index_trees(query, analysis)
    })
}

/// Build one deterministic projected page-index tree from normalized analysis records.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPage`] when the requested projected page is
/// not present in the analysis output, or another [`RepoIntelligenceError`] when projected page
/// markdown cannot be parsed into page-index trees.
pub fn build_repo_projected_page_index_tree(
    query: &RepoProjectedPageIndexTreeQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedPageIndexTreeResult, RepoIntelligenceError> {
    build_projected_page_index_tree(query, analysis)
}

/// Load configuration, analyze one repository, and return one deterministic projected page-index tree.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails, the requested projected page
/// identifier is not present for the repository, or projected page-index tree construction fails.
pub fn repo_projected_page_index_tree_from_config_with_registry(
    query: &RepoProjectedPageIndexTreeQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoProjectedPageIndexTreeResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_repo_projected_page_index_tree(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return one deterministic projected page-index tree.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails, the requested projected page
/// identifier is not present for the repository, or projected page-index tree construction fails.
pub fn repo_projected_page_index_tree_from_config(
    query: &RepoProjectedPageIndexTreeQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoProjectedPageIndexTreeResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_repo_projected_page_index_tree(query, analysis)
    })
}

/// Build one deterministic projected page-index node from normalized analysis records.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError::UnknownProjectedPageIndexNode`] when the requested projected
/// page-index node is not present in the analysis output.
pub fn build_repo_projected_page_index_node(
    query: &RepoProjectedPageIndexNodeQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedPageIndexNodeResult, RepoIntelligenceError> {
    build_projected_page_index_node(query, analysis)
}

/// Load configuration, analyze one repository, and return one deterministic projected page-index node.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page-index node identifier is not present for the repository.
pub fn repo_projected_page_index_node_from_config_with_registry(
    query: &RepoProjectedPageIndexNodeQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoProjectedPageIndexNodeResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_repo_projected_page_index_node(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return one deterministic projected page-index node.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or the requested projected
/// page-index node identifier is not present for the repository.
pub fn repo_projected_page_index_node_from_config(
    query: &RepoProjectedPageIndexNodeQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoProjectedPageIndexNodeResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_repo_projected_page_index_node(query, analysis)
    })
}

/// Build deterministic projected page-index tree search results from normalized analysis records.
#[must_use]
pub fn build_repo_projected_page_index_tree_search(
    query: &RepoProjectedPageIndexTreeSearchQuery,
    analysis: &RepositoryAnalysisOutput,
) -> RepoProjectedPageIndexTreeSearchResult {
    build_projected_page_index_tree_search(query, analysis)
}

/// Load configuration, analyze one repository, and return deterministic projected page-index tree search results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or projected page-index tree
/// construction fails.
pub fn repo_projected_page_index_tree_search_from_config_with_registry(
    query: &RepoProjectedPageIndexTreeSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoProjectedPageIndexTreeSearchResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        Ok(build_repo_projected_page_index_tree_search(query, analysis))
    })
}

/// Load configuration, analyze one repository, and return deterministic projected page-index tree search results.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis fails or projected page-index tree
/// construction fails.
pub fn repo_projected_page_index_tree_search_from_config(
    query: &RepoProjectedPageIndexTreeSearchQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoProjectedPageIndexTreeSearchResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        Ok(build_repo_projected_page_index_tree_search(query, analysis))
    })
}

/// Build deterministic projected page-index trees from normalized analysis records.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when projected page markdown cannot be parsed into
/// page-index trees.
pub fn build_repo_projected_page_index_trees(
    query: &RepoProjectedPageIndexTreesQuery,
    analysis: &RepositoryAnalysisOutput,
) -> Result<RepoProjectedPageIndexTreesResult, RepoIntelligenceError> {
    Ok(RepoProjectedPageIndexTreesResult {
        repo_id: query.repo_id.clone(),
        trees: build_projected_page_index_trees(analysis)?,
    })
}

/// Load configuration, analyze one repository, and return deterministic projected page-index trees.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis or projected page-index tree
/// construction fails.
pub fn repo_projected_page_index_trees_from_config_with_registry(
    query: &RepoProjectedPageIndexTreesQuery,
    config_path: Option<&Path>,
    cwd: &Path,
    registry: &PluginRegistry,
) -> Result<RepoProjectedPageIndexTreesResult, RepoIntelligenceError> {
    with_repository_analysis(&query.repo_id, config_path, cwd, registry, |analysis| {
        build_repo_projected_page_index_trees(query, analysis)
    })
}

/// Load configuration, analyze one repository, and return deterministic projected page-index trees.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository analysis or projected page-index tree
/// construction fails.
pub fn repo_projected_page_index_trees_from_config(
    query: &RepoProjectedPageIndexTreesQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoProjectedPageIndexTreesResult, RepoIntelligenceError> {
    with_bootstrapped_repository_analysis(&query.repo_id, config_path, cwd, |analysis| {
        build_repo_projected_page_index_trees(query, analysis)
    })
}
