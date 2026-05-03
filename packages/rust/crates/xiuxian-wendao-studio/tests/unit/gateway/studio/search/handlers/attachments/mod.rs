use std::sync::Arc;

use crate::transport::AttachmentSearchFlightRouteProvider;
use serde_json::Value;

use super::provider::StudioAttachmentSearchFlightRouteProvider;

#[tokio::test]
async fn studio_attachment_search_flight_provider_uses_attachment_contract() {
    let project_root = match tempfile::tempdir() {
        Ok(project_root) => project_root,
        Err(error) => panic!("attachment provider tempdir should build: {error}"),
    };
    if let Err(error) = std::fs::create_dir_all(project_root.path().join("docs/assets")) {
        panic!("attachment provider docs asset dir should build: {error}");
    }
    std::fs::write(
        project_root.path().join("docs/alpha.md"),
        "# Alpha\n\n![Topology](assets/topology.png)\n",
    )
    .unwrap_or_else(|error| panic!("attachment provider source doc should write: {error}"));

    let mut studio = crate::studio::search::handlers::tests::test_studio_state();
    studio.project_root = project_root.path().to_path_buf();
    studio.config_root = project_root.path().to_path_buf();
    studio.seed_eager_configured_owners_for_tests(xiuxian_wendao::search::contracts::UiConfig {
        projects: vec![xiuxian_wendao::search::contracts::UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec!["docs".to_string()],
        }],
        repo_projects: Vec::new(),
    });
    let studio = Arc::new(studio);
    let fingerprint = format!(
        "test:attachment:{}",
        blake3::hash(
            format!(
                "{}:{}:{}",
                studio.project_root.display(),
                studio.config_root.display(),
                studio.configured_projects().len()
            )
            .as_bytes()
        )
        .to_hex()
    );
    studio
        .search_plane
        .publish_attachments_from_projects(
            studio.project_root.as_path(),
            studio.config_root.as_path(),
            &studio.configured_projects(),
            fingerprint.as_str(),
        )
        .await
        .unwrap_or_else(|error| panic!("attachment provider index should publish: {error}"));

    let provider = StudioAttachmentSearchFlightRouteProvider::new(studio);

    let response = provider
        .attachment_search_batch(
            "topology",
            5,
            &["png".to_string()].into_iter().collect(),
            &["image".to_string()].into_iter().collect(),
            false,
        )
        .await
        .unwrap_or_else(|error| panic!("attachment provider should build a batch: {error}"));
    let metadata: Value = serde_json::from_slice(&response.app_metadata)
        .unwrap_or_else(|error| panic!("attachment provider app_metadata should decode: {error}"));
    let batch = response.batch;

    assert_eq!(batch.num_rows(), 1);
    assert!(batch.column_by_name("attachmentPath").is_some());
    assert!(batch.column_by_name("navigationTargetJson").is_some());
    assert_eq!(metadata["query"], "topology");
    assert_eq!(metadata["selectedScope"], "attachments");
}
