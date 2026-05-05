use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Instant, SystemTime};

use serde::Serialize;

use crate::studio::router::state::cold_start::StudioSearchColdStartTelemetryState;
use crate::studio::symbol_index::{SymbolIndexCoordinator, timestamp_now};
use crate::studio::types::{UiConfig, UiProjectConfig, UiRepoProjectConfig};
use xiuxian_wendao::analyzers::PluginRegistry;
use xiuxian_wendao::link_graph::LinkGraphIndex;
use xiuxian_wendao::repo_index::RepoIndexCoordinator;
use xiuxian_wendao::search::SearchPlaneService;
use xiuxian_wendao::unified_symbol::UnifiedSymbolIndex;

use crate::studio::types::VfsScanResult;
use xiuxian_zhenfa::ZhenfaSignal;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct GraphSourceSignature {
    pub(crate) note_count: usize,
    pub(crate) latest_modified_at: Option<SystemTime>,
    pub(crate) total_size_bytes: u64,
}

#[derive(Clone)]
pub(crate) struct GraphIndexCacheEntry {
    pub(crate) index: Arc<LinkGraphIndex>,
    pub(crate) source_signature: GraphSourceSignature,
}

#[derive(Clone)]
pub(crate) struct DeferredBootstrapBackgroundIndexingActivation {
    pub(crate) activated_at: String,
    pub(crate) source: String,
}

/// Shared bootstrap-indexing telemetry derived from the Studio runtime state.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioBootstrapBackgroundIndexingTelemetry {
    #[serde(rename = "studioBootstrapBackgroundIndexingEnabled")]
    enabled: bool,
    #[serde(rename = "studioBootstrapBackgroundIndexingMode")]
    mode: &'static str,
    #[serde(rename = "studioBootstrapBackgroundIndexingDeferredActivationObserved")]
    deferred_activation_observed: bool,
    #[serde(rename = "studioBootstrapBackgroundIndexingDeferredActivationAt")]
    deferred_activation_at: Option<String>,
    #[serde(rename = "studioBootstrapBackgroundIndexingDeferredActivationSource")]
    deferred_activation_source: Option<String>,
}

impl StudioBootstrapBackgroundIndexingTelemetry {
    /// Returns whether bootstrap-time background indexing is enabled.
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns the stable bootstrap-time background-indexing mode label.
    #[must_use]
    pub fn mode(&self) -> &'static str {
        self.mode
    }

    /// Returns whether deferred bootstrap indexing has been lazily activated since boot.
    #[must_use]
    pub fn deferred_activation_observed(&self) -> bool {
        self.deferred_activation_observed
    }

    /// Returns the first deferred bootstrap-indexing activation timestamp, if any.
    #[must_use]
    pub fn deferred_activation_at(&self) -> Option<&str> {
        self.deferred_activation_at.as_deref()
    }

    /// Returns the source that first activated deferred bootstrap indexing, if any.
    #[must_use]
    pub fn deferred_activation_source(&self) -> Option<&str> {
        self.deferred_activation_source.as_deref()
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct StudioConfiguredOwners {
    pub(crate) projects: Vec<UiProjectConfig>,
    pub(crate) repo_projects: Vec<UiRepoProjectConfig>,
}

impl StudioConfiguredOwners {
    pub(crate) fn ui_config(&self) -> UiConfig {
        UiConfig {
            projects: self.projects.clone(),
            repo_projects: self.repo_projects.clone(),
        }
    }
}

/// Shared state for the Studio API.
///
/// Contains configuration, VFS roots, and cached graph index.
pub struct StudioState {
    pub(crate) project_root: PathBuf,
    pub(crate) config_root: PathBuf,
    pub(crate) bootstrap_background_indexing: bool,
    pub(crate) cold_start_process_started_at: String,
    pub(crate) cold_start_process_started_instant: Instant,
    pub(crate) cold_start_telemetry: Arc<RwLock<StudioSearchColdStartTelemetryState>>,
    pub(crate) bootstrap_background_indexing_deferred_activation:
        Arc<RwLock<Option<DeferredBootstrapBackgroundIndexingActivation>>>,
    pub(crate) configured_owners: Arc<RwLock<StudioConfiguredOwners>>,
    pub(crate) graph_index: Arc<RwLock<Option<GraphIndexCacheEntry>>>,
    pub(crate) symbol_index: Arc<RwLock<Option<Arc<UnifiedSymbolIndex>>>>,
    pub(crate) symbol_index_coordinator: Arc<SymbolIndexCoordinator>,
    pub(crate) search_plane: SearchPlaneService,
    pub(crate) vfs_scan: Arc<RwLock<Option<VfsScanResult>>>,
    pub(crate) repo_index: Arc<RepoIndexCoordinator>,
    /// Registry of repository intelligence plugins.
    pub(crate) plugin_registry: Arc<PluginRegistry>,
}

impl StudioState {
    /// Returns one clone of the shared search-plane service owned by the Studio runtime.
    #[must_use]
    pub fn search_plane_service(&self) -> SearchPlaneService {
        self.search_plane.clone()
    }

    /// Returns whether bootstrap-time background indexing is enabled for this state instance.
    #[must_use]
    pub fn bootstrap_background_indexing_enabled(&self) -> bool {
        self.bootstrap_background_indexing
    }

    /// Returns the stable mode label for bootstrap-time background indexing.
    #[must_use]
    pub fn bootstrap_background_indexing_mode(&self) -> &'static str {
        if self.bootstrap_background_indexing_enabled() {
            "enabled"
        } else {
            "deferred"
        }
    }

    /// Returns the current bootstrap-indexing telemetry snapshot.
    #[must_use]
    pub fn bootstrap_background_indexing_telemetry(
        &self,
    ) -> StudioBootstrapBackgroundIndexingTelemetry {
        let deferred_activation_at = self.bootstrap_background_indexing_deferred_activation_at();
        let deferred_activation_source =
            self.bootstrap_background_indexing_deferred_activation_source();
        StudioBootstrapBackgroundIndexingTelemetry {
            enabled: self.bootstrap_background_indexing_enabled(),
            mode: self.bootstrap_background_indexing_mode(),
            deferred_activation_observed: deferred_activation_at.is_some(),
            deferred_activation_at,
            deferred_activation_source,
        }
    }

    /// Returns the first deferred bootstrap-indexing activation timestamp, if any.
    #[must_use]
    pub fn bootstrap_background_indexing_deferred_activation_at(&self) -> Option<String> {
        self.bootstrap_background_indexing_deferred_activation
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .map(|activation| activation.activated_at.clone())
    }

    /// Returns the source that first activated deferred bootstrap indexing, if any.
    #[must_use]
    pub fn bootstrap_background_indexing_deferred_activation_source(&self) -> Option<String> {
        self.bootstrap_background_indexing_deferred_activation
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .map(|activation| activation.source.clone())
    }

    pub(crate) fn record_deferred_bootstrap_background_indexing_activation(
        &self,
        source: &'static str,
    ) {
        if self.bootstrap_background_indexing_enabled() {
            return;
        }

        let mut guard = self
            .bootstrap_background_indexing_deferred_activation
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if guard.is_some() {
            return;
        }

        *guard = Some(DeferredBootstrapBackgroundIndexingActivation {
            activated_at: timestamp_now(),
            source: source.to_string(),
        });
    }
}

/// Shared state used by the top-level gateway process.
#[derive(Clone)]
pub struct GatewayState {
    /// Optional graph index for CLI-powered stats endpoint.
    pub index: Option<Arc<LinkGraphIndex>>,
    /// Signal sender for notification worker.
    pub signal_tx: Option<tokio::sync::mpsc::UnboundedSender<ZhenfaSignal>>,
    /// Effective webhook URL chosen at gateway startup, if configured.
    pub webhook_url: Option<String>,
    /// Studio-specific state for VFS/graph/search APIs.
    pub studio: Arc<StudioState>,
}
