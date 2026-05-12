use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::search::contracts::SearchProjectConfig;
#[cfg(test)]
use crate::search::fingerprint_source_projects;
use crate::search::reference_occurrence::build::ReferenceOccurrenceBuildPlan;
use crate::search::reference_occurrence::build::extract::build_reference_occurrences_for_file;
#[cfg(any(test, feature = "test-support"))]
use crate::search::scan_source_project_files;
use crate::search::{
    ProjectScannedFile, SearchCorpusKind, SearchFileFingerprint, SearchPlaneService,
    reference_hits_fingerprint,
};

const REFERENCE_OCCURRENCE_EXTRACTOR_VERSION: u32 = 1;

#[cfg(any(test, feature = "test-support"))]
pub(crate) fn plan_reference_occurrence_build(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    active_epoch: Option<u64>,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> ReferenceOccurrenceBuildPlan {
    let scanned_files = scan_source_project_files(project_root, config_root, projects);
    service.record_repeat_work_scanned_files(
        "reference_occurrence.plan",
        "scan_source_project_files",
        &scanned_files,
    );
    plan_reference_occurrence_build_with_scanned_files(
        service,
        project_root,
        config_root,
        projects,
        scanned_files.as_slice(),
        active_epoch,
        previous_fingerprints,
    )
}

pub(crate) fn plan_reference_occurrence_build_with_scanned_files(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
    scanned_files: &[ProjectScannedFile],
    active_epoch: Option<u64>,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> ReferenceOccurrenceBuildPlan {
    let can_incremental_reuse = active_epoch.is_some() && !previous_fingerprints.is_empty();
    if !can_incremental_reuse {
        let (file_fingerprints, changed_hits) = scanned_files.iter().fold(
            (BTreeMap::<String, SearchFileFingerprint>::new(), Vec::new()),
            |(mut file_fingerprints, mut changed_hits), file| {
                let file_hits = build_reference_occurrences_for_file(
                    service,
                    project_root,
                    config_root,
                    projects,
                    file,
                );
                file_fingerprints.insert(
                    file.normalized_path.clone(),
                    file.to_semantic_file_fingerprint(
                        REFERENCE_OCCURRENCE_EXTRACTOR_VERSION,
                        SearchCorpusKind::ReferenceOccurrence.schema_version(),
                        reference_hits_fingerprint(&file_hits),
                    ),
                );
                changed_hits.extend(file_hits);
                (file_fingerprints, changed_hits)
            },
        );
        return ReferenceOccurrenceBuildPlan {
            base_epoch: None,
            file_fingerprints,
            replaced_paths: BTreeSet::new(),
            changed_hits,
        };
    }

    let (file_fingerprints, changed_files, changed_hits) = scanned_files.iter().fold(
        (
            BTreeMap::<String, SearchFileFingerprint>::new(),
            Vec::<ProjectScannedFile>::new(),
            Vec::new(),
        ),
        |(mut file_fingerprints, mut changed_files, mut changed_hits), file| {
            if let Some(previous) = previous_fingerprints.get(file.normalized_path.as_str())
                && previous.matches_scan_metadata(
                    Some(file.partition_id.as_str()),
                    file.size_bytes,
                    file.modified_unix_ms(),
                    REFERENCE_OCCURRENCE_EXTRACTOR_VERSION,
                    SearchCorpusKind::ReferenceOccurrence.schema_version(),
                )
            {
                file_fingerprints.insert(file.normalized_path.clone(), previous.clone());
                return (file_fingerprints, changed_files, changed_hits);
            }

            let file_hits = build_reference_occurrences_for_file(
                service,
                project_root,
                config_root,
                projects,
                file,
            );
            let fingerprint = file.to_semantic_file_fingerprint(
                REFERENCE_OCCURRENCE_EXTRACTOR_VERSION,
                SearchCorpusKind::ReferenceOccurrence.schema_version(),
                reference_hits_fingerprint(&file_hits),
            );
            let changed = previous_fingerprints
                .get(file.normalized_path.as_str())
                .is_none_or(|previous| !previous.equivalent_for_incremental(&fingerprint));
            file_fingerprints.insert(file.normalized_path.clone(), fingerprint);
            if changed {
                changed_files.push(file.clone());
                changed_hits.extend(file_hits);
            }
            (file_fingerprints, changed_files, changed_hits)
        },
    );
    let mut replaced_paths = changed_files
        .iter()
        .map(|file| file.normalized_path.clone())
        .collect::<BTreeSet<_>>();
    replaced_paths.extend(
        previous_fingerprints
            .keys()
            .filter(|path| !file_fingerprints.contains_key(*path))
            .cloned(),
    );

    ReferenceOccurrenceBuildPlan {
        base_epoch: active_epoch,
        file_fingerprints,
        replaced_paths,
        changed_hits,
    }
}

#[cfg(test)]
pub(crate) fn fingerprint_projects(
    project_root: &Path,
    config_root: &Path,
    projects: &[SearchProjectConfig],
) -> String {
    fingerprint_source_projects(project_root, config_root, projects)
}
