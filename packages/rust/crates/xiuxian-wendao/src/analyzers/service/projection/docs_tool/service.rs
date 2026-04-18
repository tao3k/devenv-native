use std::path::{Path, PathBuf};

#[cfg(feature = "zhenfa-router")]
use crate::analyzers::PluginRegistry;
use crate::analyzers::RepoIntelligenceError;
#[cfg(feature = "zhenfa-router")]
use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::projection::{ProjectedPageIndexNode, ProjectionPageKind};
#[cfg(feature = "zhenfa-router")]
use crate::analyzers::service::projection::{
    build_docs_navigation, build_docs_page, build_docs_page_index_tree,
    build_docs_retrieval_context,
};
use crate::analyzers::service::projection::{
    docs_markdown_documents_from_config, docs_navigation_from_config, docs_page_from_config,
    docs_page_index_documents_from_config, docs_page_index_node_from_config,
    docs_page_index_tree_from_config, docs_page_index_tree_search_from_config,
    docs_page_index_trees_from_config, docs_retrieval_context_from_config, docs_search_from_config,
};
use crate::analyzers::{
    DocsMarkdownDocumentsQuery, DocsNavigationQuery, DocsNavigationResult,
    DocsPageIndexDocumentsQuery, DocsPageIndexDocumentsResult, DocsPageIndexNodeQuery,
    DocsPageIndexNodeResult, DocsPageIndexTreeQuery, DocsPageIndexTreeResult,
    DocsPageIndexTreeSearchQuery, DocsPageIndexTreeSearchResult, DocsPageIndexTreesQuery,
    DocsPageIndexTreesResult, DocsPageQuery, DocsPageResult, DocsRetrievalContextQuery,
    DocsRetrievalContextResult, DocsSearchQuery, DocsSearchResult,
};
#[cfg(feature = "zhenfa-router")]
use crate::analyzers::{RegisteredRepository, analyze_registered_repository_with_registry};

use super::{
    DocsDocumentSegmentResult, DocsNavigationOptions, DocsRetrievalContextOptions,
    build_document_segment,
};

/// Crate-local capability facade for docs/page-index operations.
///
/// This service stays parallel to `SearchQueryService`: it owns the in-process
/// docs capability surface, while gateway and CLI surfaces act as adapters.
#[derive(Clone, Debug)]
pub struct DocsToolService {
    project_root: PathBuf,
    repo_id: String,
    config_path: Option<PathBuf>,
}

impl DocsToolService {
    #[cfg(feature = "zhenfa-router")]
    fn with_registered_repository_analysis<T, F>(
        &self,
        repository: &RegisteredRepository,
        registry: &PluginRegistry,
        build: F,
    ) -> Result<T, RepoIntelligenceError>
    where
        F: FnOnce(&RepositoryAnalysisOutput) -> Result<T, RepoIntelligenceError>,
    {
        let analysis =
            analyze_registered_repository_with_registry(repository, self.project_root(), registry)?;
        build(&analysis)
    }

    /// Create a docs capability service for one project root and repository.
    #[must_use]
    pub fn new(project_root: impl Into<PathBuf>, repo_id: impl Into<String>) -> Self {
        Self {
            project_root: project_root.into(),
            repo_id: repo_id.into(),
            config_path: None,
        }
    }

    /// Create a docs capability service from one project root.
    #[must_use]
    pub fn from_project_root(project_root: impl Into<PathBuf>, repo_id: impl Into<String>) -> Self {
        Self::new(project_root, repo_id)
    }

    /// Override the config path used by config-backed docs capability calls.
    #[must_use]
    pub fn with_optional_config_path(mut self, config_path: Option<PathBuf>) -> Self {
        self.config_path = config_path;
        self
    }

    /// Borrow the project root used for capability calls.
    #[must_use]
    pub fn project_root(&self) -> &Path {
        self.project_root.as_path()
    }

    /// Borrow the repository identifier used for capability calls.
    #[must_use]
    pub fn repo_id(&self) -> &str {
        self.repo_id.as_str()
    }

    /// Borrow the optional config path used for capability calls.
    #[must_use]
    pub fn config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    /// Return one deterministic docs-facing projected page.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when repository analysis fails or the
    /// requested projected page is not present for the configured repository.
    pub fn get_document(&self, page_id: &str) -> Result<DocsPageResult, RepoIntelligenceError> {
        docs_page_from_config(
            &DocsPageQuery {
                repo_id: self.repo_id.clone(),
                page_id: page_id.to_string(),
            },
            self.config_path(),
            self.project_root(),
        )
    }

    /// Search docs-facing projected pages across one repository.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when repository analysis fails or the
    /// projected-page search cannot be constructed for the configured
    /// repository.
    pub fn search_documents(
        &self,
        query: &str,
        kind: Option<ProjectionPageKind>,
        limit: usize,
    ) -> Result<DocsSearchResult, RepoIntelligenceError> {
        docs_search_from_config(
            &DocsSearchQuery {
                repo_id: self.repo_id.clone(),
                query: query.to_string(),
                kind,
                limit,
            },
            self.config_path(),
            self.project_root(),
        )
    }

    #[cfg(feature = "zhenfa-router")]
    pub(crate) fn get_document_for_registered_repository(
        &self,
        page_id: &str,
        repository: &RegisteredRepository,
        registry: &PluginRegistry,
    ) -> Result<DocsPageResult, RepoIntelligenceError> {
        self.with_registered_repository_analysis(repository, registry, |analysis| {
            build_docs_page(
                &DocsPageQuery {
                    repo_id: repository.id.clone(),
                    page_id: page_id.to_string(),
                },
                analysis,
            )
        })
    }

    /// Return one deterministic docs-facing projected page-index tree.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when repository analysis fails, the
    /// requested page is not present, or page-index tree construction fails.
    pub fn get_document_structure(
        &self,
        page_id: &str,
    ) -> Result<DocsPageIndexTreeResult, RepoIntelligenceError> {
        docs_page_index_tree_from_config(
            &DocsPageIndexTreeQuery {
                repo_id: self.repo_id.clone(),
                page_id: page_id.to_string(),
            },
            self.config_path(),
            self.project_root(),
        )
    }

    /// Return one text-free docs-facing projected page-index tree for
    /// token-sensitive structure inspection.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when repository analysis fails, the
    /// requested page is not present, or page-index tree construction fails.
    pub fn get_document_structure_outline(
        &self,
        page_id: &str,
    ) -> Result<DocsPageIndexTreeResult, RepoIntelligenceError> {
        self.get_document_structure(page_id)
            .map(text_free_tree_result)
    }

    /// Return one repo-scoped text-free docs-facing projected page-index tree
    /// catalog for token-sensitive structure discovery.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when repository analysis fails or
    /// page-index tree construction fails.
    pub fn get_document_structure_catalog(
        &self,
    ) -> Result<DocsPageIndexTreesResult, RepoIntelligenceError> {
        docs_page_index_trees_from_config(
            &DocsPageIndexTreesQuery {
                repo_id: self.repo_id.clone(),
            },
            self.config_path(),
            self.project_root(),
        )
        .map(text_free_trees_result)
    }

    /// Return one precise docs-facing projected markdown segment reopened by a
    /// stable page id plus 1-based inclusive line range.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when repository analysis fails, the
    /// requested projected page is not present, or the requested line range is
    /// invalid for the rendered projected markdown document.
    pub fn get_document_segment(
        &self,
        page_id: &str,
        line_start: usize,
        line_end: usize,
    ) -> Result<DocsDocumentSegmentResult, RepoIntelligenceError> {
        let documents = docs_markdown_documents_from_config(
            &DocsMarkdownDocumentsQuery {
                repo_id: self.repo_id.clone(),
            },
            self.config_path(),
            self.project_root(),
        )?;
        let document = documents
            .documents
            .iter()
            .find(|document| document.page_id == page_id)
            .ok_or_else(|| RepoIntelligenceError::UnknownProjectedPage {
                repo_id: self.repo_id.clone(),
                page_id: page_id.to_string(),
            })?;
        build_document_segment(document, line_start, line_end)
    }

    /// Return one deterministic docs-facing projected page-index node.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when repository analysis fails, the
    /// requested page is not present, or the requested page-index node is not
    /// present for the projected page.
    pub fn get_document_node(
        &self,
        page_id: &str,
        node_id: &str,
    ) -> Result<DocsPageIndexNodeResult, RepoIntelligenceError> {
        docs_page_index_node_from_config(
            &DocsPageIndexNodeQuery {
                repo_id: self.repo_id.clone(),
                page_id: page_id.to_string(),
                node_id: node_id.to_string(),
            },
            self.config_path(),
            self.project_root(),
        )
    }

    /// Search docs-facing page-index nodes across one repository and return
    /// bounded deterministic candidate hits.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when repository analysis fails or
    /// projected page-index tree search construction fails.
    pub fn search_document_structure(
        &self,
        query: &str,
        kind: Option<ProjectionPageKind>,
        limit: usize,
    ) -> Result<DocsPageIndexTreeSearchResult, RepoIntelligenceError> {
        docs_page_index_tree_search_from_config(
            &DocsPageIndexTreeSearchQuery {
                repo_id: self.repo_id.clone(),
                query: query.to_string(),
                kind,
                limit: limit.max(1),
            },
            self.config_path(),
            self.project_root(),
        )
    }

    #[cfg(feature = "zhenfa-router")]
    pub(crate) fn get_document_structure_for_registered_repository(
        &self,
        page_id: &str,
        repository: &RegisteredRepository,
        registry: &PluginRegistry,
    ) -> Result<DocsPageIndexTreeResult, RepoIntelligenceError> {
        self.with_registered_repository_analysis(repository, registry, |analysis| {
            build_docs_page_index_tree(
                &DocsPageIndexTreeQuery {
                    repo_id: repository.id.clone(),
                    page_id: page_id.to_string(),
                },
                analysis,
            )
        })
    }

    /// Return repository-scoped markdown TOC/page-index documents for the
    /// configured repository.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when repository analysis fails or
    /// projected markdown cannot be parsed into page-index-ready documents.
    pub fn get_toc_documents(&self) -> Result<DocsPageIndexDocumentsResult, RepoIntelligenceError> {
        docs_page_index_documents_from_config(
            &DocsPageIndexDocumentsQuery {
                repo_id: self.repo_id.clone(),
            },
            self.config_path(),
            self.project_root(),
        )
    }

    /// Return one deterministic docs-facing navigation bundle using default
    /// navigation limits.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when repository analysis fails or the
    /// requested projected page, node, or family cluster is not present.
    pub fn get_navigation(
        &self,
        page_id: &str,
        node_id: Option<&str>,
    ) -> Result<DocsNavigationResult, RepoIntelligenceError> {
        self.get_navigation_with_options(
            page_id,
            DocsNavigationOptions {
                node_id: node_id.map(str::to_string),
                ..DocsNavigationOptions::default()
            },
        )
    }

    /// Return one deterministic docs-facing navigation bundle using explicit
    /// navigation options.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when repository analysis fails or the
    /// requested projected page, node, or family cluster is not present.
    pub fn get_navigation_with_options(
        &self,
        page_id: &str,
        options: DocsNavigationOptions,
    ) -> Result<DocsNavigationResult, RepoIntelligenceError> {
        let options = options.normalized();
        docs_navigation_from_config(
            &DocsNavigationQuery {
                repo_id: self.repo_id.clone(),
                page_id: page_id.to_string(),
                node_id: options.node_id,
                family_kind: options.family_kind,
                related_limit: options.related_limit,
                family_limit: options.family_limit,
            },
            self.config_path(),
            self.project_root(),
        )
    }

    #[cfg(feature = "zhenfa-router")]
    pub(crate) fn get_navigation_with_options_for_registered_repository(
        &self,
        page_id: &str,
        repository: &RegisteredRepository,
        registry: &PluginRegistry,
        options: DocsNavigationOptions,
    ) -> Result<DocsNavigationResult, RepoIntelligenceError> {
        let options = options.normalized();
        self.with_registered_repository_analysis(repository, registry, |analysis| {
            build_docs_navigation(
                &DocsNavigationQuery {
                    repo_id: repository.id.clone(),
                    page_id: page_id.to_string(),
                    node_id: options.node_id,
                    family_kind: options.family_kind,
                    related_limit: options.related_limit,
                    family_limit: options.family_limit,
                },
                analysis,
            )
        })
    }

    /// Return one deterministic docs-facing retrieval context using default
    /// related-page limits.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when repository analysis fails or the
    /// requested projected page or node is not present.
    pub fn get_retrieval_context(
        &self,
        page_id: &str,
        node_id: Option<&str>,
    ) -> Result<DocsRetrievalContextResult, RepoIntelligenceError> {
        self.get_retrieval_context_with_options(
            page_id,
            DocsRetrievalContextOptions {
                node_id: node_id.map(str::to_string),
                ..DocsRetrievalContextOptions::default()
            },
        )
    }

    /// Return one deterministic docs-facing retrieval context using explicit
    /// context options.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when repository analysis fails or the
    /// requested projected page or node is not present.
    pub fn get_retrieval_context_with_options(
        &self,
        page_id: &str,
        options: DocsRetrievalContextOptions,
    ) -> Result<DocsRetrievalContextResult, RepoIntelligenceError> {
        docs_retrieval_context_from_config(
            &DocsRetrievalContextQuery {
                repo_id: self.repo_id.clone(),
                page_id: page_id.to_string(),
                node_id: options.node_id,
                related_limit: options.related_limit,
            },
            self.config_path(),
            self.project_root(),
        )
    }

    #[cfg(feature = "zhenfa-router")]
    pub(crate) fn get_retrieval_context_with_options_for_registered_repository(
        &self,
        page_id: &str,
        repository: &RegisteredRepository,
        registry: &PluginRegistry,
        options: DocsRetrievalContextOptions,
    ) -> Result<DocsRetrievalContextResult, RepoIntelligenceError> {
        self.with_registered_repository_analysis(repository, registry, |analysis| {
            build_docs_retrieval_context(
                &DocsRetrievalContextQuery {
                    repo_id: repository.id.clone(),
                    page_id: page_id.to_string(),
                    node_id: options.node_id,
                    related_limit: options.related_limit,
                },
                analysis,
            )
        })
    }
}

fn text_free_tree_result(mut result: DocsPageIndexTreeResult) -> DocsPageIndexTreeResult {
    if let Some(tree) = result.tree.as_mut() {
        strip_text_from_nodes(tree.roots.as_mut_slice());
    }
    result
}

fn text_free_trees_result(mut result: DocsPageIndexTreesResult) -> DocsPageIndexTreesResult {
    for tree in &mut result.trees {
        strip_text_from_nodes(tree.roots.as_mut_slice());
    }
    result
}

fn strip_text_from_nodes(nodes: &mut [ProjectedPageIndexNode]) {
    for node in nodes {
        node.text.clear();
        strip_text_from_nodes(node.children.as_mut_slice());
    }
}

#[cfg(test)]
#[path = "../../../../../tests/unit/analyzers/service/projection/docs_tool/service.rs"]
mod tests;
