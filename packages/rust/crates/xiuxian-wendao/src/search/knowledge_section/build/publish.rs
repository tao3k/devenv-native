//! Test-support publication helper for knowledge-section index builds.

#[cfg(any(test, feature = "test-support"))]
use std::collections::BTreeMap;
#[cfg(any(test, feature = "test-support"))]
use std::path::Path;

#[cfg(any(test, feature = "test-support"))]
use crate::search::contracts::SearchProjectConfig;
#[cfg(any(test, feature = "test-support"))]
use crate::search::knowledge_section::build::orchestration::plan_knowledge_section_build;
#[cfg(any(test, feature = "test-support"))]
use crate::search::knowledge_section::build::types::KnowledgeSectionBuildError;
#[cfg(any(test, feature = "test-support"))]
use crate::search::knowledge_section::build::write::write_knowledge_section_epoch;
#[cfg(any(test, feature = "test-support"))]
use crate::search::knowledge_section::schema::projected_columns;
#[cfg(any(test, feature = "test-support"))]
use crate::search::{BeginBuildDecision, SearchCorpusKind, SearchPlaneService};
/// `publish_knowledge_sections_from_projects` public function boundary for Wendao.
#[cfg(any(test, feature = "test-support"))]
pub async fn publish_knowledge_sections_from_projects(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    fingerprint: &str,
) -> Result<(), KnowledgeSectionBuildError> {
    let lease = match service.coordinator().begin_build(
        SearchCorpusKind::KnowledgeSection,
        fingerprint,
        SearchCorpusKind::KnowledgeSection.schema_version(),
    ) {
        BeginBuildDecision::Started(lease) => lease,
        BeginBuildDecision::AlreadyReady(_) | BeginBuildDecision::AlreadyIndexing(_) => {
            return Ok(());
        }
    };
    let plan = plan_knowledge_section_build(
        service,
        project_root,
        config_root,
        projects,
        None,
        &BTreeMap::new(),
    );
    match write_knowledge_section_epoch(service, &lease, &plan).await {
        Ok(write) => {
            let prewarm_columns = projected_columns();
            service
                .prewarm_epoch_table(lease.corpus, lease.epoch, &prewarm_columns)
                .await?;
            service
                .publish_ready_and_maintain(&lease, write.row_count, write.fragment_count)
                .await;
            Ok(())
        }
        Err(error) => {
            service.coordinator().fail_build(
                &lease,
                format!("knowledge section epoch write failed: {error}"),
            );
            Err(KnowledgeSectionBuildError::Storage(error))
        }
    }
}
