//! `search::attachment::build::orchestration` owns Wendao attachment build orchestration behavior.

use std::path::Path;

use tokio::runtime::Handle;

#[cfg(any(test, feature = "test-support"))]
use crate::search::attachment::build::AttachmentBuildError;
#[cfg(any(test, feature = "test-support"))]
use crate::search::attachment::build::plan_attachment_build;
use crate::search::attachment::build::{
    plan_attachment_build_with_scanned_files, write_attachment_epoch,
};
use crate::search::attachment::schema::projected_columns_with_hit_json;
use crate::search::cache::SearchPlaneFileFingerprintScope;
use crate::search::contracts::SearchProjectConfig;
use crate::search::{
    BeginBuildDecision, ProjectScannedFile, SearchCorpusKind, SearchPlaneService,
    fingerprint_note_projects_from_scanned_files,
};
/// `ensure_attachment_index_started` public function boundary for Wendao.

#[cfg(any(test, feature = "test-support"))]
pub fn ensure_attachment_index_started(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
) -> bool {
    if projects.is_empty() {
        return false;
    }

    let (fingerprint, scanned_files) = service.fingerprint_note_projects_with_repeat_work_details(
        "attachment.fingerprint",
        project_root,
        config_root,
        projects,
    );
    ensure_attachment_index_started_with_fingerprint_and_scanned_files(
        service,
        project_root,
        config_root,
        projects,
        fingerprint,
        scanned_files,
    )
}
/// `ensure_attachment_index_started_with_scanned_files` public function boundary for Wendao.

/// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
pub fn ensure_attachment_index_started_with_scanned_files(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    scanned_files: &[ProjectScannedFile],
) -> bool {
    if projects.is_empty() {
        return false;
    }

    let fingerprint = fingerprint_note_projects_from_scanned_files(
        project_root,
        config_root,
        projects,
        scanned_files,
    );
    ensure_attachment_index_started_with_fingerprint_and_scanned_files(
        service,
        project_root,
        config_root,
        projects,
        fingerprint,
        scanned_files.to_vec(),
    )
}

fn ensure_attachment_index_started_with_fingerprint_and_scanned_files(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    fingerprint: String,
    scanned_files: Vec<ProjectScannedFile>,
) -> bool {
    let decision = service.coordinator().begin_build(
        SearchCorpusKind::Attachment,
        fingerprint,
        SearchCorpusKind::Attachment.schema_version(),
    );
    let BeginBuildDecision::Started(lease) = decision else {
        return false;
    };

    let build_projects = projects.to_vec();
    let build_project_root = project_root.to_path_buf();
    let build_config_root = config_root.to_path_buf();
    let build_scanned_files = scanned_files;
    let active_epoch = service.corpus_active_epoch(SearchCorpusKind::Attachment);
    let service = service.clone();

    if let Ok(handle) = Handle::try_current() {
        handle.spawn(async move {
            let previous_fingerprints = service
                .file_fingerprints(SearchPlaneFileFingerprintScope::corpus(
                    SearchCorpusKind::Attachment,
                ))
                .await;
            let build_service = service.clone();
            let build: Result<_, tokio::task::JoinError> = tokio::task::spawn_blocking(move || {
                plan_attachment_build_with_scanned_files(
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
                    let write = write_attachment_epoch(&service, &lease, &plan).await;
                    if let Err(error) = write {
                        service
                            .coordinator()
                            .fail_build(&lease, format!("attachment epoch write failed: {error}"));
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
                                    SearchCorpusKind::Attachment,
                                ),
                                &plan.file_fingerprints,
                            )
                            .await;
                    }
                    service.coordinator().update_progress(&lease, 0.9);
                    let prewarm_columns = projected_columns_with_hit_json();
                    if let Err(error) = service
                        .prewarm_epoch_table(lease.corpus, lease.epoch, &prewarm_columns)
                        .await
                    {
                        log::warn!(
                            "attachment epoch prewarm failed after publish for epoch {}: {}",
                            lease.epoch,
                            error
                        );
                    }
                    service.coordinator().update_progress(&lease, 1.0);
                }
                Err(error) => {
                    service.coordinator().fail_build(
                        &lease,
                        format!("attachment background build panicked: {error}"),
                    );
                }
            }
        });
    } else {
        service.coordinator().fail_build(
            &lease,
            "Tokio runtime unavailable for attachment index build",
        );
    }

    true
}
/// `publish_attachments_from_projects` public function boundary for Wendao.

#[cfg(any(test, feature = "test-support"))]
pub async fn publish_attachments_from_projects(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    fingerprint: &str,
) -> Result<(), AttachmentBuildError> {
    let lease = match service.coordinator().begin_build(
        SearchCorpusKind::Attachment,
        fingerprint,
        SearchCorpusKind::Attachment.schema_version(),
    ) {
        BeginBuildDecision::Started(lease) => lease,
        BeginBuildDecision::AlreadyReady(_) | BeginBuildDecision::AlreadyIndexing(_) => {
            return Ok(());
        }
    };
    let plan = plan_attachment_build(
        service,
        project_root,
        config_root,
        projects,
        None,
        &std::collections::BTreeMap::new(),
    );
    match write_attachment_epoch(service, &lease, &plan).await {
        Ok(write) => {
            let prewarm_columns = projected_columns_with_hit_json();
            service
                .prewarm_epoch_table(lease.corpus, lease.epoch, &prewarm_columns)
                .await?;
            service
                .publish_ready_and_maintain(&lease, write.row_count, write.fragment_count)
                .await;
            Ok(())
        }
        Err(error) => {
            service
                .coordinator()
                .fail_build(&lease, format!("attachment epoch write failed: {error}"));
            Err(AttachmentBuildError::Storage(error))
        }
    }
}
