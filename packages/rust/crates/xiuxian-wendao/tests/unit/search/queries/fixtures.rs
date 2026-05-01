use std::fs;

use tempfile::TempDir;

use crate::gateway::studio::types::{ReferenceSearchHit, StudioNavigationTarget};
use crate::search::{
    BeginBuildDecision, SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace,
    SearchPlaneService, reference_occurrence_batches,
};
use xiuxian_db_store::write_lance_batches_to_parquet_file;

pub(crate) fn fixture_service(temp_dir: &TempDir, keyspace: &str) -> SearchPlaneService {
    let project_root = temp_dir.path().join("project");
    fs::create_dir_all(&project_root).unwrap_or_else(|error| {
        panic!(
            "create query fixture project root `{}`: {error}",
            project_root.display()
        )
    });
    SearchPlaneService::with_paths(
        project_root,
        temp_dir.path().join("search_plane"),
        SearchManifestKeyspace::new(keyspace),
        SearchMaintenancePolicy::default(),
    )
}

pub(crate) fn sample_hit(name: &str, path: &str, line: usize) -> ReferenceSearchHit {
    ReferenceSearchHit {
        name: name.to_string(),
        path: path.to_string(),
        language: "rust".to_string(),
        crate_name: "kernel".to_string(),
        project_name: None,
        root_label: None,
        line,
        column: 5,
        line_text: format!("let _value = {name};"),
        navigation_target: StudioNavigationTarget {
            path: path.to_string(),
            category: "doc".to_string(),
            project_name: None,
            root_label: None,
            line: Some(line),
            line_end: Some(line),
            column: Some(5),
        },
        score: 0.0,
    }
}

pub(crate) async fn publish_reference_hits(
    service: &SearchPlaneService,
    build_id: &str,
    hits: &[ReferenceSearchHit],
) {
    let lease = match service.coordinator().begin_build(
        SearchCorpusKind::ReferenceOccurrence,
        build_id,
        SearchCorpusKind::ReferenceOccurrence.schema_version(),
    ) {
        BeginBuildDecision::Started(lease) => lease,
        other => panic!("unexpected begin decision: {other:?}"),
    };
    let batches = reference_occurrence_batches(hits)
        .unwrap_or_else(|error| panic!("reference occurrence batches: {error}"));
    write_lance_batches_to_parquet_file(
        service
            .local_epoch_parquet_path(SearchCorpusKind::ReferenceOccurrence, lease.epoch)
            .as_path(),
        batches.as_slice(),
    )
    .unwrap_or_else(|error| panic!("export reference occurrence parquet: {error}"));
    service
        .coordinator()
        .publish_ready(&lease, hits.len() as u64, 1);
}
