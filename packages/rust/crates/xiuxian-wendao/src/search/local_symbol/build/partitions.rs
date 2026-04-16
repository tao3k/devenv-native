use std::collections::BTreeMap;
use std::path::Path;

use crate::gateway::studio::search::is_markdown_path;
use crate::gateway::studio::types::AstSearchHit;
use crate::search::local_symbol::build::LocalSymbolPartitionBuildPlan;
use crate::search::{MarkdownProjectSnapshot, ProjectScannedFile, SearchPlaneService};

pub(crate) fn build_partition_plans_from_file_hits(
    files: &[ProjectScannedFile],
    file_hits_by_path: &BTreeMap<String, Vec<AstSearchHit>>,
) -> BTreeMap<String, LocalSymbolPartitionBuildPlan> {
    let mut partitions = BTreeMap::<String, LocalSymbolPartitionBuildPlan>::new();
    for file in files {
        let Some(file_hits) = file_hits_by_path.get(file.normalized_path.as_str()) else {
            continue;
        };
        partitions
            .entry(file.partition_id.clone())
            .or_default()
            .changed_hits
            .extend(file_hits.clone());
    }
    partitions
}

pub(crate) fn build_hits_for_file(
    service: &SearchPlaneService,
    project_root: &Path,
    file: &ProjectScannedFile,
    markdown_snapshot: &MarkdownProjectSnapshot,
) -> Vec<AstSearchHit> {
    let mut file_hits = if is_markdown_path(file.absolute_path.as_path()) {
        markdown_snapshot
            .entry(file.normalized_path.as_str())
            .map_or_else(Vec::new, |entry| entry.clone_ast_hits())
    } else {
        service
            .shared_source_snapshot_entry(project_root, file)
            .ast_hits
            .clone()
    };
    for hit in &mut file_hits {
        if file.project_name.is_some() {
            hit.project_name.clone_from(&file.project_name);
            hit.navigation_target
                .project_name
                .clone_from(&file.project_name);
        }
        if file.root_label.is_some() {
            hit.root_label.clone_from(&file.root_label);
            hit.navigation_target
                .root_label
                .clone_from(&file.root_label);
        }
    }
    file_hits
}
