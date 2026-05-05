use std::path::Path;
use std::sync::{Arc, RwLock};

use anyhow::Result;

use crate::studio::symbol_index::SymbolIndexCoordinator;
use crate::studio::types::{UiConfig, UiRepoProjectConfig};
use crate::studio::{
    GatewayState, StudioState, load_ui_config_from_wendao_toml, studio_effective_wendao_toml_path,
};
use xiuxian_wendao::analyzers::{
    RegisteredRepository, RepositoryPluginConfig, RepositoryRef, RepositoryRefreshPolicy,
    bootstrap_builtin_registry, load_repo_intelligence_config,
};
use xiuxian_wendao::repo_index::start_repo_index_coordinator;
use xiuxian_wendao::search::SearchPlaneService;

pub(crate) fn gateway_state_for_project(project_root: &Path) -> Result<Arc<GatewayState>> {
    let config_root = project_root.to_path_buf();
    let ui_config = gateway_ui_config_for_project(config_root.as_path())?;
    let plugin_registry = Arc::new(bootstrap_builtin_registry()?);
    let search_plane = SearchPlaneService::new(project_root.to_path_buf());
    let repo_index = start_repo_index_coordinator(
        project_root.to_path_buf(),
        Arc::clone(&plugin_registry),
        search_plane.clone(),
    );

    let state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(StudioState {
            project_root: project_root.to_path_buf(),
            config_root: config_root.clone(),
            bootstrap_background_indexing: false,
            cold_start_process_started_at: crate::studio::symbol_index::timestamp_now(),
            cold_start_process_started_instant: std::time::Instant::now(),
            cold_start_telemetry: Arc::new(RwLock::new(
                crate::studio::router::StudioSearchColdStartTelemetryState::default(),
            )),
            bootstrap_background_indexing_deferred_activation: Arc::new(RwLock::new(None)),
            configured_owners: Arc::new(RwLock::new(
                StudioState::configured_owners_from_ui_config(ui_config),
            )),
            graph_index: Arc::new(RwLock::new(None)),
            symbol_index: Arc::new(RwLock::new(None)),
            symbol_index_coordinator: Arc::new(SymbolIndexCoordinator::new(
                project_root.to_path_buf(),
                config_root.clone(),
                search_plane.clone(),
            )),
            search_plane,
            vfs_scan: Arc::new(RwLock::new(None)),
            repo_index,
            plugin_registry,
        }),
    });

    Ok(state)
}

pub(crate) fn gateway_ui_config_for_project(config_root: &Path) -> Result<UiConfig> {
    let mut ui_config = load_ui_config_from_wendao_toml(config_root).unwrap_or_default();
    if !ui_config.repo_projects.is_empty() {
        return Ok(ui_config);
    }

    let effective_config_path = studio_effective_wendao_toml_path(config_root);
    let config = load_repo_intelligence_config(Some(effective_config_path.as_path()), config_root)?;
    ui_config.repo_projects = config
        .repos
        .into_iter()
        .map(ui_repo_project_from_registered_repository)
        .collect();
    Ok(ui_config)
}

fn ui_repo_project_from_registered_repository(
    repository: RegisteredRepository,
) -> UiRepoProjectConfig {
    UiRepoProjectConfig {
        id: repository.id,
        root: repository
            .path
            .map(|path| path.to_string_lossy().into_owned()),
        url: repository.url,
        git_ref: repository.git_ref.as_ref().map(repository_ref_string),
        refresh: Some(repository_refresh_policy_string(repository.refresh).to_string()),
        plugins: repository
            .plugins
            .into_iter()
            .map(|plugin| match plugin {
                RepositoryPluginConfig::Id(id) | RepositoryPluginConfig::Config { id, .. } => id,
            })
            .collect(),
    }
}

fn repository_ref_string(reference: &RepositoryRef) -> String {
    reference.as_str().to_string()
}

fn repository_refresh_policy_string(refresh: RepositoryRefreshPolicy) -> &'static str {
    match refresh {
        RepositoryRefreshPolicy::Fetch => "fetch",
        RepositoryRefreshPolicy::Manual => "manual",
    }
}
