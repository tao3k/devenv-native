//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
//! Coordinates search-plane query methods across repository, symbol, attachment, and vector owners.

use super::SearchPlaneService;
use crate::search::contracts::{ProjectConfigView, materialize_project_configs};
use crate::search::{
    AttachmentSearchError, KnowledgeSectionSearchError, LocalSymbolSearchError, ProjectScannedFile,
    ReferenceOccurrenceSearchError,
};
#[cfg(feature = "duckdb")]
use xiuxian_db_store::VectorStoreError;

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

    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    #[must_use]
    pub fn ensure_local_symbol_index_started(
        &self,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[impl ProjectConfigView],
    ) -> bool {
        let projects = materialize_project_configs(projects);
        crate::search::local_symbol::ensure_local_symbol_index_started(
            self,
            project_root,
            config_root,
            projects.as_slice(),
        )
    }

    /// Start or reuse the local-symbol index using a precomputed file scan.
    #[must_use]
    pub fn ensure_local_symbol_index_started_with_scanned_files(
        &self,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[impl ProjectConfigView],
        scanned_files: &[ProjectScannedFile],
    ) -> bool {
        let projects = materialize_project_configs(projects);
        crate::search::local_symbol::ensure_local_symbol_index_started_with_scanned_files(
            self,
            project_root,
            config_root,
            projects.as_slice(),
            scanned_files,
        )
    }

    /// Search local symbol hits in the active symbol index.
    ///
    /// # Errors
    ///
    /// Returns a local-symbol search error when the active publication cannot
    /// be queried or decoded.
    pub async fn search_local_symbols(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<crate::search::contracts::SourceSymbolHit>, LocalSymbolSearchError> {
        crate::search::local_symbol::search_local_symbols(self, query, limit).await
    }

    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    #[must_use]
    pub fn ensure_knowledge_section_index_started(
        &self,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[impl ProjectConfigView],
    ) -> bool {
        let projects = materialize_project_configs(projects);
        crate::search::knowledge_section::ensure_knowledge_section_index_started(
            self,
            project_root,
            config_root,
            projects.as_slice(),
        )
    }

    /// Start or reuse the knowledge-section index using a precomputed file scan.
    #[must_use]
    pub fn ensure_knowledge_section_index_started_with_scanned_files(
        &self,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[impl ProjectConfigView],
        scanned_files: &[ProjectScannedFile],
    ) -> bool {
        let projects = materialize_project_configs(projects);
        crate::search::knowledge_section::ensure_knowledge_section_index_started_with_scanned_files(
            self,
            project_root,
            config_root,
            projects.as_slice(),
            scanned_files,
        )
    }

    /// Search knowledge-section hits in the active note index.
    ///
    /// # Errors
    ///
    /// Returns a knowledge-section search error when the active publication
    /// cannot be queried or decoded.
    pub async fn search_knowledge_sections(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<crate::search::contracts::SearchHit>, KnowledgeSectionSearchError> {
        crate::search::knowledge_section::search_knowledge_sections(self, query, limit).await
    }

    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    #[must_use]
    pub fn ensure_attachment_index_started(
        &self,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[impl ProjectConfigView],
    ) -> bool {
        let projects = materialize_project_configs(projects);
        crate::search::attachment::ensure_attachment_index_started(
            self,
            project_root,
            config_root,
            projects.as_slice(),
        )
    }

    /// Start or reuse the attachment index using a precomputed file scan.
    #[must_use]
    pub fn ensure_attachment_index_started_with_scanned_files(
        &self,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[impl ProjectConfigView],
        scanned_files: &[ProjectScannedFile],
    ) -> bool {
        let projects = materialize_project_configs(projects);
        crate::search::attachment::ensure_attachment_index_started_with_scanned_files(
            self,
            project_root,
            config_root,
            projects.as_slice(),
            scanned_files,
        )
    }

    /// Search attachment hits in the active attachment index.
    ///
    /// # Errors
    ///
    /// Returns an attachment search error when the active publication cannot
    /// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
    /// be queried or decoded.
    pub async fn search_attachment_hits(
        &self,
        query: &str,
        limit: usize,
        extensions: &[String],
        kinds: &[crate::link_graph::LinkGraphAttachmentKind],
        case_sensitive: bool,
    ) -> Result<Vec<crate::search::contracts::AttachmentSearchHit>, AttachmentSearchError> {
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

    /// Autocomplete local symbols from the active symbol index.
    ///
    /// # Errors
    ///
    /// Returns a local-symbol search error when the active publication cannot
    /// be queried or decoded.
    pub async fn autocomplete_local_symbols(
        &self,
        prefix: &str,
        limit: usize,
    ) -> Result<Vec<crate::search::contracts::AutocompleteSuggestion>, LocalSymbolSearchError> {
        crate::search::local_symbol::autocomplete_local_symbols(self, prefix, limit).await
    }

    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    #[must_use]
    pub fn ensure_reference_occurrence_index_started(
        &self,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[impl ProjectConfigView],
    ) -> bool {
        let projects = materialize_project_configs(projects);
        crate::search::reference_occurrence::ensure_reference_occurrence_index_started(
            self,
            project_root,
            config_root,
            projects.as_slice(),
        )
    }

    /// Start or reuse the reference-occurrence index using a precomputed file scan.
    #[must_use]
    pub fn ensure_reference_occurrence_index_started_with_scanned_files(
        &self,
        project_root: &std::path::Path,
        config_root: &std::path::Path,
        projects: &[impl ProjectConfigView],
        scanned_files: &[ProjectScannedFile],
    ) -> bool {
        let projects = materialize_project_configs(projects);
        crate::search::reference_occurrence::ensure_reference_occurrence_index_started_with_scanned_files(
            self,
            project_root,
            config_root,
            projects.as_slice(),
            scanned_files,
        )
    }

    /// Search reference occurrences in the active reference index.
    ///
    /// # Errors
    ///
    /// Returns a reference-occurrence search error when the active publication
    /// cannot be queried or decoded.
    pub async fn search_reference_occurrences(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<crate::search::contracts::ReferenceSearchHit>, ReferenceOccurrenceSearchError>
    {
        crate::search::reference_occurrence::search_reference_occurrences(self, query, limit).await
    }
}
