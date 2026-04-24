use std::path::PathBuf;

#[cfg(feature = "duckdb")]
use std::fs;

use crate::gateway::studio::types::{AttachmentSearchHit, StudioNavigationTarget};
use crate::search::attachment::schema::attachment_batches;
use crate::search::{
    BeginBuildDecision, SearchCorpusKind, SearchMaintenancePolicy, SearchManifestKeyspace,
    SearchPlaneService,
};
#[cfg(feature = "duckdb")]
use crate::set_link_graph_wendao_config_override;
use xiuxian_db_store::write_lance_batches_to_parquet_file;

pub(super) fn fixture_service(temp_dir: &tempfile::TempDir) -> SearchPlaneService {
    SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        SearchManifestKeyspace::new("xiuxian:test:attachment"),
        SearchMaintenancePolicy::default(),
    )
}

#[cfg(feature = "duckdb")]
pub(super) fn write_search_duckdb_runtime_override(
    body: &str,
) -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let config_path = temp.path().join("wendao.toml");
    fs::write(&config_path, body)?;
    set_link_graph_wendao_config_override(&config_path.to_string_lossy());
    Ok(temp)
}

pub(super) fn sample_hit(
    name: &str,
    source_path: &str,
    attachment_path: &str,
    kind: &str,
) -> AttachmentSearchHit {
    AttachmentSearchHit {
        name: name.to_string(),
        path: source_path.to_string(),
        source_id: source_path.trim_end_matches(".md").to_string(),
        source_stem: "alpha".to_string(),
        source_title: "Alpha".to_string(),
        source_path: source_path.to_string(),
        attachment_id: format!("att://{source_path}/{attachment_path}"),
        attachment_path: attachment_path.to_string(),
        attachment_name: name.to_string(),
        attachment_ext: attachment_path
            .split('.')
            .next_back()
            .unwrap_or_default()
            .to_string(),
        kind: kind.to_string(),
        navigation_target: StudioNavigationTarget {
            path: source_path.to_string(),
            category: "doc".to_string(),
            project_name: None,
            root_label: None,
            line: None,
            line_end: None,
            column: None,
        },
        score: 0.0,
        vision_snippet: None,
    }
}

pub(super) async fn publish_attachment_hits(
    service: &SearchPlaneService,
    build_id: &str,
    hits: &[AttachmentSearchHit],
) {
    let lease = match service.coordinator().begin_build(
        SearchCorpusKind::Attachment,
        build_id,
        SearchCorpusKind::Attachment.schema_version(),
    ) {
        BeginBuildDecision::Started(lease) => lease,
        other => panic!("unexpected begin decision: {other:?}"),
    };
    let batches = attachment_batches(hits).unwrap_or_else(|error| panic!("batches: {error}"));
    write_lance_batches_to_parquet_file(
        service
            .local_epoch_parquet_path(SearchCorpusKind::Attachment, lease.epoch)
            .as_path(),
        &batches,
    )
    .unwrap_or_else(|error| panic!("write attachment parquet: {error}"));
    service.coordinator().publish_ready(
        &lease,
        u64::try_from(hits.len()).unwrap_or(u64::MAX),
        u64::try_from(batches.len()).unwrap_or(u64::MAX),
    );
}
