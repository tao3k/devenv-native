use std::sync::Arc;

use log::info;
use xiuxian_zhenfa::ZhenfaSignal;

use crate::studio::router::state::cold_start::StudioSearchColdStartTelemetryState;
use crate::studio::router::state::types::{
    GatewayState, LocalCorpusScanCoalescingState, StudioConfiguredOwners, StudioState,
};
use crate::studio::router::{
    load_ui_config_from_wendao_toml, load_ui_config_from_wendao_toml_path,
    resolve_studio_config_root,
};
use crate::studio::symbol_index::SymbolIndexCoordinator;
use xiuxian_wendao::analyzers::PluginRegistry;
use xiuxian_wendao::link_graph::LinkGraphIndex;
use xiuxian_wendao::repo_index::start_repo_index_coordinator;
use xiuxian_wendao::search::SearchPlaneService;
#[cfg(test)]
use xiuxian_wendao::search::{SearchMaintenancePolicy, SearchManifestKeyspace};

const GATEWAY_BOOTSTRAP_BACKGROUND_INDEXING_ENV: &str =
    "XIUXIAN_WENDAO_GATEWAY_BOOTSTRAP_BACKGROUND_INDEXING";
const DEFAULT_STUDIO_BOOTSTRAP_BACKGROUND_INDEXING: bool = false;
const DEFAULT_GATEWAY_START_BOOTSTRAP_BACKGROUND_INDEXING: bool = true;

impl Drop for StudioState {
    fn drop(&mut self) {
        self.stop_background_services();
    }
}

impl GatewayState {
    /// Create gateway state shared by the CLI endpoints and Studio router.
    #[must_use]
    pub fn new(
        index: Option<Arc<LinkGraphIndex>>,
        signal_tx: Option<tokio::sync::mpsc::UnboundedSender<ZhenfaSignal>>,
        plugin_registry: Arc<PluginRegistry>,
    ) -> Self {
        Self::new_with_webhook_url(index, signal_tx, None, plugin_registry)
    }

    /// Create gateway state shared by the CLI endpoints and Studio router
    /// with the effective webhook URL chosen at startup.
    #[must_use]
    pub fn new_with_webhook_url(
        index: Option<Arc<LinkGraphIndex>>,
        signal_tx: Option<tokio::sync::mpsc::UnboundedSender<ZhenfaSignal>>,
        webhook_url: Option<String>,
        plugin_registry: Arc<PluginRegistry>,
    ) -> Self {
        Self::new_with_webhook_url_and_config_path(
            index,
            signal_tx,
            webhook_url,
            None,
            plugin_registry,
        )
    }

    /// Create gateway state shared by the CLI endpoints and Studio router
    /// with the effective webhook URL and config path chosen at startup.
    #[must_use]
    pub fn new_with_webhook_url_and_config_path(
        index: Option<Arc<LinkGraphIndex>>,
        signal_tx: Option<tokio::sync::mpsc::UnboundedSender<ZhenfaSignal>>,
        webhook_url: Option<String>,
        bootstrap_config_path: Option<&std::path::Path>,
        plugin_registry: Arc<PluginRegistry>,
    ) -> Self {
        Self {
            index,
            signal_tx,
            webhook_url,
            studio: Arc::new(StudioState::new_with_bootstrap_ui_config_from_path(
                plugin_registry,
                bootstrap_config_path,
            )),
        }
    }

    /// Create gateway state for the real `gateway start` runtime, with the
    /// launch-path bootstrap policy resolved from the runtime env.
    #[must_use]
    pub fn new_for_gateway_start(
        index: Option<Arc<LinkGraphIndex>>,
        signal_tx: Option<tokio::sync::mpsc::UnboundedSender<ZhenfaSignal>>,
        webhook_url: Option<String>,
        bootstrap_config_path: Option<&std::path::Path>,
        plugin_registry: Arc<PluginRegistry>,
    ) -> Self {
        Self {
            index,
            signal_tx,
            webhook_url,
            studio: Arc::new(
                StudioState::new_for_gateway_start_with_bootstrap_ui_config_from_path(
                    plugin_registry,
                    bootstrap_config_path,
                ),
            ),
        }
    }
}

impl StudioState {
    fn build_runtime_state(
        plugin_registry: Arc<PluginRegistry>,
        project_root: std::path::PathBuf,
        config_root: std::path::PathBuf,
        search_plane: SearchPlaneService,
        bootstrap_background_indexing: bool,
    ) -> Self {
        let bootstrap_snapshot = search_plane.status();
        let repo_index = start_repo_index_coordinator(
            project_root.clone(),
            Arc::clone(&plugin_registry),
            search_plane.clone(),
        );
        let symbol_index_coordinator = Arc::new(SymbolIndexCoordinator::new(
            project_root.clone(),
            config_root.clone(),
            search_plane.clone(),
        ));
        let state = Self {
            project_root,
            config_root,
            bootstrap_background_indexing,
            cold_start_process_started_at: crate::studio::symbol_index::timestamp_now(),
            cold_start_process_started_instant: std::time::Instant::now(),
            cold_start_telemetry: Arc::new(std::sync::RwLock::new(
                StudioSearchColdStartTelemetryState::default(),
            )),
            bootstrap_background_indexing_deferred_activation: Arc::new(std::sync::RwLock::new(
                None,
            )),
            configured_owners: Arc::new(std::sync::RwLock::new(StudioConfiguredOwners::default())),
            graph_index: Arc::new(std::sync::RwLock::new(None)),
            symbol_index: Arc::new(std::sync::RwLock::new(None)),
            symbol_index_coordinator,
            local_corpus_scan_coalescing: Arc::new(std::sync::RwLock::new(
                LocalCorpusScanCoalescingState::default(),
            )),
            search_plane,
            vfs_scan: Arc::new(std::sync::RwLock::new(None)),
            repo_index,
            plugin_registry,
        };
        state.record_local_corpus_ready_observations_from_snapshot(
            &bootstrap_snapshot,
            "search_plane_bootstrap",
        );
        state
    }

    /// Create a new `StudioState` with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::new_with_bootstrap_ui_config(Arc::new(PluginRegistry::new()))
    }

    /// Create a new `StudioState` and bootstrap UI config from `wendao.toml`.
    #[must_use]
    pub fn new_with_bootstrap_ui_config(plugin_registry: Arc<PluginRegistry>) -> Self {
        Self::new_with_bootstrap_ui_config_from_path(plugin_registry, None)
    }

    /// Create a new `StudioState` and bootstrap UI config from one explicit
    /// gateway config path when available.
    #[must_use]
    pub fn new_with_bootstrap_ui_config_from_path(
        plugin_registry: Arc<PluginRegistry>,
        bootstrap_config_path: Option<&std::path::Path>,
    ) -> Self {
        let project_root = xiuxian_io::PrjDirs::project_root();
        let config_root = bootstrap_config_path
            .and_then(std::path::Path::parent)
            .map_or_else(
                || resolve_studio_config_root(project_root.as_path()),
                std::path::Path::to_path_buf,
            );
        let search_plane = SearchPlaneService::new(project_root.clone());
        Self::new_with_bootstrap_ui_config_for_roots_and_search_plane_and_path(
            plugin_registry,
            project_root,
            config_root,
            bootstrap_config_path,
            search_plane,
        )
    }

    /// Create a new `StudioState` for the real gateway runtime and bootstrap
    /// UI config from one explicit gateway config path when available.
    #[must_use]
    pub fn new_for_gateway_start_with_bootstrap_ui_config_from_path(
        plugin_registry: Arc<PluginRegistry>,
        bootstrap_config_path: Option<&std::path::Path>,
    ) -> Self {
        let project_root = xiuxian_io::PrjDirs::project_root();
        let config_root = bootstrap_config_path
            .and_then(std::path::Path::parent)
            .map_or_else(
                || resolve_studio_config_root(project_root.as_path()),
                std::path::Path::to_path_buf,
            );
        let search_plane = SearchPlaneService::new(project_root.clone());
        Self::new_with_bootstrap_ui_config_for_roots_and_search_plane_and_path_and_background_indexing(
            plugin_registry,
            project_root,
            config_root,
            bootstrap_config_path,
            search_plane,
            gateway_start_bootstrap_background_indexing(),
        )
    }

    /// Create a new `StudioState` with explicit project/config roots and search-plane state.
    #[must_use]
    pub(crate) fn new_with_bootstrap_ui_config_for_roots_and_search_plane(
        plugin_registry: Arc<PluginRegistry>,
        project_root: std::path::PathBuf,
        config_root: std::path::PathBuf,
        search_plane: SearchPlaneService,
    ) -> Self {
        Self::new_with_bootstrap_ui_config_for_roots_and_search_plane_and_path(
            plugin_registry,
            project_root,
            config_root,
            None,
            search_plane,
        )
    }

    /// Create a new `StudioState` with explicit project/config roots, an
    /// optional boot config path, and search-plane state.
    #[must_use]
    pub(crate) fn new_with_bootstrap_ui_config_for_roots_and_search_plane_and_path(
        plugin_registry: Arc<PluginRegistry>,
        project_root: std::path::PathBuf,
        config_root: std::path::PathBuf,
        bootstrap_config_path: Option<&std::path::Path>,
        search_plane: SearchPlaneService,
    ) -> Self {
        Self::new_with_bootstrap_ui_config_for_roots_and_search_plane_and_path_and_background_indexing(
            plugin_registry,
            project_root,
            config_root,
            bootstrap_config_path,
            search_plane,
            gateway_bootstrap_background_indexing(),
        )
    }

    /// Create a new `StudioState` with explicit project/config roots, an
    /// optional boot config path, search-plane state, and a resolved bootstrap
    /// indexing policy.
    #[must_use]
    pub(crate) fn new_with_bootstrap_ui_config_for_roots_and_search_plane_and_path_and_background_indexing(
        plugin_registry: Arc<PluginRegistry>,
        project_root: std::path::PathBuf,
        config_root: std::path::PathBuf,
        bootstrap_config_path: Option<&std::path::Path>,
        search_plane: SearchPlaneService,
        eager_background_indexing: bool,
    ) -> Self {
        let state = Self::build_runtime_state(
            plugin_registry,
            project_root,
            config_root,
            search_plane,
            eager_background_indexing,
        );
        let boot_config = bootstrap_config_path
            .and_then(load_ui_config_from_wendao_toml_path)
            .or_else(|| load_ui_config_from_wendao_toml(state.config_root.as_path()));
        if let Some(config) = boot_config {
            info!(
                "Studio bootstrap loaded {} link_graph.projects and {} repo projects; startup background indexing is {}",
                config.projects.len(),
                config.repo_projects.len(),
                if eager_background_indexing {
                    "enabled"
                } else {
                    "deferred"
                }
            );
            state.bootstrap_runtime_ui_config(config, eager_background_indexing);
        }
        state
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn new_with_bootstrap_ui_config_and_search_plane_root(
        plugin_registry: Arc<PluginRegistry>,
        search_plane_root: std::path::PathBuf,
    ) -> Self {
        let project_root = xiuxian_io::PrjDirs::project_root();
        let config_root = resolve_studio_config_root(project_root.as_path());
        let manifest_keyspace = SearchManifestKeyspace::new(format!(
            "xiuxian:test:search_plane:{}",
            blake3::hash(search_plane_root.to_string_lossy().as_bytes(),).to_hex()
        ));
        let search_plane = SearchPlaneService::with_paths(
            project_root.clone(),
            search_plane_root,
            manifest_keyspace,
            SearchMaintenancePolicy::default(),
        );
        Self::build_runtime_state(
            plugin_registry,
            project_root,
            config_root,
            search_plane,
            false,
        )
    }

    pub(crate) fn stop_background_services(&self) {
        self.repo_index.stop();
        self.symbol_index_coordinator.stop();
        self.search_plane.stop_background_maintenance();
    }
}

impl Default for StudioState {
    fn default() -> Self {
        Self::new()
    }
}

fn gateway_bootstrap_background_indexing() -> bool {
    gateway_bootstrap_background_indexing_with_lookup(&|key| std::env::var(key).ok())
}

fn gateway_start_bootstrap_background_indexing() -> bool {
    gateway_start_bootstrap_background_indexing_with_lookup(&|key| std::env::var(key).ok())
}

pub(crate) fn gateway_bootstrap_background_indexing_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> bool {
    bootstrap_background_indexing_with_lookup(DEFAULT_STUDIO_BOOTSTRAP_BACKGROUND_INDEXING, lookup)
}

pub(crate) fn gateway_start_bootstrap_background_indexing_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> bool {
    bootstrap_background_indexing_with_lookup(
        DEFAULT_GATEWAY_START_BOOTSTRAP_BACKGROUND_INDEXING,
        lookup,
    )
}

fn bootstrap_background_indexing_with_lookup(
    default_enabled: bool,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> bool {
    lookup(GATEWAY_BOOTSTRAP_BACKGROUND_INDEXING_ENV)
        .as_deref()
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
        .and_then(parse_bootstrap_background_indexing_override)
        .unwrap_or(default_enabled)
}

fn parse_bootstrap_background_indexing_override(raw: &str) -> Option<bool> {
    if raw == "1"
        || raw.eq_ignore_ascii_case("true")
        || raw.eq_ignore_ascii_case("yes")
        || raw.eq_ignore_ascii_case("on")
    {
        return Some(true);
    }
    if raw == "0"
        || raw.eq_ignore_ascii_case("false")
        || raw.eq_ignore_ascii_case("no")
        || raw.eq_ignore_ascii_case("off")
    {
        return Some(false);
    }
    None
}
