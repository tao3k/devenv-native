use std::fs;
use std::sync::Arc;

use crate::analyzers::PluginRegistry;
use crate::gateway::studio::router::{GatewayState, StudioState};
use crate::set_link_graph_wendao_config_override;
use xiuxian_wendao_builtin::linked_builtin_julia_gateway_artifact_path;

pub(super) fn expected_builtin_languages(registry: &PluginRegistry) -> Vec<String> {
    let registry_languages = registry
        .plugin_ids()
        .into_iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>();
    let mut expected = xiuxian_ast::Lang::all()
        .iter()
        .copied()
        .map(xiuxian_ast::Lang::as_str)
        .map(std::string::ToString::to_string)
        .collect::<std::collections::BTreeSet<_>>();
    expected.extend(registry_languages);
    expected.into_iter().collect::<Vec<_>>()
}

pub(super) fn assert_repo_discovery_contract(
    repo_discovery: &crate::gateway::studio::types::UiRepoDiscoveryContract,
) {
    assert_eq!(repo_discovery.suggest.source, "repo_index_status");
    assert_eq!(repo_discovery.suggest.default_limit, 6);
    assert!(repo_discovery.suggest.exhaustive);
    assert!(!repo_discovery.suggest.query_scoped);

    assert_eq!(repo_discovery.facet.source, "search_results");
    assert_eq!(repo_discovery.facet.default_limit, 6);
    assert!(!repo_discovery.facet.exhaustive);
    assert!(repo_discovery.facet.query_scoped);

    assert_eq!(repo_discovery.inventory.source, "repo_index_status");
    assert_eq!(repo_discovery.inventory.default_limit, 200);
    assert!(repo_discovery.inventory.exhaustive);
    assert!(!repo_discovery.inventory.query_scoped);
}

pub(super) fn plugin_artifact_state(
    runtime_config_toml: &str,
) -> (
    Arc<GatewayState>,
    crate::gateway::studio::router::handlers::capabilities::PluginArtifactPath,
) {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let config_path = temp.path().join("wendao.toml");
    let (plugin_id, artifact_id) = linked_builtin_julia_gateway_artifact_path();
    fs::write(&config_path, runtime_config_toml)
        .unwrap_or_else(|error| panic!("write config: {error}"));
    let config_path_string = config_path.to_string_lossy().to_string();
    set_link_graph_wendao_config_override(&config_path_string);

    let state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(StudioState::new()),
    });

    std::mem::forget(temp);

    (
        state,
        crate::gateway::studio::router::handlers::capabilities::PluginArtifactPath {
            plugin_id,
            artifact_id,
        },
    )
}
