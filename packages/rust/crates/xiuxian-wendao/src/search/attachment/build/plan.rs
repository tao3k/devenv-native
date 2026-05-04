use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::search::attachment::build::AttachmentBuildPlan;
use crate::search::attachment::build::extract::build_attachment_hits_for_entry;
use crate::search::contracts::SearchProjectConfig;
#[cfg(any(test, feature = "test-support"))]
use crate::search::scan_note_project_files;
use crate::search::{
    ProjectScannedFile, SearchCorpusKind, SearchFileFingerprint, SearchPlaneService,
    attachment_hits_fingerprint,
};

const ATTACHMENT_EXTRACTOR_VERSION: u32 = 1;

#[cfg(any(test, feature = "test-support"))]
pub(crate) fn plan_attachment_build(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    active_epoch: Option<u64>,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> AttachmentBuildPlan {
    let scanned_files = scan_note_project_files(project_root, config_root, projects);
    service.record_repeat_work_scanned_files(
        "attachment.plan",
        "scan_note_project_files",
        &scanned_files,
    );
    plan_attachment_build_with_scanned_files(
        service,
        project_root,
        config_root,
        projects,
        scanned_files.as_slice(),
        active_epoch,
        previous_fingerprints,
    )
}

pub(crate) fn plan_attachment_build_with_scanned_files(
    service: &SearchPlaneService,
    project_root: &Path,
    _config_root: &Path,
    _projects: &[SearchProjectConfig],
    scanned_files: &[ProjectScannedFile],
    active_epoch: Option<u64>,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> AttachmentBuildPlan {
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
                            ATTACHMENT_EXTRACTOR_VERSION,
                            SearchCorpusKind::Attachment.schema_version(),
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
    let mut changed_hits = Vec::new();

    for file in scanned_files {
        if can_incremental_reuse
            && let Some(previous) = previous_fingerprints.get(file.normalized_path.as_str())
            && previous.matches_scan_metadata(
                Some(file.partition_id.as_str()),
                file.size_bytes,
                file.modified_unix_ms(),
                ATTACHMENT_EXTRACTOR_VERSION,
                SearchCorpusKind::Attachment.schema_version(),
            )
        {
            file_fingerprints.insert(file.normalized_path.clone(), previous.clone());
            continue;
        }

        let entry = markdown_snapshot.entry(file.normalized_path.as_str());
        let semantic_fingerprint = entry
            .and_then(|entry| entry.note_fingerprint.clone())
            .unwrap_or_else(|| attachment_hits_fingerprint(&[]));
        let fingerprint = file.to_semantic_file_fingerprint(
            ATTACHMENT_EXTRACTOR_VERSION,
            SearchCorpusKind::Attachment.schema_version(),
            semantic_fingerprint,
        );
        let changed = !can_incremental_reuse
            || previous_fingerprints
                .get(file.normalized_path.as_str())
                .is_none_or(|previous| !previous.equivalent_for_incremental(&fingerprint));
        file_fingerprints.insert(file.normalized_path.clone(), fingerprint);
        if changed {
            let file_hits =
                entry.map_or_else(Vec::new, |entry| build_attachment_hits_for_entry(entry));
            changed_files.push(file.clone());
            changed_hits.extend(file_hits);
        }
    }

    if !can_incremental_reuse {
        return AttachmentBuildPlan {
            base_epoch: None,
            file_fingerprints,
            replaced_paths: BTreeSet::new(),
            changed_hits,
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

    AttachmentBuildPlan {
        base_epoch: active_epoch,
        file_fingerprints,
        replaced_paths,
        changed_hits,
    }
}
