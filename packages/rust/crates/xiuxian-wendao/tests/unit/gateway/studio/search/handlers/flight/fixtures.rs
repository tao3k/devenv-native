use std::fs;
use std::sync::Arc;

use async_trait::async_trait;
use tempfile::{TempDir, tempdir};
use xiuxian_vector::{
    LanceDataType, LanceField, LanceFloat64Array, LanceRecordBatch, LanceSchema, LanceStringArray,
};
use xiuxian_wendao_runtime::transport::{
    RepoSearchFlightRouteProvider, RerankScoreWeights, WendaoFlightService,
};

use super::build_studio_search_flight_service_with_repo_provider;
use crate::gateway::studio::search::handlers::tests::linked_parser_summary::ensure_linked_julia_parser_summary_service;
use crate::gateway::studio::search::handlers::tests::test_studio_state;
use crate::gateway::studio::test_support::init_git_repository;
use crate::gateway::studio::types::{UiConfig, UiProjectConfig, UiRepoProjectConfig};
use crate::gateway::studio::{GatewayState, StudioState};
use crate::gateway::studio::{build_ast_index, search::build_symbol_index};

pub(super) struct GatewayStateFixture {
    _temp_dir: TempDir,
    pub(super) state: Arc<GatewayState>,
}

fn write_fixture_files(root: &std::path::Path, files: &[(&str, &str)], context: &str) {
    for (path, contents) in files {
        let full_path = root.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|error| panic!("create {context} dirs for {path}: {error}"));
        }
        fs::write(&full_path, contents)
            .unwrap_or_else(|error| panic!("write {context} file {path}: {error}"));
    }
}

fn gateway_state_fixture(temp_dir: TempDir, studio: StudioState) -> GatewayStateFixture {
    GatewayStateFixture {
        _temp_dir: temp_dir,
        state: Arc::new(GatewayState {
            index: None,
            signal_tx: None,
            webhook_url: None,
            studio: Arc::new(studio),
        }),
    }
}

pub(super) async fn make_gateway_state_with_docs(docs: &[(&str, &str)]) -> GatewayStateFixture {
    let temp_dir = tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    write_fixture_files(temp_dir.path(), docs, "fixture");

    let mut studio = test_studio_state();
    studio.project_root = temp_dir.path().to_path_buf();
    studio.config_root = temp_dir.path().to_path_buf();
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec!["docs".to_string(), "packages".to_string()],
        }],
        repo_projects: Vec::new(),
    });
    let warmed_index = build_symbol_index(
        studio.project_root.as_path(),
        studio.config_root.as_path(),
        studio.configured_projects().as_slice(),
    );
    studio.symbol_index_coordinator.set_ready_index_for_test(
        studio.configured_projects().as_slice(),
        Arc::clone(&studio.symbol_index),
        warmed_index,
    );
    publish_local_symbol_index(&studio).await;

    gateway_state_fixture(temp_dir, studio)
}

pub(super) async fn make_gateway_state_with_search_routes() -> GatewayStateFixture {
    let temp_dir = tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let docs = [
        (
            "docs/alpha.md",
            "# Alpha\n\nIntent keyword: alpha.\n\n![Topology](assets/topology.png)\n",
        ),
        (
            "packages/rust/crates/demo/src/lib.rs",
            "pub struct AlphaService;\npub fn alpha_handler() {}\n",
        ),
    ];
    write_fixture_files(temp_dir.path(), &docs, "fixture");

    let mut studio = test_studio_state();
    studio.project_root = temp_dir.path().to_path_buf();
    studio.config_root = temp_dir.path().to_path_buf();
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec!["docs".to_string(), "packages".to_string()],
        }],
        repo_projects: Vec::new(),
    });

    let configured_projects = studio.configured_projects();
    publish_local_symbol_index(&studio).await;

    let fingerprint_seed = format!(
        "{}:{}:{}",
        studio.project_root.display(),
        studio.config_root.display(),
        configured_projects.len()
    );
    let knowledge_fingerprint = format!(
        "test:knowledge:{}",
        blake3::hash(fingerprint_seed.as_bytes()).to_hex()
    );
    studio
        .search_plane
        .publish_knowledge_sections_from_projects(
            studio.project_root.as_path(),
            studio.config_root.as_path(),
            &configured_projects,
            knowledge_fingerprint.as_str(),
        )
        .await
        .unwrap_or_else(|error| panic!("publish knowledge sections: {error}"));

    let reference_fingerprint = format!(
        "test:reference:{}",
        blake3::hash(fingerprint_seed.as_bytes()).to_hex()
    );
    studio
        .search_plane
        .publish_reference_occurrences_from_projects(
            studio.project_root.as_path(),
            studio.config_root.as_path(),
            &configured_projects,
            reference_fingerprint.as_str(),
        )
        .await
        .unwrap_or_else(|error| panic!("publish reference occurrences: {error}"));

    let attachment_fingerprint = format!(
        "test:attachment:{}",
        blake3::hash(fingerprint_seed.as_bytes()).to_hex()
    );
    studio
        .search_plane
        .publish_attachments_from_projects(
            studio.project_root.as_path(),
            studio.config_root.as_path(),
            &configured_projects,
            attachment_fingerprint.as_str(),
        )
        .await
        .unwrap_or_else(|error| panic!("publish attachments: {error}"));

    gateway_state_fixture(temp_dir, studio)
}

async fn publish_local_symbol_index(studio: &StudioState) {
    let hits = build_ast_index(
        studio.project_root.as_path(),
        studio.config_root.as_path(),
        studio.configured_projects().as_slice(),
    );
    let fingerprint = format!(
        "test:local-symbol:{}",
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
        .unwrap_or_else(|error| panic!("publish local symbols: {error}"));
}

pub(super) fn make_gateway_state_with_repo(repo_files: &[(&str, &str)]) -> GatewayStateFixture {
    ensure_linked_julia_parser_summary_service()
        .unwrap_or_else(|error| panic!("ensure linked Julia parser-summary service: {error}"));
    let temp_dir = tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    init_git_repository(temp_dir.path().join("repo"));
    for (path, contents) in repo_files {
        let full_path = temp_dir.path().join("repo").join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)
                .unwrap_or_else(|error| panic!("create repo fixture dirs for {path}: {error}"));
        }
        fs::write(&full_path, contents)
            .unwrap_or_else(|error| panic!("write repo fixture {path}: {error}"));
    }

    let mut studio = test_studio_state();
    studio.project_root = temp_dir.path().to_path_buf();
    studio.config_root = temp_dir.path().to_path_buf();
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec!["docs".to_string()],
        }],
        repo_projects: vec![UiRepoProjectConfig {
            id: "demo".to_string(),
            root: Some("repo".to_string()),
            url: None,
            git_ref: None,
            refresh: None,
            plugins: vec!["julia".to_string()],
        }],
    });

    gateway_state_fixture(temp_dir, studio)
}

pub(super) async fn make_gateway_state_with_attachments() -> GatewayStateFixture {
    let temp_dir = tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    fs::create_dir_all(temp_dir.path().join("docs/assets"))
        .unwrap_or_else(|error| panic!("create docs/assets: {error}"));
    fs::write(
        temp_dir.path().join("docs/alpha.md"),
        "# Alpha\n\n![Topology](assets/topology.png)\n",
    )
    .unwrap_or_else(|error| panic!("write alpha.md: {error}"));

    let mut studio = test_studio_state();
    studio.project_root = temp_dir.path().to_path_buf();
    studio.config_root = temp_dir.path().to_path_buf();
    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec!["docs".to_string()],
        }],
        repo_projects: Vec::new(),
    });

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
        .unwrap_or_else(|error| panic!("publish attachments: {error}"));

    gateway_state_fixture(temp_dir, studio)
}

#[derive(Debug)]
struct RecordingRepoSearchProvider;

#[async_trait]
impl RepoSearchFlightRouteProvider for RecordingRepoSearchProvider {
    async fn repo_search_batch(
        &self,
        request: &xiuxian_wendao_runtime::transport::RepoSearchFlightRequest,
    ) -> Result<LanceRecordBatch, String> {
        LanceRecordBatch::try_new(
            Arc::new(LanceSchema::new(vec![
                LanceField::new("doc_id", LanceDataType::Utf8, false),
                LanceField::new("score", LanceDataType::Float64, false),
            ])),
            vec![
                Arc::new(LanceStringArray::from(vec![format!(
                    "repo:{}:{}",
                    request.query_text, request.limit
                )])) as _,
                Arc::new(LanceFloat64Array::from(vec![0.99_f64])) as _,
            ],
        )
        .map_err(|error| error.to_string())
    }
}

pub(super) fn build_service(state: Arc<GatewayState>) -> WendaoFlightService {
    build_studio_search_flight_service_with_repo_provider(
        "v2",
        Arc::new(RecordingRepoSearchProvider),
        state,
        3,
        RerankScoreWeights::default(),
    )
    .unwrap_or_else(|error| panic!("build studio flight service: {error}"))
}

#[allow(dead_code)]
pub(super) fn bare_gateway_state() -> Arc<GatewayState> {
    Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(StudioState::new_with_bootstrap_ui_config(Arc::new(
            crate::analyzers::bootstrap_builtin_registry()
                .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
        ))),
    })
}
