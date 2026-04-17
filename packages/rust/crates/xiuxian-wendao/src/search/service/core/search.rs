use super::types::SearchPlaneService;
use crate::gateway::studio::types::UiProjectConfig;
use crate::search::{
    AttachmentSearchError, KnowledgeSectionSearchError, LocalSymbolSearchError, ProjectScannedFile,
    ReferenceOccurrenceSearchError,
};
#[cfg(feature = "duckdb")]
use xiuxian_vector_store::VectorStoreError;

impl SearchPlaneService {
    #[cfg(feature = "duckdb")]
    pub(crate) fn repo_parquet_query_engine(
        &self,
    ) -> Result<crate::duckdb::ParquetQueryEngine, VectorStoreError> {
        if let Some(engine) = self.parquet_query_engine.get() {
            return Ok(engine.clone());
        }

        let engine = crate::duckdb::ParquetQueryEngine::configured()?;

        let _ = self.parquet_query_engine.set(engine.clone());
        Ok(engine)
    }

    #[cfg(not(feature = "duckdb"))]
    pub(crate) fn repo_parquet_query_engine(&self) -> crate::duckdb::ParquetQueryEngine {
        if let Some(engine) = self.parquet_query_engine.get() {
            return engine.clone();
        }

        let engine =
            crate::duckdb::ParquetQueryEngine::configured(self.datafusion_query_engine().clone());
        let _ = self.parquet_query_engine.set(engine.clone());
        engine
    }

    #[cfg(test)]
    pub(crate) fn ensure_local_symbol_index_started(
        &self,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[UiProjectConfig],
    ) -> bool {
        crate::search::local_symbol::ensure_local_symbol_index_started(
            self,
            project_root,
            config_root,
            projects,
        )
    }

    pub(crate) fn ensure_local_symbol_index_started_with_scanned_files(
        &self,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[UiProjectConfig],
        scanned_files: &[ProjectScannedFile],
    ) -> bool {
        crate::search::local_symbol::ensure_local_symbol_index_started_with_scanned_files(
            self,
            project_root,
            config_root,
            projects,
            scanned_files,
        )
    }

    pub(crate) async fn search_local_symbols(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<crate::gateway::studio::types::AstSearchHit>, LocalSymbolSearchError> {
        crate::search::local_symbol::search_local_symbols(self, query, limit).await
    }

    #[cfg(test)]
    pub(crate) fn ensure_knowledge_section_index_started(
        &self,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[UiProjectConfig],
    ) -> bool {
        crate::search::knowledge_section::ensure_knowledge_section_index_started(
            self,
            project_root,
            config_root,
            projects,
        )
    }

    pub(crate) fn ensure_knowledge_section_index_started_with_scanned_files(
        &self,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[UiProjectConfig],
        scanned_files: &[ProjectScannedFile],
    ) -> bool {
        crate::search::knowledge_section::ensure_knowledge_section_index_started_with_scanned_files(
            self,
            project_root,
            config_root,
            projects,
            scanned_files,
        )
    }

    pub(crate) async fn search_knowledge_sections(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<crate::gateway::studio::types::SearchHit>, KnowledgeSectionSearchError> {
        crate::search::knowledge_section::search_knowledge_sections(self, query, limit).await
    }

    #[cfg(test)]
    pub(crate) fn ensure_attachment_index_started(
        &self,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[UiProjectConfig],
    ) -> bool {
        crate::search::attachment::ensure_attachment_index_started(
            self,
            project_root,
            config_root,
            projects,
        )
    }

    pub(crate) fn ensure_attachment_index_started_with_scanned_files(
        &self,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[UiProjectConfig],
        scanned_files: &[ProjectScannedFile],
    ) -> bool {
        crate::search::attachment::ensure_attachment_index_started_with_scanned_files(
            self,
            project_root,
            config_root,
            projects,
            scanned_files,
        )
    }

    pub(crate) async fn search_attachment_hits(
        &self,
        query: &str,
        limit: usize,
        extensions: &[String],
        kinds: &[crate::link_graph::LinkGraphAttachmentKind],
        case_sensitive: bool,
    ) -> Result<Vec<crate::gateway::studio::types::AttachmentSearchHit>, AttachmentSearchError>
    {
        crate::search::attachment::search_attachment_hits(
            self,
            query,
            limit,
            extensions,
            kinds,
            case_sensitive,
        )
        .await
    }

    pub(crate) async fn autocomplete_local_symbols(
        &self,
        prefix: &str,
        limit: usize,
    ) -> Result<Vec<crate::gateway::studio::types::AutocompleteSuggestion>, LocalSymbolSearchError>
    {
        crate::search::local_symbol::autocomplete_local_symbols(self, prefix, limit).await
    }

    #[cfg(test)]
    pub(crate) fn ensure_reference_occurrence_index_started(
        &self,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[UiProjectConfig],
    ) -> bool {
        crate::search::reference_occurrence::ensure_reference_occurrence_index_started(
            self,
            project_root,
            config_root,
            projects,
        )
    }

    pub(crate) fn ensure_reference_occurrence_index_started_with_scanned_files(
        &self,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[UiProjectConfig],
        scanned_files: &[ProjectScannedFile],
    ) -> bool {
        crate::search::reference_occurrence::ensure_reference_occurrence_index_started_with_scanned_files(
            self,
            project_root,
            config_root,
            projects,
            scanned_files,
        )
    }

    pub(crate) async fn search_reference_occurrences(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<
        Vec<crate::gateway::studio::types::ReferenceSearchHit>,
        ReferenceOccurrenceSearchError,
    > {
        crate::search::reference_occurrence::search_reference_occurrences(self, query, limit).await
    }
}
