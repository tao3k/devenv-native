#[cfg(feature = "duckdb")]
use std::fs;

#[cfg(feature = "duckdb")]
use crate::set_link_graph_wendao_config_override;

pub(super) fn repo_search_service() -> SearchPlaneService {
    let temp_dir = temp_dir();
    SearchPlaneService::with_paths(
        PathBuf::from("/tmp/project"),
        temp_dir.path().join("search_plane"),
        service_test_manifest_keyspace(),
        SearchMaintenancePolicy::default(),
    )
}

pub(super) async fn publish_repo_entities(service: &SearchPlaneService) {
    ok_or_panic(
        service
            .publish_repo_entities_with_revision(
                "alpha/repo",
                &sample_repo_analysis(),
                &sample_repo_documents(),
                None,
            )
            .await,
        "publish repo entities",
    );
}

pub(super) async fn publish_repo_content_chunks(service: &SearchPlaneService) {
    ok_or_panic(
        service
            .publish_repo_content_chunks_with_revision("alpha/repo", &sample_repo_documents(), None)
            .await,
        "publish repo content chunks",
    );
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

pub(super) use crate::search::service::tests::support::*;
