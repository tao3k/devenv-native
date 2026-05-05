use std::fs;
use std::sync::Arc;

use crate::studio::arrow_types::{
    LanceDataType, LanceField, LanceFloat64Array, LanceRecordBatch, LanceSchema, LanceStringArray,
};
use crate::transport::{
    AnalysisFlightRouteResponse, CodeAstAnalysisFlightRouteProvider,
    MarkdownAnalysisFlightRouteProvider, RefineDocFlightRouteProvider,
    RepoDocCoverageFlightRouteProvider, RepoIndexFlightRouteProvider,
    RepoIndexStatusFlightRouteProvider, RepoOverviewFlightRouteProvider,
    RepoProjectedPageIndexTreeFlightRouteProvider, RepoSearchFlightRouteProvider,
    RepoSyncFlightRouteProvider, RerankScoreWeights, WendaoFlightRouteProviders,
    WendaoFlightService,
};
use async_trait::async_trait;
use tempfile::{TempDir, tempdir};
use tonic::Status;

use super::build_studio_search_flight_service_with_repo_provider;
use crate::contracts::{UiConfig, UiProjectConfig};
use crate::studio::search::handlers::tests::test_studio_state;
use crate::studio::{GatewayState, StudioState};
use crate::studio::{build_ast_index, search::build_symbol_index};

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
    let hits = crate::contracts::domain_ast_hits_for_search_plane(hits);
    studio
        .search_plane
        .publish_local_symbol_hits(fingerprint.as_str(), hits.as_slice())
        .await
        .unwrap_or_else(|error| panic!("publish local symbols: {error}"));
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

#[derive(Debug)]
struct RecordingAnalysisRouteProvider;

#[async_trait]
impl RepoSearchFlightRouteProvider for RecordingRepoSearchProvider {
    async fn repo_search_batch(
        &self,
        request: &crate::transport::RepoSearchFlightRequest,
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

fn analysis_route_response(
    route: &str,
    subject: impl Into<String>,
) -> Result<AnalysisFlightRouteResponse, String> {
    let subject = subject.into();
    let batch = LanceRecordBatch::try_new(
        Arc::new(LanceSchema::new(vec![
            LanceField::new("route", LanceDataType::Utf8, false),
            LanceField::new("subject", LanceDataType::Utf8, false),
        ])),
        vec![
            Arc::new(LanceStringArray::from(vec![route.to_string()])) as _,
            Arc::new(LanceStringArray::from(vec![subject.clone()])) as _,
        ],
    )
    .map_err(|error| error.to_string())?;
    let metadata = serde_json::to_vec(&serde_json::json!({
        "route": route,
        "subject": subject,
    }))
    .map_err(|error| error.to_string())?;
    Ok(AnalysisFlightRouteResponse::new(batch).with_app_metadata(metadata))
}

#[async_trait]
impl MarkdownAnalysisFlightRouteProvider for RecordingAnalysisRouteProvider {
    async fn markdown_analysis_batch(
        &self,
        path: &str,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        analysis_route_response("markdown", path)
    }
}

#[async_trait]
impl CodeAstAnalysisFlightRouteProvider for RecordingAnalysisRouteProvider {
    async fn code_ast_analysis_batch(
        &self,
        path: &str,
        repo_id: &str,
        line_hint: Option<usize>,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        analysis_route_response("code_ast", format!("{repo_id}:{path}:{line_hint:?}"))
    }
}

#[async_trait]
impl RepoOverviewFlightRouteProvider for RecordingAnalysisRouteProvider {
    async fn repo_overview_batch(
        &self,
        repo_id: &str,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        analysis_route_response("repo_overview", repo_id)
    }
}

#[async_trait]
impl RepoIndexFlightRouteProvider for RecordingAnalysisRouteProvider {
    async fn repo_index_batch(
        &self,
        repo_id: Option<&str>,
        refresh: bool,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        analysis_route_response("repo_index", format!("{repo_id:?}:{refresh}"))
    }
}

#[async_trait]
impl RepoIndexStatusFlightRouteProvider for RecordingAnalysisRouteProvider {
    async fn repo_index_status_batch(
        &self,
        repo_id: Option<&str>,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        analysis_route_response("repo_index_status", format!("{repo_id:?}"))
    }
}

#[async_trait]
impl RepoSyncFlightRouteProvider for RecordingAnalysisRouteProvider {
    async fn repo_sync_batch(
        &self,
        repo_id: &str,
        mode: &str,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        analysis_route_response("repo_sync", format!("{repo_id}:{mode}"))
    }
}

#[async_trait]
impl RepoDocCoverageFlightRouteProvider for RecordingAnalysisRouteProvider {
    async fn repo_doc_coverage_batch(
        &self,
        repo_id: &str,
        module_id: Option<&str>,
    ) -> Result<AnalysisFlightRouteResponse, String> {
        analysis_route_response("repo_doc_coverage", format!("{repo_id}:{module_id:?}"))
    }
}

#[async_trait]
impl RepoProjectedPageIndexTreeFlightRouteProvider for RecordingAnalysisRouteProvider {
    async fn repo_projected_page_index_tree_batch(
        &self,
        repo_id: &str,
        page_id: &str,
    ) -> Result<AnalysisFlightRouteResponse, Status> {
        analysis_route_response(
            "repo_projected_page_index_tree",
            format!("{repo_id}:{page_id}"),
        )
        .map_err(Status::internal)
    }
}

#[async_trait]
impl RefineDocFlightRouteProvider for RecordingAnalysisRouteProvider {
    async fn refine_doc_batch(
        &self,
        repo_id: &str,
        entity_id: &str,
        user_hints: Option<&str>,
    ) -> Result<AnalysisFlightRouteResponse, Status> {
        analysis_route_response(
            "refine_doc",
            format!("{repo_id}:{entity_id}:{user_hints:?}"),
        )
        .map_err(Status::internal)
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

pub(super) fn build_analysis_route_service() -> WendaoFlightService {
    let analysis_provider = Arc::new(RecordingAnalysisRouteProvider);
    let mut route_providers =
        WendaoFlightRouteProviders::new(Arc::new(RecordingRepoSearchProvider));
    route_providers.markdown_analysis = Some(analysis_provider.clone());
    route_providers.code_ast_analysis = Some(analysis_provider.clone());
    route_providers.repo_overview = Some(analysis_provider.clone());
    route_providers.repo_index = Some(analysis_provider.clone());
    route_providers.repo_index_status = Some(analysis_provider.clone());
    route_providers.repo_sync = Some(analysis_provider.clone());
    route_providers.repo_doc_coverage = Some(analysis_provider.clone());
    route_providers.repo_projected_page_index_tree = Some(analysis_provider.clone());
    route_providers.refine_doc = Some(analysis_provider);
    WendaoFlightService::new_with_route_providers_and_sql(
        "v2",
        route_providers,
        3,
        RerankScoreWeights::default(),
    )
    .unwrap_or_else(|error| panic!("build analysis route flight service: {error}"))
}

#[allow(dead_code)]
pub(super) fn bare_gateway_state() -> Arc<GatewayState> {
    Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(StudioState::new_with_bootstrap_ui_config(Arc::new(
            xiuxian_wendao::analyzers::bootstrap_builtin_registry()
                .unwrap_or_else(|error| panic!("bootstrap registry: {error}")),
        ))),
    })
}
