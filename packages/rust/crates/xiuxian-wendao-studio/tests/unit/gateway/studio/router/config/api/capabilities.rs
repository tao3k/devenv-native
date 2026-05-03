use std::sync::Arc;

use axum::extract::State;
use chrono::DateTime;

use crate::studio::router::tests::repo_project;
use crate::studio::router::{GatewayState, StudioState};
use crate::studio::test_support::assert_studio_json_snapshot;
use xiuxian_wendao::analyzers::bootstrap_builtin_registry;
use xiuxian_wendao::repo_index::RepoIndexPhase;
use xiuxian_wendao::search::contracts::{UiConfig, UiProjectConfig};

use super::support::{assert_repo_discovery_contract, expected_builtin_languages};

#[tokio::test]
async fn repo_index_status_bootstraps_deferred_repo_indexing() {
    let studio = StudioState::new();

    studio.seed_configured_owners_for_tests(
        UiConfig {
            projects: vec![UiProjectConfig {
                name: "kernel".to_string(),
                root: ".".to_string(),
                dirs: vec!["docs".to_string()],
            }],
            repo_projects: vec![repo_project("sciml")],
        },
        false,
    );

    assert_eq!(studio.repo_index.status_response(None).total, 0);
    assert_eq!(
        studio.bootstrap_background_indexing_deferred_activation_source(),
        None
    );

    let repo_status = studio.repo_index_status(None);

    assert_eq!(repo_status.total, 1);
    assert_eq!(repo_status.repos[0].repo_id, "sciml");
    assert_ne!(repo_status.repos[0].phase, RepoIndexPhase::Idle);
    assert_eq!(
        studio.bootstrap_background_indexing_deferred_activation_source(),
        Some("repo_index_status".to_string())
    );
}

#[tokio::test]
async fn ui_capabilities_reports_builtin_plugin_languages() {
    let registry = bootstrap_builtin_registry()
        .unwrap_or_else(|error| panic!("builtin registry should bootstrap: {error:?}"));
    let expected = expected_builtin_languages(&registry);
    let studio = StudioState::new_with_bootstrap_ui_config(Arc::new(registry));
    studio.seed_configured_owners_for_tests(
        UiConfig {
            projects: Vec::new(),
            repo_projects: vec![repo_project("kernel"), repo_project("sciml")],
        },
        false,
    );
    let state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(studio),
    });

    let response = crate::studio::router::handlers::capabilities::get(State(Arc::clone(&state)))
        .await
        .unwrap_or_else(|error| panic!("ui capabilities should resolve: {error:?}"))
        .0;

    assert_eq!(response.projects.len(), 0);
    assert_eq!(
        response.repo_projects,
        vec![repo_project("kernel"), repo_project("sciml")]
    );
    assert_eq!(response.languages, expected);
    assert_eq!(response.repositories, vec!["kernel", "sciml"]);
    assert_eq!(
        response.kinds,
        crate::studio::router::state::supported_code_kinds()
    );
    assert_eq!(response.search_contract.contract_version, "1");
    assert_eq!(response.search_contract.code_search.intent, "code_search");
    assert_eq!(
        response.search_contract.code_search.backend_prefixes,
        vec!["lang", "kind", "repo"]
    );
    assert_eq!(
        response.search_contract.code_search.composed_prefixes,
        vec!["path"]
    );
    assert_eq!(
        response.search_contract.code_search.backend_kind_filters,
        vec!["file", "symbol", "function", "module", "example"]
    );
    assert_repo_discovery_contract(&response.search_contract.repo_discovery);
    assert_studio_json_snapshot(
        "ui_capabilities_search_contract_payload",
        &response.search_contract,
    );
    assert!(!response.studio_bootstrap_background_indexing_enabled);
    assert_eq!(
        response.studio_bootstrap_background_indexing_mode,
        "deferred"
    );
    assert!(!response.studio_bootstrap_background_indexing_deferred_activation_observed);
}

#[tokio::test]
async fn symbol_index_status_records_first_deferred_bootstrap_activation() {
    let studio = StudioState::new();
    studio.seed_configured_owners_for_tests(
        UiConfig {
            projects: vec![UiProjectConfig {
                name: "kernel".to_string(),
                root: ".".to_string(),
                dirs: vec!["docs".to_string()],
            }],
            repo_projects: vec![repo_project("sciml")],
        },
        false,
    );

    assert_eq!(
        studio.bootstrap_background_indexing_deferred_activation_at(),
        None
    );
    assert_eq!(
        studio.bootstrap_background_indexing_deferred_activation_source(),
        None
    );

    let _ = studio
        .symbol_index_status()
        .unwrap_or_else(|error| panic!("symbol index status should resolve: {error:?}"));

    let activated_at = studio
        .bootstrap_background_indexing_deferred_activation_at()
        .unwrap_or_else(|| panic!("deferred activation should record a timestamp"));
    DateTime::parse_from_rfc3339(&activated_at)
        .unwrap_or_else(|error| panic!("parse deferred activation timestamp: {error}"));
    assert!(
        studio
            .bootstrap_background_indexing_telemetry()
            .deferred_activation_observed()
    );
    assert_eq!(
        studio.bootstrap_background_indexing_deferred_activation_source(),
        Some("symbol_index_status".to_string())
    );
}
