use std::fs;
use std::sync::Arc;

use axum::body::to_bytes;
use axum::extract::{Path, Query, State};

use crate::analyzers::bootstrap_builtin_registry;
use crate::analyzers::registry::PluginRegistry;
use crate::gateway::studio::router::tests::repo_project;
use crate::gateway::studio::router::{GatewayState, StudioState};
use crate::gateway::studio::symbol_index::SymbolIndexPhase;
use crate::gateway::studio::types::UiPluginArtifact;
use crate::gateway::studio::types::{
    UiConfig, UiProjectConfig, UiRepoProjectConfig, VfsScanResult,
};
use crate::repo_index::RepoIndexPhase;
use crate::search::SearchPlaneService;
use crate::set_link_graph_wendao_config_override;
use crate::unified_symbol::UnifiedSymbolIndex;
use chrono::DateTime;
use serial_test::serial;
use xiuxian_wendao_builtin::{
    linked_builtin_julia_gateway_artifact_base_url,
    linked_builtin_julia_gateway_artifact_expected_toml_fragments,
    linked_builtin_julia_gateway_artifact_path, linked_builtin_julia_gateway_artifact_route,
    linked_builtin_julia_gateway_artifact_runtime_config_toml,
    linked_builtin_julia_gateway_artifact_schema_version,
    linked_builtin_julia_gateway_artifact_selected_transport,
    linked_builtin_julia_gateway_launcher_path,
};

#[test]
fn seed_eager_configured_owners_for_tests_preserves_cached_state_when_effectively_unchanged() {
    let studio = StudioState::new();
    let config = UiConfig {
        projects: vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec!["docs".to_string()],
        }],
        repo_projects: vec![repo_project("sciml")],
    };
    studio.seed_eager_configured_owners_for_tests(config.clone());

    *studio
        .symbol_index
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) =
        Some(Arc::new(UnifiedSymbolIndex::new()));
    *studio
        .vfs_scan
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(VfsScanResult {
        entries: Vec::new(),
        file_count: 0,
        dir_count: 0,
        scan_duration_ms: 0,
    });

    studio.seed_eager_configured_owners_for_tests(config);

    assert!(
        studio
            .vfs_scan
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    );
    let repo_status = studio.repo_index.status_response(None);
    assert_eq!(repo_status.total, 1);
    assert_ne!(repo_status.repos[0].phase, RepoIndexPhase::Idle);
    assert_ne!(
        studio.symbol_index_coordinator.status().phase,
        SymbolIndexPhase::Idle
    );
}

#[test]
fn seed_configured_owners_for_tests_without_eager_background_indexing_keeps_indexes_idle() {
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

    let repo_status = studio.repo_index.status_response(None);
    assert_eq!(repo_status.total, 0);
    assert!(repo_status.repos.is_empty());
    assert_eq!(
        studio.symbol_index_coordinator.status().phase,
        SymbolIndexPhase::Idle
    );
}

#[tokio::test]
async fn seed_eager_configured_owners_for_tests_still_eagerly_enqueues_background_indexes() {
    let studio = StudioState::new();

    studio.seed_eager_configured_owners_for_tests(UiConfig {
        projects: vec![UiProjectConfig {
            name: "kernel".to_string(),
            root: ".".to_string(),
            dirs: vec!["docs".to_string()],
        }],
        repo_projects: vec![repo_project("sciml")],
    });

    let repo_status = studio.repo_index.status_response(None);
    assert_eq!(repo_status.total, 1);
    assert_ne!(repo_status.repos[0].phase, RepoIndexPhase::Idle);
    assert_ne!(
        studio.symbol_index_coordinator.status().phase,
        SymbolIndexPhase::Idle
    );
}

#[test]
fn studio_bootstrap_uses_explicit_gateway_config_path_and_its_imports() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let project_root = temp.path().join("project");
    let frontend_root = project_root.join(".data").join("wendao-frontend");
    fs::create_dir_all(frontend_root.as_path())
        .unwrap_or_else(|error| panic!("create frontend root: {error}"));

    let gateway_config_path = project_root.join("wendao.toml");
    fs::write(
        &gateway_config_path,
        r#"imports = [".data/wendao-frontend/wendao.toml"]

[link_graph.projects.main]
root = "."
dirs = ["docs"]
"#,
    )
    .unwrap_or_else(|error| panic!("write gateway config: {error}"));
    fs::write(
        frontend_root.join("wendao.toml"),
        r#"[link_graph.projects.kernel]
root = "."
dirs = ["docs"]

[link_graph.projects.frontend]
root = "."
dirs = ["src"]
"#,
    )
    .unwrap_or_else(|error| panic!("write frontend config: {error}"));

    let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane_and_path(
        Arc::new(PluginRegistry::new()),
        project_root.clone(),
        gateway_config_path
            .parent()
            .unwrap_or_else(|| panic!("gateway config should have parent"))
            .to_path_buf(),
        Some(gateway_config_path.as_path()),
        SearchPlaneService::new(project_root),
    );

    assert_eq!(
        studio.ui_config(),
        UiConfig {
            projects: vec![
                UiProjectConfig {
                    name: "frontend".to_string(),
                    root: ".".to_string(),
                    dirs: vec!["src".to_string()],
                },
                UiProjectConfig {
                    name: "kernel".to_string(),
                    root: ".".to_string(),
                    dirs: vec!["docs".to_string()],
                },
                UiProjectConfig {
                    name: "main".to_string(),
                    root: ".".to_string(),
                    dirs: vec!["docs".to_string()],
                },
            ],
            repo_projects: Vec::new(),
        }
    );
}

#[test]
fn studio_bootstrap_preserves_imported_search_only_repo_projects_from_explicit_root_config() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let project_root = temp.path().join("project");
    fs::create_dir_all(project_root.as_path())
        .unwrap_or_else(|error| panic!("create project root: {error}"));
    let frontend_root = project_root.join(".data").join("wendao-frontend");
    fs::create_dir_all(frontend_root.as_path())
        .unwrap_or_else(|error| panic!("create frontend root: {error}"));

    fs::write(
        project_root.join("github-repo-list.toml"),
        r#"[link_graph.projects.lance]
dirs = []
url = "https://github.com/lance-format/lance"
refresh = "fetch"
plugins = ["ast-grep"]
"#,
    )
    .unwrap_or_else(|error| panic!("write repo list: {error}"));

    let gateway_config_path = project_root.join("wendao.toml");
    fs::write(
        &gateway_config_path,
        r#"imports = ["github-repo-list.toml", ".data/wendao-frontend/wendao.toml"]

[link_graph.projects.main]
root = "."
dirs = ["docs"]
"#,
    )
    .unwrap_or_else(|error| panic!("write gateway config: {error}"));
    fs::write(
        frontend_root.join("wendao.toml"),
        r#"[link_graph.projects.frontend]
root = "."
dirs = ["src"]
"#,
    )
    .unwrap_or_else(|error| panic!("write frontend config: {error}"));

    let studio = StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane_and_path(
        Arc::new(PluginRegistry::new()),
        project_root.clone(),
        gateway_config_path
            .parent()
            .unwrap_or_else(|| panic!("gateway config should have parent"))
            .to_path_buf(),
        Some(gateway_config_path.as_path()),
        SearchPlaneService::new(project_root),
    );

    assert_eq!(
        studio.ui_config().repo_projects,
        vec![UiRepoProjectConfig {
            id: "lance".to_string(),
            root: None,
            url: Some("https://github.com/lance-format/lance".to_string()),
            git_ref: None,
            refresh: Some("fetch".to_string()),
            plugins: vec!["ast-grep".to_string()],
        }],
    );
}

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
    let expected = expected.into_iter().collect::<Vec<_>>();
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

    let response =
        crate::gateway::studio::router::handlers::get_ui_capabilities(State(Arc::clone(&state)))
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
        crate::gateway::studio::router::state::supported_code_kinds()
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

#[tokio::test]
#[serial]
async fn plugin_artifact_handler_returns_resolved_artifact() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let config_path = temp.path().join("wendao.toml");
    let (plugin_id, artifact_id) = linked_builtin_julia_gateway_artifact_path();
    fs::write(
        &config_path,
        linked_builtin_julia_gateway_artifact_runtime_config_toml(Some("similarity_only")),
    )
    .unwrap_or_else(|error| panic!("write config: {error}"));
    let config_path_string = config_path.to_string_lossy().to_string();
    set_link_graph_wendao_config_override(&config_path_string);

    let state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(StudioState::new()),
    });

    let response = crate::gateway::studio::router::handlers::get_plugin_artifact(
        State(Arc::clone(&state)),
        Path(
            crate::gateway::studio::router::handlers::capabilities::PluginArtifactPath {
                plugin_id: plugin_id.clone(),
                artifact_id: artifact_id.clone(),
            },
        ),
        Query(
            crate::gateway::studio::router::handlers::capabilities::PluginArtifactQuery {
                format: None,
            },
        ),
    )
    .await
    .unwrap_or_else(|error| panic!("deployment artifact handler should resolve: {error:?}"));

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|error| panic!("read json body: {error}"));
    let artifact: UiPluginArtifact = serde_json::from_slice(&body)
        .unwrap_or_else(|error| panic!("decode artifact json: {error}"));

    assert_eq!(artifact.plugin_id, plugin_id);
    assert_eq!(artifact.artifact_id, artifact_id);
    assert_eq!(
        artifact.artifact_schema_version,
        linked_builtin_julia_gateway_artifact_schema_version()
    );
    DateTime::parse_from_rfc3339(&artifact.generated_at)
        .unwrap_or_else(|error| panic!("parse artifact generated_at: {error}"));
    assert_eq!(
        artifact.base_url.as_deref(),
        Some(linked_builtin_julia_gateway_artifact_base_url())
    );
    assert_eq!(
        artifact.route.as_deref(),
        Some(linked_builtin_julia_gateway_artifact_route())
    );
    assert_eq!(
        artifact.schema_version.as_deref(),
        Some(linked_builtin_julia_gateway_artifact_schema_version())
    );
    assert_eq!(
        artifact.selected_transport,
        Some(crate::gateway::studio::types::config::UiPluginTransportKind::ArrowFlight)
    );
    assert_eq!(artifact.fallback_from, None);
    assert_eq!(artifact.fallback_reason, None);
    assert_eq!(
        artifact
            .launch
            .as_ref()
            .map(|launch| launch.launcher_path.as_str()),
        Some(linked_builtin_julia_gateway_launcher_path())
    );
}

#[tokio::test]
#[serial]
async fn plugin_artifact_handler_returns_canonical_json_shape() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let config_path = temp.path().join("wendao.toml");
    let (plugin_id, artifact_id) = linked_builtin_julia_gateway_artifact_path();
    fs::write(
        &config_path,
        linked_builtin_julia_gateway_artifact_runtime_config_toml(None),
    )
    .unwrap_or_else(|error| panic!("write config: {error}"));
    let config_path_string = config_path.to_string_lossy().to_string();
    set_link_graph_wendao_config_override(&config_path_string);

    let state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(StudioState::new()),
    });

    let response = crate::gateway::studio::router::handlers::get_plugin_artifact(
        State(Arc::clone(&state)),
        Path(
            crate::gateway::studio::router::handlers::capabilities::PluginArtifactPath {
                plugin_id: plugin_id.clone(),
                artifact_id: artifact_id.clone(),
            },
        ),
        Query(
            crate::gateway::studio::router::handlers::capabilities::PluginArtifactQuery {
                format: None,
            },
        ),
    )
    .await
    .unwrap_or_else(|error| panic!("deployment artifact handler should resolve: {error:?}"));

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|error| panic!("read json body: {error}"));
    let artifact: serde_json::Value = serde_json::from_slice(&body)
        .unwrap_or_else(|error| panic!("decode artifact json: {error}"));

    assert_eq!(artifact["pluginId"], plugin_id);
    assert_eq!(artifact["artifactId"], artifact_id);
    assert_eq!(
        artifact["selectedTransport"],
        linked_builtin_julia_gateway_artifact_selected_transport()
    );
    assert_eq!(
        artifact["launch"]["launcherPath"],
        linked_builtin_julia_gateway_launcher_path()
    );
}

#[tokio::test]
#[serial]
async fn plugin_artifact_handler_returns_toml_when_requested() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let config_path = temp.path().join("wendao.toml");
    let (plugin_id, artifact_id) = linked_builtin_julia_gateway_artifact_path();
    fs::write(
        &config_path,
        linked_builtin_julia_gateway_artifact_runtime_config_toml(None),
    )
    .unwrap_or_else(|error| panic!("write config: {error}"));
    let config_path_string = config_path.to_string_lossy().to_string();
    set_link_graph_wendao_config_override(&config_path_string);

    let state = Arc::new(GatewayState {
        index: None,
        signal_tx: None,
        webhook_url: None,
        studio: Arc::new(StudioState::new()),
    });

    let response = crate::gateway::studio::router::handlers::get_plugin_artifact(
        State(Arc::clone(&state)),
        Path(
            crate::gateway::studio::router::handlers::capabilities::PluginArtifactPath {
                plugin_id,
                artifact_id,
            },
        ),
        Query(
            crate::gateway::studio::router::handlers::capabilities::PluginArtifactQuery {
                format: Some(crate::zhenfa_router::native::WendaoPluginArtifactOutputFormat::Toml),
            },
        ),
    )
    .await
    .unwrap_or_else(|error| panic!("deployment artifact toml handler should resolve: {error:?}"));

    let content_type = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|error| panic!("read toml body: {error}"));
    let body_text =
        String::from_utf8(body.to_vec()).unwrap_or_else(|error| panic!("utf8 toml body: {error}"));

    assert_eq!(content_type, "text/plain; charset=utf-8");
    for expected_fragment in linked_builtin_julia_gateway_artifact_expected_toml_fragments() {
        assert!(
            body_text.contains(&expected_fragment),
            "expected rendered TOML to contain `{expected_fragment}`"
        );
    }
}
