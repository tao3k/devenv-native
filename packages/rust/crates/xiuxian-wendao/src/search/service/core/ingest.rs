#[cfg(any(test, feature = "test-support"))]
use std::path::Path;

#[cfg(any(test, feature = "test-support"))]
use super::types::SearchPlaneService;
#[cfg(any(test, feature = "test-support"))]
use crate::search::attachment::AttachmentBuildError;
#[cfg(any(test, feature = "test-support"))]
use crate::search::contracts::{AstSearchHit, UiProjectConfig};
#[cfg(any(test, feature = "test-support"))]
use crate::search::knowledge_section::KnowledgeSectionBuildError;
#[cfg(any(test, feature = "test-support"))]
use crate::search::local_symbol::LocalSymbolBuildError;
#[cfg(any(test, feature = "test-support"))]
use crate::search::reference_occurrence::ReferenceOccurrenceBuildError;

#[cfg(any(test, feature = "test-support"))]
impl SearchPlaneService {
    /// Publish local symbol hits into the test search plane.
    ///
    /// # Errors
    ///
    /// Returns a local-symbol build error when the hits cannot be written to
    /// the active test search plane.
    pub async fn publish_local_symbol_hits(
        &self,
        fingerprint: &str,
        hits: &[AstSearchHit],
    ) -> Result<(), LocalSymbolBuildError> {
        crate::search::local_symbol::publish_local_symbol_hits(self, fingerprint, hits).await
    }

    /// Publish reference occurrence hits for test projects.
    ///
    /// # Errors
    ///
    /// Returns a reference-occurrence build error when project references
    /// cannot be scanned or published.
    pub async fn publish_reference_occurrences_from_projects(
        &self,
        project_root: &Path,
        config_root: &Path,
        projects: &[UiProjectConfig],
        fingerprint: &str,
    ) -> Result<(), ReferenceOccurrenceBuildError> {
        crate::search::reference_occurrence::publish_reference_occurrences_from_projects(
            self,
            project_root,
            config_root,
            projects,
            fingerprint,
        )
        .await
    }

    /// Publish attachment hits for test projects.
    ///
    /// # Errors
    ///
    /// Returns an attachment build error when attachment metadata cannot be
    /// scanned or published.
    pub async fn publish_attachments_from_projects(
        &self,
        project_root: &Path,
        config_root: &Path,
        projects: &[UiProjectConfig],
        fingerprint: &str,
    ) -> Result<(), AttachmentBuildError> {
        crate::search::attachment::publish_attachments_from_projects(
            self,
            project_root,
            config_root,
            projects,
            fingerprint,
        )
        .await
    }

    /// Publish knowledge section rows for test projects.
    ///
    /// # Errors
    ///
    /// Returns a knowledge-section build error when note sections cannot be
    /// scanned or published.
    pub async fn publish_knowledge_sections_from_projects(
        &self,
        project_root: &Path,
        config_root: &Path,
        projects: &[UiProjectConfig],
        fingerprint: &str,
    ) -> Result<(), KnowledgeSectionBuildError> {
        crate::search::knowledge_section::publish_knowledge_sections_from_projects(
            self,
            project_root,
            config_root,
            projects,
            fingerprint,
        )
        .await
    }
}
