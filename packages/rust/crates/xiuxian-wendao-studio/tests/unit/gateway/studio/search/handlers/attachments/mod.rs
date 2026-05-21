use std::sync::Arc;

use crate::studio::search::handlers::queries::AttachmentSearchQuery;
use crate::studio::types::AttachmentSearchResponse;
use crate::transport::AttachmentSearchFlightRouteProvider;
use serde_json::Value;
use xiuxian_wendao::search::{SearchCorpusKind, SearchPlaneCacheTtl};

use super::provider::StudioAttachmentSearchFlightRouteProvider;
use super::response::load_attachment_search_response_from_studio;

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
    studio.seed_eager_configured_owners_for_tests(crate::contracts::UiConfig {
        projects: vec![crate::contracts::UiProjectConfig {
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

#[tokio::test]
async fn studio_attachment_search_response_cache_is_scoped_to_active_epoch() {
    let project_root = match tempfile::tempdir() {
        Ok(project_root) => project_root,
        Err(error) => panic!("attachment cache tempdir should build: {error}"),
    };
    if let Err(error) = std::fs::create_dir_all(project_root.path().join("docs/assets")) {
        panic!("attachment cache docs asset dir should build: {error}");
    }
    std::fs::write(
        project_root.path().join("docs/alpha.md"),
        "# Alpha\n\n![Topology](assets/topology.png)\n",
    )
    .unwrap_or_else(|error| panic!("attachment cache source doc should write: {error}"));

    let mut studio = crate::studio::search::handlers::tests::test_studio_state_with_cache();
    studio.project_root = project_root.path().to_path_buf();
    studio.config_root = project_root.path().to_path_buf();
    studio.seed_eager_configured_owners_for_tests(crate::contracts::UiConfig {
        projects: vec![crate::contracts::UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec!["docs".to_string()],
        }],
        repo_projects: Vec::new(),
    });
    studio
        .search_plane
        .publish_attachments_from_projects(
            studio.project_root.as_path(),
            studio.config_root.as_path(),
            &studio.configured_projects(),
            "test:attachment:first",
        )
        .await
        .unwrap_or_else(|error| panic!("first attachment epoch should publish: {error}"));

    let cache_key = studio
        .search_plane
        .search_query_cache_key(
            "attachment",
            &[SearchCorpusKind::Attachment],
            "topology",
            5,
            Some("ext:png|kind:image|case:false"),
            None,
        )
        .unwrap_or_else(|| panic!("attachment cache key should resolve for active epoch"));
    studio
        .search_plane
        .cache_set_json(
            cache_key.as_str(),
            SearchPlaneCacheTtl::HotQuery,
            &AttachmentSearchResponse {
                query: "cached-topology".to_string(),
                hits: Vec::new(),
                hit_count: 0,
                selected_scope: "attachments-cache".to_string(),
                partial: false,
                indexing_state: Some("ready".to_string().into()),
                index_error: None,
            },
        )
        .await;

    let cached = load_attachment_search_response_from_studio(
        &studio,
        AttachmentSearchQuery {
            q: Some("topology".to_string()),
            limit: Some(5),
            ext: vec!["png".to_string()],
            kind: vec!["image".to_string()],
            case_sensitive: false,
        },
    )
    .await
    .unwrap_or_else(|error| panic!("cached attachment search should load: {error:?}"));
    assert_eq!(cached.query, "cached-topology");
    assert_eq!(cached.selected_scope, "attachments-cache");

    studio
        .search_plane
        .publish_attachments_from_projects(
            studio.project_root.as_path(),
            studio.config_root.as_path(),
            &studio.configured_projects(),
            "test:attachment:second",
        )
        .await
        .unwrap_or_else(|error| panic!("second attachment epoch should publish: {error}"));
    let refreshed = load_attachment_search_response_from_studio(
        &studio,
        AttachmentSearchQuery {
            q: Some("topology".to_string()),
            limit: Some(5),
            ext: vec!["png".to_string()],
            kind: vec!["image".to_string()],
            case_sensitive: false,
        },
    )
    .await
    .unwrap_or_else(|error| panic!("refreshed attachment search should load: {error:?}"));
    assert_eq!(refreshed.query, "topology");
    assert_eq!(refreshed.selected_scope, "attachments");
    assert_eq!(refreshed.hit_count, 1);
}
