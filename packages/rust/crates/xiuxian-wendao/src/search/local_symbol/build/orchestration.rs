use std::path::Path;

use tokio::runtime::Handle;

use crate::search::cache::SearchPlaneFileFingerprintScope;
use crate::search::contracts::SearchProjectConfig;
#[cfg(any(test, feature = "test-support"))]
use crate::search::local_symbol::build::LocalSymbolBuildError;
use crate::search::local_symbol::build::{
    plan_local_symbol_build_with_scanned_files, write_local_symbol_epoch,
};
use crate::search::{
    BeginBuildDecision, ProjectScannedFile, SearchCorpusKind, SearchPlaneService,
    fingerprint_symbol_projects_from_scanned_files,
};

#[cfg(any(test, feature = "test-support"))]
pub fn ensure_local_symbol_index_started(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
) -> bool {
    if projects.is_empty() {
        return false;
    }

    let (fingerprint, scanned_files) = service
        .fingerprint_symbol_projects_with_repeat_work_details(
            "local_symbol.fingerprint",
            project_root,
            config_root,
            projects,
        );
    ensure_local_symbol_index_started_with_fingerprint_and_scanned_files(
        service,
        project_root,
        config_root,
        projects,
        fingerprint,
        scanned_files,
    )
}

pub fn ensure_local_symbol_index_started_with_scanned_files(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    scanned_files: &[ProjectScannedFile],
) -> bool {
    if projects.is_empty() {
        return false;
    }

    let fingerprint = fingerprint_symbol_projects_from_scanned_files(
        project_root,
        config_root,
        projects,
        scanned_files,
    );
    ensure_local_symbol_index_started_with_fingerprint_and_scanned_files(
        service,
        project_root,
        config_root,
        projects,
        fingerprint,
        scanned_files.to_vec(),
    )
}

fn ensure_local_symbol_index_started_with_fingerprint_and_scanned_files(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    fingerprint: String,
    scanned_files: Vec<ProjectScannedFile>,
) -> bool {
    let decision = service.coordinator().begin_build(
        SearchCorpusKind::LocalSymbol,
        fingerprint,
        SearchCorpusKind::LocalSymbol.schema_version(),
    );
    let BeginBuildDecision::Started(lease) = decision else {
        return false;
    };

    let build_projects = projects.to_vec();
    let build_project_root = project_root.to_path_buf();
    let build_config_root = config_root.to_path_buf();
    let build_scanned_files = scanned_files;
    let active_epoch = service
        .corpus_active_epoch(SearchCorpusKind::LocalSymbol)
        .filter(|epoch| {
            service.local_epoch_has_partition_tables(SearchCorpusKind::LocalSymbol, *epoch)
        });
    let service = service.clone();

    if let Ok(handle) = Handle::try_current() {
        handle.spawn(async move {
            let previous_fingerprints = service
                .file_fingerprints(SearchPlaneFileFingerprintScope::corpus(
                    SearchCorpusKind::LocalSymbol,
                ))
                .await;
            let build_service = service.clone();
            let build: Result<_, tokio::task::JoinError> = tokio::task::spawn_blocking(move || {
                plan_local_symbol_build_with_scanned_files(
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
                    let write = write_local_symbol_epoch(&service, &lease, &plan).await;
                    if let Err(error) = write {
                        service.coordinator().fail_build(
                            &lease,
                            format!("local symbol epoch write failed: {error}"),
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
                                    SearchCorpusKind::LocalSymbol,
                                ),
                                &plan.file_fingerprints,
                            )
                            .await;
                    }
                    service.coordinator().update_progress(&lease, 0.9);
                    let prewarm_columns = crate::search::local_symbol::schema::projected_columns();
                    if let Err(error) = service
                        .prewarm_epoch_table(lease.corpus, lease.epoch, &prewarm_columns)
                        .await
                    {
                        log::warn!(
                            "local symbol epoch prewarm failed after publish for epoch {}: {}",
                            lease.epoch,
                            error
                        );
                    }
                    service.coordinator().update_progress(&lease, 1.0);
                }
                Err(error) => {
                    service.coordinator().fail_build(
                        &lease,
                        format!("local symbol background build panicked: {error}"),
                    );
                }
            }
        });
    } else {
        service.coordinator().fail_build(
            &lease,
            "Tokio runtime unavailable for local symbol index build",
        );
    }

    true
}

#[cfg(any(test, feature = "test-support"))]
pub async fn publish_local_symbol_hits(
    service: &SearchPlaneService,
    fingerprint: &str,
    hits: &[crate::search::contracts::AstSearchHit],
) -> Result<(), LocalSymbolBuildError> {
    let lease = match service.coordinator().begin_build(
        SearchCorpusKind::LocalSymbol,
        fingerprint,
        SearchCorpusKind::LocalSymbol.schema_version(),
    ) {
        BeginBuildDecision::Started(lease) => lease,
        BeginBuildDecision::AlreadyReady(_) | BeginBuildDecision::AlreadyIndexing(_) => {
            return Err(LocalSymbolBuildError::BuildRejected(
                fingerprint.to_string(),
            ));
        }
    };

    let plan = crate::search::local_symbol::build::LocalSymbolBuildPlan {
        base_epoch: None,
        file_fingerprints: std::collections::BTreeMap::new(),
        partitions: std::collections::BTreeMap::from([(
            "manual".to_string(),
            crate::search::local_symbol::build::LocalSymbolPartitionBuildPlan {
                replaced_paths: std::collections::BTreeSet::new(),
                changed_hits: hits.to_vec(),
            },
        )]),
    };

    match write_local_symbol_epoch(service, &lease, &plan).await {
        Ok(write) => {
            let prewarm_columns = crate::search::local_symbol::schema::projected_columns();
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
                .fail_build(&lease, format!("local symbol epoch write failed: {error}"));
            Err(LocalSymbolBuildError::Storage(error))
        }
    }
}
