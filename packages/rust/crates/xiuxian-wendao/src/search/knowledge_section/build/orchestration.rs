use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::gateway::studio::types::UiProjectConfig;
use crate::search::knowledge_section::build::rows::build_knowledge_section_rows_for_entry;
use crate::search::knowledge_section::build::types::KnowledgeSectionBuildPlan;
use crate::search::knowledge_section::build::write::write_knowledge_section_epoch;
use crate::search::knowledge_section::schema::projected_columns;
#[cfg(test)]
use crate::search::scan_note_project_files;
use crate::search::{
    BeginBuildDecision, ProjectScannedFile, SearchCorpusKind, SearchFileFingerprint,
    SearchPlaneService, fingerprint_note_projects_from_scanned_files, stable_payload_fingerprint,
};
use tokio::runtime::Handle;

const KNOWLEDGE_SECTION_EXTRACTOR_VERSION: u32 = 1;

#[cfg(test)]
pub(crate) fn ensure_knowledge_section_index_started(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
) -> bool {
    if projects.is_empty() {
        return false;
    }

    let (fingerprint, scanned_files) = service.fingerprint_note_projects_with_repeat_work_details(
        "knowledge_section.fingerprint",
        project_root,
        config_root,
        projects,
    );
    ensure_knowledge_section_index_started_with_fingerprint_and_scanned_files(
        service,
        project_root,
        config_root,
        projects,
        fingerprint,
        scanned_files,
    )
}

pub(crate) fn ensure_knowledge_section_index_started_with_scanned_files(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
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
    ensure_knowledge_section_index_started_with_fingerprint_and_scanned_files(
        service,
        project_root,
        config_root,
        projects,
        fingerprint,
        scanned_files.to_vec(),
    )
}

fn ensure_knowledge_section_index_started_with_fingerprint_and_scanned_files(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
    fingerprint: String,
    scanned_files: Vec<ProjectScannedFile>,
) -> bool {
    let decision = service.coordinator().begin_build(
        SearchCorpusKind::KnowledgeSection,
        fingerprint,
        SearchCorpusKind::KnowledgeSection.schema_version(),
    );
    let BeginBuildDecision::Started(lease) = decision else {
        return false;
    };

    let build_projects = projects.to_vec();
    let build_project_root = project_root.to_path_buf();
    let build_config_root = config_root.to_path_buf();
    let build_scanned_files = scanned_files;
    let active_epoch = service.corpus_active_epoch(SearchCorpusKind::KnowledgeSection);
    let service = service.clone();

    if let Ok(handle) = Handle::try_current() {
        handle.spawn(async move {
            let previous_fingerprints = service
                .corpus_file_fingerprints(SearchCorpusKind::KnowledgeSection)
                .await;
            let build_service = service.clone();
            let build: Result<KnowledgeSectionBuildPlan, tokio::task::JoinError> =
                tokio::task::spawn_blocking(move || {
                    plan_knowledge_section_build_with_scanned_files(
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
                    let write = write_knowledge_section_epoch(&service, &lease, &plan).await;
                    if let Err(error) = write {
                        service.coordinator().fail_build(
                            &lease,
                            format!("knowledge section epoch write failed: {error}"),
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
                            .set_corpus_file_fingerprints(
                                SearchCorpusKind::KnowledgeSection,
                                &plan.file_fingerprints,
                            )
                            .await;
                    }
                    service.coordinator().update_progress(&lease, 0.9);
                    let prewarm_columns = projected_columns();
                    if let Err(error) = service
                        .prewarm_epoch_table(lease.corpus, lease.epoch, &prewarm_columns)
                        .await
                    {
                        log::warn!(
                            "knowledge section epoch prewarm failed after publish for epoch {}: {}",
                            lease.epoch,
                            error
                        );
                    }
                    service.coordinator().update_progress(&lease, 1.0);
                }
                Err(error) => {
                    service.coordinator().fail_build(
                        &lease,
                        format!("knowledge section background build panicked: {error}"),
                    );
                }
            }
        });
    } else {
        service.coordinator().fail_build(
            &lease,
            "Tokio runtime unavailable for knowledge section build",
        );
    }

    true
}

#[cfg(test)]
pub(super) fn plan_knowledge_section_build(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
    active_epoch: Option<u64>,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> KnowledgeSectionBuildPlan {
    let scanned_files = scan_note_project_files(project_root, config_root, projects);
    service.record_repeat_work_scanned_files(
        "knowledge_section.plan",
        "scan_note_project_files",
        &scanned_files,
    );
    plan_knowledge_section_build_with_scanned_files(
        service,
        project_root,
        config_root,
        projects,
        scanned_files.as_slice(),
        active_epoch,
        previous_fingerprints,
    )
}

fn plan_knowledge_section_build_with_scanned_files(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
    scanned_files: &[ProjectScannedFile],
    active_epoch: Option<u64>,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> KnowledgeSectionBuildPlan {
    let can_incremental_reuse = active_epoch.is_some() && !previous_fingerprints.is_empty();
    let files_requiring_semantic_eval = if can_incremental_reuse {
        scanned_files
            .iter()
            .filter(|file| {
                previous_fingerprints
                    .get(file.normalized_path.as_str())
                    .is_none_or(|previous| {
                        !previous.matches_scan_metadata(
                            Some(file.partition_id.as_str()),
                            file.size_bytes,
                            file.modified_unix_ms(),
                            KNOWLEDGE_SECTION_EXTRACTOR_VERSION,
                            SearchCorpusKind::KnowledgeSection.schema_version(),
                        )
                    })
            })
            .cloned()
            .collect::<Vec<_>>()
    } else {
        scanned_files.to_vec()
    };
    let markdown_snapshot = service
        .shared_markdown_project_snapshot(project_root, files_requiring_semantic_eval.as_slice());

    let mut file_fingerprints = BTreeMap::<String, SearchFileFingerprint>::new();
    let mut changed_files = Vec::<ProjectScannedFile>::new();
    let mut changed_rows = Vec::new();

    for file in scanned_files {
        if can_incremental_reuse
            && let Some(previous) = previous_fingerprints.get(file.normalized_path.as_str())
            && previous.matches_scan_metadata(
                Some(file.partition_id.as_str()),
                file.size_bytes,
                file.modified_unix_ms(),
                KNOWLEDGE_SECTION_EXTRACTOR_VERSION,
                SearchCorpusKind::KnowledgeSection.schema_version(),
            )
        {
            file_fingerprints.insert(file.normalized_path.clone(), previous.clone());
            continue;
        }

        let entry = markdown_snapshot.entry(file.normalized_path.as_str());
        let semantic_fingerprint = entry
            .and_then(|entry| entry.note_fingerprint.clone())
            .unwrap_or_else(|| knowledge_section_rows_fingerprint(&[]));
        let fingerprint = file.to_semantic_file_fingerprint(
            KNOWLEDGE_SECTION_EXTRACTOR_VERSION,
            SearchCorpusKind::KnowledgeSection.schema_version(),
            semantic_fingerprint,
        );
        let changed = !can_incremental_reuse
            || previous_fingerprints
                .get(file.normalized_path.as_str())
                .is_none_or(|previous| !previous.equivalent_for_incremental(&fingerprint));
        file_fingerprints.insert(file.normalized_path.clone(), fingerprint);
        if changed {
            let file_rows = entry.map_or_else(Vec::new, |entry| {
                build_knowledge_section_rows_for_entry(project_root, config_root, projects, entry)
            });
            changed_files.push(file.clone());
            changed_rows.extend(file_rows);
        }
    }

    if !can_incremental_reuse {
        return KnowledgeSectionBuildPlan {
            base_epoch: None,
            file_fingerprints,
            replaced_paths: BTreeSet::new(),
            changed_rows,
        };
    }

    let mut replaced_paths = changed_files
        .iter()
        .map(|file| file.normalized_path.clone())
        .collect::<BTreeSet<_>>();
    for path in previous_fingerprints.keys() {
        if !file_fingerprints.contains_key(path) {
            replaced_paths.insert(path.clone());
        }
    }

    KnowledgeSectionBuildPlan {
        base_epoch: active_epoch,
        file_fingerprints,
        replaced_paths,
        changed_rows,
    }
}

fn knowledge_section_rows_fingerprint(
    rows: &[crate::search::knowledge_section::schema::KnowledgeSectionRow],
) -> String {
    let payload = rows
        .iter()
        .map(|row| {
            (
                row.id.as_str(),
                row.path.as_str(),
                row.stem.as_str(),
                row.title.as_deref(),
                row.best_section.as_deref(),
                row.search_text.as_str(),
                row.hit_json.as_str(),
            )
        })
        .collect::<Vec<_>>();
    stable_payload_fingerprint("knowledge_section_rows", &payload)
}
