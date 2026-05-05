use std::path::Path;

use tokio::runtime::Handle;

use crate::search::cache::SearchPlaneFileFingerprintScope;
use crate::search::contracts::SearchProjectConfig;
#[cfg(any(test, feature = "test-support"))]
use crate::search::reference_occurrence::build::ReferenceOccurrenceBuildError;
use crate::search::{
    BeginBuildDecision, ProjectScannedFile, SearchCorpusKind, SearchPlaneService,
    fingerprint_source_projects_from_scanned_files,
};

#[cfg(any(test, feature = "test-support"))]
use crate::search::reference_occurrence::build::plan_reference_occurrence_build;
use crate::search::reference_occurrence::build::{
    plan_reference_occurrence_build_with_scanned_files, write_reference_occurrence_epoch,
};

#[cfg(any(test, feature = "test-support"))]
pub fn ensure_reference_occurrence_index_started(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
) -> bool {
    if projects.is_empty() {
        return false;
    }

    let (fingerprint, scanned_files) = service
        .fingerprint_source_projects_with_repeat_work_details(
            "reference_occurrence.fingerprint",
            project_root,
            config_root,
            projects,
        );
    ensure_reference_occurrence_index_started_with_fingerprint_and_scanned_files(
        service,
        project_root,
        config_root,
        projects,
        fingerprint,
        scanned_files,
    )
}

pub fn ensure_reference_occurrence_index_started_with_scanned_files(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    scanned_files: &[ProjectScannedFile],
) -> bool {
    if projects.is_empty() {
        return false;
    }

    let fingerprint = fingerprint_source_projects_from_scanned_files(
        project_root,
        config_root,
        projects,
        scanned_files,
    );
    ensure_reference_occurrence_index_started_with_fingerprint_and_scanned_files(
        service,
        project_root,
        config_root,
        projects,
        fingerprint,
        scanned_files.to_vec(),
    )
}

fn ensure_reference_occurrence_index_started_with_fingerprint_and_scanned_files(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    fingerprint: String,
    scanned_files: Vec<ProjectScannedFile>,
) -> bool {
    let decision = service.coordinator().begin_build(
        SearchCorpusKind::ReferenceOccurrence,
        fingerprint,
        SearchCorpusKind::ReferenceOccurrence.schema_version(),
    );
    let BeginBuildDecision::Started(lease) = decision else {
        return false;
    };

    let build_projects = projects.to_vec();
    let build_project_root = project_root.to_path_buf();
    let build_config_root = config_root.to_path_buf();
    let build_scanned_files = scanned_files;
    let active_epoch = service.corpus_active_epoch(SearchCorpusKind::ReferenceOccurrence);
    let service = service.clone();

    if let Ok(handle) = Handle::try_current() {
        handle.spawn(async move {
            let previous_fingerprints = service
                .file_fingerprints(SearchPlaneFileFingerprintScope::corpus(
                    SearchCorpusKind::ReferenceOccurrence,
                ))
                .await;
            let build_service = service.clone();
            let build: Result<_, tokio::task::JoinError> =
                tokio::task::spawn_blocking(move || {
                    plan_reference_occurrence_build_with_scanned_files(
                        &build_service,
                        build_project_root.as_path(),
                        build_config_root.as_path(),
                        &build_projects,
                        build_scanned_files.as_slice(),
                        active_epoch,
                        &previous_fingerprints,
                    )
                })
                .await;

            match build {
                Ok(plan) => {
                    service.coordinator().update_progress(&lease, 0.3);
                    let write = write_reference_occurrence_epoch(&service, &lease, &plan).await;
                    if let Err(error) = write {
                        service.coordinator().fail_build(
                            &lease,
                            format!("reference occurrence epoch write failed: {error}"),
                        );
                        return;
                    }
                    let write = write.unwrap_or_else(|_| unreachable!());
                    service.coordinator().update_progress(&lease, 0.8);
                    if service
                        .publish_ready_and_maintain(&lease, write.row_count, write.fragment_count)
                        .await
                    {
                        service
                            .set_file_fingerprints(
                                SearchPlaneFileFingerprintScope::corpus(
                                    SearchCorpusKind::ReferenceOccurrence,
                                ),
                                &plan.file_fingerprints,
                            )
                            .await;
                    }
                    service.coordinator().update_progress(&lease, 0.9);
                    let prewarm_columns =
                        crate::search::reference_occurrence::schema::projected_columns();
                    if let Err(error) = service
                        .prewarm_epoch_table(lease.corpus, lease.epoch, &prewarm_columns)
                        .await
                    {
                        log::warn!(
                            "reference occurrence epoch prewarm failed after publish for epoch {}: {}",
                            lease.epoch,
                            error
                        );
                    }
                    service.coordinator().update_progress(&lease, 1.0);
                }
                Err(error) => {
                    service.coordinator().fail_build(
                        &lease,
                        format!("reference occurrence background build panicked: {error}"),
                    );
                }
            }
        });
    } else {
        service.coordinator().fail_build(
            &lease,
            "Tokio runtime unavailable for reference occurrence index build",
        );
    }

    true
}

#[cfg(any(test, feature = "test-support"))]
pub async fn publish_reference_occurrences_from_projects(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    fingerprint: &str,
) -> Result<(), ReferenceOccurrenceBuildError> {
    let lease = match service.coordinator().begin_build(
        SearchCorpusKind::ReferenceOccurrence,
        fingerprint,
        SearchCorpusKind::ReferenceOccurrence.schema_version(),
    ) {
        BeginBuildDecision::Started(lease) => lease,
        BeginBuildDecision::AlreadyReady(_) | BeginBuildDecision::AlreadyIndexing(_) => {
            return Ok(());
        }
    };
    let plan = plan_reference_occurrence_build(
        service,
        project_root,
        config_root,
        projects,
        None,
        &std::collections::BTreeMap::new(),
    );
    match write_reference_occurrence_epoch(service, &lease, &plan).await {
        Ok(write) => {
            let prewarm_columns = crate::search::reference_occurrence::schema::projected_columns();
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
                format!("reference occurrence epoch write failed: {error}"),
            );
            Err(ReferenceOccurrenceBuildError::Storage(error))
        }
    }
}
