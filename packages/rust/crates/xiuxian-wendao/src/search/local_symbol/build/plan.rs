use std::collections::BTreeMap;
use std::path::Path;

use crate::gateway::studio::search::is_markdown_path;
use crate::gateway::studio::types::UiProjectConfig;
use crate::search::local_symbol::build::LocalSymbolBuildPlan;
use crate::search::local_symbol::build::partitions::{
    build_hits_for_file, build_partition_plans_from_file_hits,
};
use crate::search::{
    MarkdownProjectSnapshot, ProjectScannedFile, SearchCorpusKind, SearchFileFingerprint,
    SearchPlaneService, ast_hits_fingerprint,
};
#[cfg(test)]
use crate::search::{fingerprint_symbol_projects, scan_symbol_project_files};

const LOCAL_SYMBOL_EXTRACTOR_VERSION: u32 = 2;

type LocalSymbolFileHits = Vec<crate::gateway::studio::types::AstSearchHit>;

struct LocalSymbolFileEvaluation {
    fingerprint: SearchFileFingerprint,
    changed: bool,
    hits: Option<LocalSymbolFileHits>,
}

#[cfg(test)]
pub(crate) fn plan_local_symbol_build(
    service: &SearchPlaneService,
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
    active_epoch: Option<u64>,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> LocalSymbolBuildPlan {
    let scanned_files = scan_symbol_project_files(project_root, config_root, projects);
    service.record_repeat_work_scanned_files(
        "local_symbol.plan",
        "scan_symbol_project_files",
        &scanned_files,
    );
    plan_local_symbol_build_with_scanned_files(
        service,
        project_root,
        config_root,
        projects,
        scanned_files.as_slice(),
        active_epoch,
        previous_fingerprints,
    )
}

pub(crate) fn plan_local_symbol_build_with_scanned_files(
    service: &SearchPlaneService,
    project_root: &Path,
    _config_root: &Path,
    _projects: &[UiProjectConfig],
    scanned_files: &[ProjectScannedFile],
    active_epoch: Option<u64>,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> LocalSymbolBuildPlan {
    let can_incremental_reuse = active_epoch.is_some() && !previous_fingerprints.is_empty();
    let files_requiring_semantic_eval = collect_files_requiring_semantic_eval(
        scanned_files,
        can_incremental_reuse,
        previous_fingerprints,
    );
    let markdown_snapshot = service
        .shared_markdown_project_snapshot(project_root, files_requiring_semantic_eval.as_slice());
    let (file_fingerprints, changed_files, changed_file_hits) = collect_local_symbol_file_changes(
        service,
        project_root,
        scanned_files,
        &markdown_snapshot,
        can_incremental_reuse,
        previous_fingerprints,
    );

    if !can_incremental_reuse {
        return build_full_local_symbol_plan(scanned_files, file_fingerprints, &changed_file_hits);
    }

    build_incremental_local_symbol_plan(
        active_epoch,
        file_fingerprints,
        changed_files.as_slice(),
        &changed_file_hits,
        previous_fingerprints,
    )
}

fn collect_files_requiring_semantic_eval(
    scanned_files: &[ProjectScannedFile],
    can_incremental_reuse: bool,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> Vec<ProjectScannedFile> {
    if !can_incremental_reuse {
        return scanned_files.to_vec();
    }

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
                        LOCAL_SYMBOL_EXTRACTOR_VERSION,
                        SearchCorpusKind::LocalSymbol.schema_version(),
                    )
                })
        })
        .cloned()
        .collect()
}

fn collect_local_symbol_file_changes(
    service: &SearchPlaneService,
    project_root: &Path,
    scanned_files: &[ProjectScannedFile],
    markdown_snapshot: &MarkdownProjectSnapshot,
    can_incremental_reuse: bool,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> (
    BTreeMap<String, SearchFileFingerprint>,
    Vec<ProjectScannedFile>,
    BTreeMap<String, LocalSymbolFileHits>,
) {
    let mut file_fingerprints = BTreeMap::<String, SearchFileFingerprint>::new();
    let mut changed_files = Vec::<ProjectScannedFile>::new();
    let mut changed_file_hits = BTreeMap::<String, LocalSymbolFileHits>::new();

    for file in scanned_files {
        let evaluation = evaluate_local_symbol_file(
            service,
            project_root,
            file,
            markdown_snapshot,
            can_incremental_reuse,
            previous_fingerprints,
        );
        file_fingerprints.insert(file.normalized_path.clone(), evaluation.fingerprint);
        if evaluation.changed {
            changed_files.push(file.clone());
            if let Some(hits) = evaluation.hits {
                changed_file_hits.insert(file.normalized_path.clone(), hits);
            }
        }
    }

    (file_fingerprints, changed_files, changed_file_hits)
}

fn evaluate_local_symbol_file(
    service: &SearchPlaneService,
    project_root: &Path,
    file: &ProjectScannedFile,
    markdown_snapshot: &MarkdownProjectSnapshot,
    can_incremental_reuse: bool,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> LocalSymbolFileEvaluation {
    if can_incremental_reuse
        && let Some(previous) = previous_fingerprints.get(file.normalized_path.as_str())
        && previous.matches_scan_metadata(
            Some(file.partition_id.as_str()),
            file.size_bytes,
            file.modified_unix_ms(),
            LOCAL_SYMBOL_EXTRACTOR_VERSION,
            SearchCorpusKind::LocalSymbol.schema_version(),
        )
    {
        return LocalSymbolFileEvaluation {
            fingerprint: previous.clone(),
            changed: false,
            hits: None,
        };
    }

    if is_markdown_path(file.absolute_path.as_path())
        && let Some(entry) = markdown_snapshot.entry(file.normalized_path.as_str())
        && let Some(symbol_fingerprint) = entry.symbol_fingerprint.as_ref()
    {
        let fingerprint = file.to_semantic_file_fingerprint(
            LOCAL_SYMBOL_EXTRACTOR_VERSION,
            SearchCorpusKind::LocalSymbol.schema_version(),
            symbol_fingerprint.clone(),
        );
        return LocalSymbolFileEvaluation {
            changed: local_symbol_file_changed(
                can_incremental_reuse,
                previous_fingerprints,
                file.normalized_path.as_str(),
                &fingerprint,
            ),
            fingerprint,
            hits: Some(entry.clone_ast_hits()),
        };
    }

    let file_hits = build_hits_for_file(service, project_root, file, markdown_snapshot);
    let fingerprint = file.to_semantic_file_fingerprint(
        LOCAL_SYMBOL_EXTRACTOR_VERSION,
        SearchCorpusKind::LocalSymbol.schema_version(),
        ast_hits_fingerprint(&file_hits),
    );
    LocalSymbolFileEvaluation {
        changed: local_symbol_file_changed(
            can_incremental_reuse,
            previous_fingerprints,
            file.normalized_path.as_str(),
            &fingerprint,
        ),
        fingerprint,
        hits: Some(file_hits),
    }
}

fn local_symbol_file_changed(
    can_incremental_reuse: bool,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
    path: &str,
    fingerprint: &SearchFileFingerprint,
) -> bool {
    !can_incremental_reuse
        || previous_fingerprints
            .get(path)
            .is_none_or(|previous| !previous.equivalent_for_incremental(fingerprint))
}

fn build_full_local_symbol_plan(
    scanned_files: &[ProjectScannedFile],
    file_fingerprints: BTreeMap<String, SearchFileFingerprint>,
    changed_file_hits: &BTreeMap<String, LocalSymbolFileHits>,
) -> LocalSymbolBuildPlan {
    LocalSymbolBuildPlan {
        base_epoch: None,
        file_fingerprints,
        partitions: build_partition_plans_from_file_hits(scanned_files, changed_file_hits),
    }
}

fn build_incremental_local_symbol_plan(
    active_epoch: Option<u64>,
    file_fingerprints: BTreeMap<String, SearchFileFingerprint>,
    changed_files: &[ProjectScannedFile],
    changed_file_hits: &BTreeMap<String, LocalSymbolFileHits>,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) -> LocalSymbolBuildPlan {
    let mut partitions = build_partition_plans_from_file_hits(changed_files, changed_file_hits);
    mark_changed_local_symbol_paths(&mut partitions, changed_files);
    mark_deleted_or_repartitioned_local_symbol_paths(
        &mut partitions,
        &file_fingerprints,
        previous_fingerprints,
    );

    LocalSymbolBuildPlan {
        base_epoch: active_epoch,
        file_fingerprints,
        partitions,
    }
}

fn mark_changed_local_symbol_paths(
    partitions: &mut BTreeMap<
        String,
        crate::search::local_symbol::build::LocalSymbolPartitionBuildPlan,
    >,
    changed_files: &[ProjectScannedFile],
) {
    for file in changed_files {
        partitions
            .entry(file.partition_id.clone())
            .or_default()
            .replaced_paths
            .insert(file.normalized_path.clone());
    }
}

fn mark_deleted_or_repartitioned_local_symbol_paths(
    partitions: &mut BTreeMap<
        String,
        crate::search::local_symbol::build::LocalSymbolPartitionBuildPlan,
    >,
    file_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
    previous_fingerprints: &BTreeMap<String, SearchFileFingerprint>,
) {
    for (path, previous_fingerprint) in previous_fingerprints {
        let current_fingerprint = file_fingerprints.get(path.as_str());
        if current_fingerprint.is_none() {
            if let Some(partition_id) = previous_fingerprint.partition_id.as_deref() {
                partitions
                    .entry(partition_id.to_string())
                    .or_default()
                    .replaced_paths
                    .insert(path.clone());
            }
            continue;
        }

        if let Some(current_fingerprint) = current_fingerprint
            && current_fingerprint.partition_id != previous_fingerprint.partition_id
            && let Some(partition_id) = previous_fingerprint.partition_id.as_deref()
        {
            partitions
                .entry(partition_id.to_string())
                .or_default()
                .replaced_paths
                .insert(path.clone());
        }
    }
}

#[cfg(test)]
pub(crate) fn fingerprint_projects(
    project_root: &Path,
    config_root: &Path,
    projects: &[UiProjectConfig],
) -> String {
    fingerprint_symbol_projects(project_root, config_root, projects)
}
