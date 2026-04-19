use std::sync::Arc;

use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::projection::ProjectionPageKind;
use crate::analyzers::{
    DocsNavigationResult, DocsPageIndexDocumentsResult, DocsPageIndexNodeResult,
    DocsPageIndexTreeResult, DocsPageIndexTreeSearchResult, DocsPageIndexTreesResult,
    DocsPageResult, DocsRetrievalContextResult, DocsSearchResult,
};

use super::{
    DocsDocumentSegmentResult, DocsNavigationOptions, DocsRetrievalContextOptions, DocsToolService,
};

/// Crate-local execution contract for docs capability calls.
pub(crate) trait DocsToolRuntime: Send + Sync {
    fn search_documents(
        &self,
        query: &str,
        kind: Option<ProjectionPageKind>,
        limit: usize,
    ) -> Result<DocsSearchResult, RepoIntelligenceError>;

    fn get_document(&self, page_id: &str) -> Result<DocsPageResult, RepoIntelligenceError>;

    fn get_page_index_tree(
        &self,
        page_id: &str,
    ) -> Result<DocsPageIndexTreeResult, RepoIntelligenceError>;

    fn get_page_index_outline(
        &self,
        page_id: &str,
    ) -> Result<DocsPageIndexTreeResult, RepoIntelligenceError>;

    fn get_page_index(&self) -> Result<DocsPageIndexTreesResult, RepoIntelligenceError>;

    fn get_document_segment(
        &self,
        page_id: &str,
        line_start: usize,
        line_end: usize,
    ) -> Result<DocsDocumentSegmentResult, RepoIntelligenceError>;

    fn get_document_node(
        &self,
        page_id: &str,
        node_id: &str,
    ) -> Result<DocsPageIndexNodeResult, RepoIntelligenceError>;

    fn search_page_index(
        &self,
        query: &str,
        kind: Option<ProjectionPageKind>,
        limit: usize,
    ) -> Result<DocsPageIndexTreeSearchResult, RepoIntelligenceError>;

    fn get_toc_documents(&self) -> Result<DocsPageIndexDocumentsResult, RepoIntelligenceError>;

    fn get_navigation_with_options(
        &self,
        page_id: &str,
        options: DocsNavigationOptions,
    ) -> Result<DocsNavigationResult, RepoIntelligenceError>;

    fn get_retrieval_context_with_options(
        &self,
        page_id: &str,
        options: DocsRetrievalContextOptions,
    ) -> Result<DocsRetrievalContextResult, RepoIntelligenceError>;
}

impl DocsToolRuntime for DocsToolService {
    fn search_documents(
        &self,
        query: &str,
        kind: Option<ProjectionPageKind>,
        limit: usize,
    ) -> Result<DocsSearchResult, RepoIntelligenceError> {
        DocsToolService::search_documents(self, query, kind, limit)
    }

    fn get_document(&self, page_id: &str) -> Result<DocsPageResult, RepoIntelligenceError> {
        DocsToolService::get_document(self, page_id)
    }

    fn get_page_index_tree(
        &self,
        page_id: &str,
    ) -> Result<DocsPageIndexTreeResult, RepoIntelligenceError> {
        DocsToolService::get_page_index_tree(self, page_id)
    }

    fn get_page_index_outline(
        &self,
        page_id: &str,
    ) -> Result<DocsPageIndexTreeResult, RepoIntelligenceError> {
        DocsToolService::get_page_index_outline(self, page_id)
    }

    fn get_page_index(&self) -> Result<DocsPageIndexTreesResult, RepoIntelligenceError> {
        DocsToolService::get_page_index(self)
    }

    fn get_document_segment(
        &self,
        page_id: &str,
        line_start: usize,
        line_end: usize,
    ) -> Result<DocsDocumentSegmentResult, RepoIntelligenceError> {
        DocsToolService::get_document_segment(self, page_id, line_start, line_end)
    }

    fn get_document_node(
        &self,
        page_id: &str,
        node_id: &str,
    ) -> Result<DocsPageIndexNodeResult, RepoIntelligenceError> {
        DocsToolService::get_document_node(self, page_id, node_id)
    }

    fn search_page_index(
        &self,
        query: &str,
        kind: Option<ProjectionPageKind>,
        limit: usize,
    ) -> Result<DocsPageIndexTreeSearchResult, RepoIntelligenceError> {
        DocsToolService::search_page_index(self, query, kind, limit)
    }

    fn get_toc_documents(&self) -> Result<DocsPageIndexDocumentsResult, RepoIntelligenceError> {
        DocsToolService::get_toc_documents(self)
    }

    fn get_navigation_with_options(
        &self,
        page_id: &str,
        options: DocsNavigationOptions,
    ) -> Result<DocsNavigationResult, RepoIntelligenceError> {
        DocsToolService::get_navigation_with_options(self, page_id, options)
    }

    fn get_retrieval_context_with_options(
        &self,
        page_id: &str,
        options: DocsRetrievalContextOptions,
    ) -> Result<DocsRetrievalContextResult, RepoIntelligenceError> {
        DocsToolService::get_retrieval_context_with_options(self, page_id, options)
    }
}

#[derive(Clone)]
pub(crate) struct DocsToolRuntimeHandle {
    inner: Arc<dyn DocsToolRuntime>,
}

impl DocsToolRuntimeHandle {
    #[cfg(test)]
    #[must_use]
    pub(crate) fn new(inner: Arc<dyn DocsToolRuntime>) -> Self {
        Self { inner }
    }

    #[must_use]
    pub(crate) fn inner(&self) -> Arc<dyn DocsToolRuntime> {
        Arc::clone(&self.inner)
    }
}
