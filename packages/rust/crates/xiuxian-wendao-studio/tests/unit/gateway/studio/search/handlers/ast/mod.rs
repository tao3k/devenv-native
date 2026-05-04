use std::fs;
use std::sync::Arc;

use crate::transport::AstSearchFlightRouteProvider;
use serde_json::Value;
use tempfile::tempdir;

use super::provider::StudioAstSearchFlightRouteProvider;
use crate::contracts::{UiConfig, UiProjectConfig};
use crate::studio::GatewayState;
use crate::studio::build_ast_index;
use crate::studio::search::handlers::tests::test_studio_state;

#[tokio::test]
async fn studio_ast_flight_provider_materializes_ast_batches() {
    let temp_dir = match tempdir() {
        Ok(temp_dir) => temp_dir,
        Err(error) => panic!("AST provider tempdir should build: {error}"),
    };
    let source_dir = temp_dir.path().join("packages/rust/crates/demo/src");
    if let Err(error) = fs::create_dir_all(&source_dir) {
        panic!("AST provider source dir should build: {error}");
    }
    fs::write(
        source_dir.join("lib.rs"),
        "pub struct AlphaService;\npub fn alpha_handler() {}\n",
    )
    .unwrap_or_else(|error| panic!("AST provider source fixture should write: {error}"));

    let mut studio = test_studio_state();
    studio.project_root = temp_dir.path().to_path_buf();
    studio.config_root = temp_dir.path().to_path_buf();
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec!["packages".to_string()],
        }],
        repo_projects: Vec::new(),
    });
    let projects = studio.configured_projects();
    let hits = build_ast_index(
        studio.project_root.as_path(),
        studio.config_root.as_path(),
        &projects,
    );
    let fingerprint = format!(
        "test:{}",
        blake3::hash(
            format!(
                "{}:{}:{}",
                studio.project_root.display(),
                studio.config_root.display(),
                hits.len()
            )
            .as_bytes()
        )
        .to_hex()
    );
    studio
        .search_plane
        .publish_local_symbol_hits(fingerprint.as_str(), &hits)
        .await
        .unwrap_or_else(|error| panic!("publish local symbol epoch: {error}"));

    let provider = StudioAstSearchFlightRouteProvider::new(Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(studio),
    }));

    let response = provider
        .ast_search_batch("alpha", 5)
        .await
        .unwrap_or_else(|error| {
            panic!("dedicated AST provider should accept AST requests: {error}")
        });
    let metadata: Value = serde_json::from_slice(&response.app_metadata)
        .unwrap_or_else(|error| panic!("AST provider app_metadata should decode: {error}"));
    let batch = response.batch;

    assert!(batch.num_rows() >= 1);
    assert_eq!(metadata["query"], "alpha");
    assert_eq!(metadata["selectedScope"], "definitions");
}
