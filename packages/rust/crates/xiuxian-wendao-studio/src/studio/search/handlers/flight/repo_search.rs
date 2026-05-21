//! Owns the Studio handlers flight repo search surface.

use std::sync::Arc;

use crate::studio::arrow_types::LanceRecordBatch;
use async_trait::async_trait;
use xiuxian_wendao_server::transport::{
    RepoSearchFlightRequest, RepoSearchFlightRouteProvider, RerankScoreWeights, WendaoFlightService,
};

use super::service::build_studio_search_flight_service_with_repo_provider;
use crate::studio::types::{UiConfig, UiRepoProjectConfig};
use crate::studio::{GatewayState, StudioState};
use xiuxian_db_store::lance_batch_to_engine_batch;
use xiuxian_wendao::repo_index::RepoCodeDocument;
use xiuxian_wendao::search::SearchPlaneService;
use xiuxian_wendao::search::repo_search::{
    search_repo_content_batch, search_repo_content_batch_with_repository,
};

/// Runtime-backed repo-search Flight provider that reads from the Wendao search plane.
#[derive(Clone)]
pub struct StudioRepoSearchFlightRouteProvider {
    search_plane: Arc<SearchPlaneService>,
    studio: Option<Arc<StudioState>>,
}

impl std::fmt::Debug for StudioRepoSearchFlightRouteProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StudioRepoSearchFlightRouteProvider")
            .finish_non_exhaustive()
    }
}

impl StudioRepoSearchFlightRouteProvider {
    /// Create one Studio repo-search Flight provider backed by the Wendao
    /// search plane.
    #[must_use]
    pub fn new(search_plane: Arc<SearchPlaneService>) -> Self {
        Self {
            search_plane,
            studio: None,
        }
    }

    /// Create one Studio repo-search Flight provider with access to the
    /// Studio repository configuration for search-only checkout fallback.
    #[must_use]
    pub fn with_studio(search_plane: Arc<SearchPlaneService>, studio: Arc<StudioState>) -> Self {
        Self {
            search_plane,
            studio: Some(studio),
        }
    }
}

#[async_trait]
impl RepoSearchFlightRouteProvider for StudioRepoSearchFlightRouteProvider {
    async fn repo_search_batch(
        &self,
        request: &RepoSearchFlightRequest,
    ) -> Result<LanceRecordBatch, String> {
        if request.repo_id.trim().is_empty() {
            return Err("repo-search Flight request repo_id must not be blank".to_string());
        }
        let result = if let Some(studio) = self.studio.as_ref() {
            let runtime_request = runtime_repo_search_request(request);
            let repository =
                crate::studio::configured_repository(studio.as_ref(), request.repo_id.trim()).ok();
            search_repo_content_batch_with_repository(
                self.search_plane.as_ref(),
                repository.as_ref(),
                &runtime_request,
            )
            .await
        } else {
            let runtime_request = runtime_repo_search_request(request);
            search_repo_content_batch(self.search_plane.as_ref(), &runtime_request).await
        };
        let batch =
            result.map_err(|error| format!("repo-search Flight provider failed: {error}"))?;
        Ok(lance_batch_to_engine_batch(&batch))
    }
}

/// Project and config roots used to bootstrap a Studio Flight service.
#[derive(Debug, Clone)]
pub struct StudioFlightRoots {
    project_root: std::path::PathBuf,
    config_root: std::path::PathBuf,
    direct_repo_id: Option<String>,
}

impl StudioFlightRoots {
    /// Create a root bundle for Studio Flight service bootstrap.
    #[must_use]
    pub fn new(project_root: std::path::PathBuf, config_root: std::path::PathBuf) -> Self {
        Self {
            project_root,
            config_root,
            direct_repo_id: None,
        }
    }

    /// Register the project root as a direct repository when the runtime config
    /// does not already declare the requested repo id.
    #[must_use]
    pub fn with_direct_repo_id(mut self, repo_id: impl Into<String>) -> Self {
        self.direct_repo_id = Some(repo_id.into());
        self
    }
}

fn runtime_repo_search_request(
    request: &RepoSearchFlightRequest,
) -> xiuxian_wendao_runtime::transport::RepoSearchFlightRequest {
    xiuxian_wendao_runtime::transport::RepoSearchFlightRequest {
        repo_id: request.repo_id.clone(),
        query_text: request.query_text.clone(),
        limit: request.limit,
        language_filters: request.language_filters.clone(),
        path_prefixes: request.path_prefixes.clone(),
        title_filters: request.title_filters.clone(),
        tag_filters: request.tag_filters.clone(),
        filename_filters: request.filename_filters.clone(),
    }
}

/// Build one runtime-owned Flight service from the Wendao search plane.
///
/// # Errors
///
/// Returns an error when the runtime Flight service cannot be constructed for
/// the requested schema version and rerank dimension.
pub fn build_repo_search_flight_service(
    search_plane: Arc<SearchPlaneService>,
    expected_schema_version: impl Into<String>,
    rerank_dimension: usize,
) -> Result<WendaoFlightService, String> {
    build_repo_search_flight_service_with_weights(
        search_plane,
        expected_schema_version,
        rerank_dimension,
        RerankScoreWeights::default(),
    )
}

/// Build one runtime-owned Flight service from the Wendao search plane with
/// explicit rerank score weights.
///
/// # Errors
///
/// Returns an error when the runtime Flight service cannot be constructed for
/// the requested schema version, rerank dimension, and rerank score weights.
pub fn build_repo_search_flight_service_with_weights(
    search_plane: Arc<SearchPlaneService>,
    expected_schema_version: impl Into<String>,
    rerank_dimension: usize,
    rerank_weights: RerankScoreWeights,
) -> Result<WendaoFlightService, String> {
    let provider = Arc::new(StudioRepoSearchFlightRouteProvider::new(search_plane));
    WendaoFlightService::new_with_provider(
        expected_schema_version,
        provider,
        rerank_dimension,
        rerank_weights,
    )
}

/// Build one runtime-owned Flight service from the Wendao search plane and the
/// current Studio-owned semantic search providers.
///
/// # Errors
///
/// Returns an error when the runtime Flight service cannot be constructed for
/// the requested schema version and rerank dimension.
pub fn build_studio_flight_service(
    search_plane: Arc<SearchPlaneService>,
    gateway_state: Arc<GatewayState>,
    expected_schema_version: impl Into<String>,
    rerank_dimension: usize,
) -> Result<WendaoFlightService, String> {
    build_studio_flight_service_with_weights(
        search_plane,
        gateway_state,
        expected_schema_version,
        rerank_dimension,
        RerankScoreWeights::default(),
    )
}

/// Build one runtime-owned Flight service from the Wendao search plane and the
/// current Studio-owned semantic search providers with explicit rerank weights.
///
/// # Errors
///
/// Returns an error when the runtime Flight service cannot be constructed for
/// the requested schema version, rerank dimension, and rerank score weights.
pub(crate) fn build_studio_flight_service_with_weights(
    search_plane: Arc<SearchPlaneService>,
    gateway_state: Arc<GatewayState>,
    expected_schema_version: impl Into<String>,
    rerank_dimension: usize,
    rerank_weights: RerankScoreWeights,
) -> Result<WendaoFlightService, String> {
    let provider = Arc::new(StudioRepoSearchFlightRouteProvider::with_studio(
        search_plane,
        Arc::clone(&gateway_state.studio),
    ));
    build_studio_search_flight_service_with_repo_provider(
        expected_schema_version,
        provider,
        gateway_state,
        rerank_dimension,
        rerank_weights,
    )
}

/// Build one runtime-owned Flight service from the Wendao search plane and one
/// Studio bootstrap state resolved from explicit project/config roots.
///
/// # Errors
///
/// Returns an error when the plugin registry cannot be bootstrapped or when the
/// runtime Flight service cannot be constructed for the requested schema
/// version and rerank dimension.
pub fn build_studio_flight_service_for_roots(
    search_plane: Arc<SearchPlaneService>,
    roots: StudioFlightRoots,
    expected_schema_version: impl Into<String>,
    rerank_dimension: usize,
) -> Result<WendaoFlightService, String> {
    build_studio_flight_service_for_roots_with_weights(
        search_plane,
        roots,
        expected_schema_version,
        rerank_dimension,
        RerankScoreWeights::default(),
    )
}

/// Build one runtime-owned Flight service from the Wendao search plane and one
/// Studio bootstrap state resolved from explicit project/config roots with
/// explicit rerank score weights.
///
/// # Errors
///
/// Returns an error when the plugin registry cannot be bootstrapped or when the
/// runtime Flight service cannot be constructed for the requested schema
/// version, rerank dimension, and rerank score weights.
pub(crate) fn build_studio_flight_service_for_roots_with_weights(
    search_plane: Arc<SearchPlaneService>,
    roots: StudioFlightRoots,
    expected_schema_version: impl Into<String>,
    rerank_dimension: usize,
    rerank_weights: RerankScoreWeights,
) -> Result<WendaoFlightService, String> {
    let plugin_registry = Arc::new(
        xiuxian_wendao::analyzers::bootstrap_builtin_registry()
            .map_err(|error| format!("bootstrap registry: {error}"))?,
    );
    let StudioFlightRoots {
        project_root,
        config_root,
        direct_repo_id,
    } = roots;
    let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane(
        plugin_registry,
        project_root.clone(),
        config_root,
        search_plane.as_ref().clone(),
    );
    if let Some(repo_id) = direct_repo_id
        && !studio
            .configured_repo_projects()
            .iter()
            .any(|repository| repository.id == repo_id)
    {
        studio.bootstrap_runtime_ui_config(
            UiConfig {
                projects: Vec::new(),
                repo_projects: vec![UiRepoProjectConfig {
                    id: repo_id,
                    root: Some(project_root.display().to_string()),
                    url: None,
                    git_ref: None,
                    refresh: Some("manual".to_string()),
                    plugins: vec!["markdown-parser".to_string()],
                }],
            },
            false,
        );
    }
    let gateway_state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(studio),
    });
    build_studio_flight_service_with_weights(
        search_plane,
        gateway_state,
        expected_schema_version,
        rerank_dimension,
        rerank_weights,
    )
}

/// Seed one minimal repo-content sample into the search plane for Flight smoke
/// tests and local bring-up.
///
/// # Errors
///
/// Returns an error when the repo identifier is blank or when repo-content
/// publication fails.
pub async fn bootstrap_sample_repo_search_content(
    search_plane: &SearchPlaneService,
    repo_id: impl AsRef<str>,
) -> Result<(), String> {
    let repo_id = repo_id.as_ref().trim();
    if repo_id.is_empty() {
        return Err("sample repo-search bootstrap repo_id must not be blank".to_string());
    }

    let documents = vec![
        RepoCodeDocument {
            path: "src/lib.rs".to_string(),
            language: Some("rust".to_string()),
            contents: Arc::<str>::from("pub fn alpha_beta() {}\n"),
            size_bytes: 23,
            modified_unix_ms: 10,
        },
        RepoCodeDocument {
            path: "src/flight.rs".to_string(),
            language: Some("rust".to_string()),
            contents: Arc::<str>::from("pub fn flight_router() -> &'static str { \"flight\" }\n"),
            size_bytes: 52,
            modified_unix_ms: 10,
        },
        RepoCodeDocument {
            path: "src/search.rs".to_string(),
            language: Some("rust".to_string()),
            contents: Arc::<str>::from(
                "pub fn repo_search_kernel() -> &'static str { \"searchonlytoken semantic search kernel\" }\n",
            ),
            size_bytes: 88,
            modified_unix_ms: 10,
        },
        RepoCodeDocument {
            path: "src/flight_search.rs".to_string(),
            language: Some("rust".to_string()),
            contents: Arc::<str>::from(
                "pub fn flight_search_bridge() -> &'static str { \"flightbridgetoken flight search bridge\" }\n",
            ),
            size_bytes: 92,
            modified_unix_ms: 10,
        },
        RepoCodeDocument {
            path: "src/camelbridge.rs".to_string(),
            language: Some("rust".to_string()),
            contents: Arc::<str>::from(
                "pub fn camel_bridge_lower() -> &'static str { \"camelbridgetoken\" }\n",
            ),
            size_bytes: 70,
            modified_unix_ms: 10,
        },
        RepoCodeDocument {
            path: "src/a_rank.rs".to_string(),
            language: Some("rust".to_string()),
            contents: Arc::<str>::from(
                "pub fn alpha_rank() -> &'static str { \"ranktieexacttoken\" }\n",
            ),
            size_bytes: 62,
            modified_unix_ms: 10,
        },
        RepoCodeDocument {
            path: "src/z_rank.rs".to_string(),
            language: Some("rust".to_string()),
            contents: Arc::<str>::from(
                "pub fn zeta_rank() -> &'static str { \"ranktieexacttoken\" }\n",
            ),
            size_bytes: 61,
            modified_unix_ms: 10,
        },
        RepoCodeDocument {
            path: "README.md".to_string(),
            language: Some("markdown".to_string()),
            contents: Arc::<str>::from(
                "# alpha repo\nThis repo mentions alpha beta flight search.\n",
            ),
            size_bytes: 56,
            modified_unix_ms: 10,
        },
        RepoCodeDocument {
            path: "docs/CamelBridge.md".to_string(),
            language: Some("markdown".to_string()),
            contents: Arc::<str>::from(
                "# CamelBridgeToken\nExact-case bridge token for flight ranking.\n",
            ),
            size_bytes: 64,
            modified_unix_ms: 10,
        },
    ];

    search_plane
        .publish_repo_content_chunks_with_revision(repo_id, &documents, Some("flight-smoke-rev"))
        .await
        .map_err(|error| {
            format!("failed to bootstrap sample repo-search content for `{repo_id}`: {error}")
        })
}

#[path = "../../../../../tests/unit/gateway/studio/search/handlers/flight/repo_search/mod.rs"]
mod tests;
