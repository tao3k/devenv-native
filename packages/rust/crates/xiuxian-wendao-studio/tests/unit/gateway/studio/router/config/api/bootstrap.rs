use std::fs;
use std::sync::Arc;

use crate::contracts::{UiConfig, UiProjectConfig, UiRepoProjectConfig, VfsScanResult};
use crate::studio::router::StudioState;
use crate::studio::router::tests::repo_project;
use crate::studio::symbol_index::SymbolIndexPhase;
use xiuxian_wendao::analyzers::PluginRegistry;
use xiuxian_wendao::repo_index::RepoIndexPhase;
use xiuxian_wendao::search::SearchPlaneService;
use xiuxian_wendao::unified_symbol::UnifiedSymbolIndex;

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
            repo_projects: vec![
                UiRepoProjectConfig {
                    id: "frontend".to_string(),
                    root: Some(".".to_string()),
                    url: None,
                    git_ref: None,
                    refresh: None,
                    plugins: vec!["markdown-parser".to_string()],
                },
                UiRepoProjectConfig {
                    id: "kernel".to_string(),
                    root: Some(".".to_string()),
                    url: None,
                    git_ref: None,
                    refresh: None,
                    plugins: vec!["markdown-parser".to_string()],
                },
                UiRepoProjectConfig {
                    id: "main".to_string(),
                    root: Some(".".to_string()),
                    url: None,
                    git_ref: None,
                    refresh: None,
                    plugins: vec!["markdown-parser".to_string()],
                },
            ],
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

#[test]
fn eager_bootstrap_enqueues_imported_search_only_repo_projects() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let project_root = temp.path().join("project");
    fs::create_dir_all(project_root.as_path())
        .unwrap_or_else(|error| panic!("create project root: {error}"));

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
        r#"imports = ["github-repo-list.toml"]

[link_graph.projects.main]
root = "."
dirs = ["docs"]
"#,
    )
    .unwrap_or_else(|error| panic!("write gateway config: {error}"));

    let studio =
        StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane_and_path_and_background_indexing(
            Arc::new(PluginRegistry::new()),
            project_root.clone(),
            gateway_config_path
                .parent()
                .unwrap_or_else(|| panic!("gateway config should have parent"))
                .to_path_buf(),
            Some(gateway_config_path.as_path()),
            SearchPlaneService::new(project_root),
            true,
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
    assert_eq!(
        studio.repo_index.pending_repo_ids_for_test(),
        vec!["lance".to_string()]
    );
    let repo_status = studio.repo_index.status_response(None);
    assert_eq!(repo_status.total, 1);
    assert_eq!(repo_status.repos[0].repo_id, "lance");
    assert_ne!(repo_status.repos[0].phase, RepoIndexPhase::Idle);
}

#[test]
fn eager_bootstrap_enqueues_imported_repo_intelligence_projects() {
    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir: {error}"));
    let project_root = temp.path().join("project");
    fs::create_dir_all(project_root.as_path())
        .unwrap_or_else(|error| panic!("create project root: {error}"));

    fs::write(
        project_root.join("github-repo-list.toml"),
        r#"[link_graph.projects.sciml]
dirs = []
url = "https://github.com/SciML/OrdinaryDiffEq.jl"
refresh = "fetch"
plugins = ["julia-code-parser"]
"#,
    )
    .unwrap_or_else(|error| panic!("write repo list: {error}"));

    let gateway_config_path = project_root.join("wendao.toml");
    fs::write(
        &gateway_config_path,
        r#"imports = ["github-repo-list.toml"]

[link_graph.projects.main]
root = "."
dirs = ["docs"]
"#,
    )
    .unwrap_or_else(|error| panic!("write gateway config: {error}"));

    let studio =
        StudioState::new_with_bootstrap_ui_config_for_roots_and_search_plane_and_path_and_background_indexing(
            Arc::new(PluginRegistry::new()),
            project_root.clone(),
            gateway_config_path
                .parent()
                .unwrap_or_else(|| panic!("gateway config should have parent"))
                .to_path_buf(),
            Some(gateway_config_path.as_path()),
            SearchPlaneService::new(project_root),
            true,
        );

    assert_eq!(
        studio.ui_config().repo_projects,
        vec![UiRepoProjectConfig {
            id: "sciml".to_string(),
            root: None,
            url: Some("https://github.com/SciML/OrdinaryDiffEq.jl".to_string()),
            git_ref: None,
            refresh: Some("fetch".to_string()),
            plugins: vec!["julia-code-parser".to_string()],
        }],
    );
    assert_eq!(
        studio.repo_index.pending_repo_ids_for_test(),
        vec!["sciml".to_string()]
    );
    let repo_status = studio.repo_index.status_response(None);
    assert_eq!(repo_status.total, 1);
    assert_eq!(repo_status.repos[0].repo_id, "sciml");
    assert_ne!(repo_status.repos[0].phase, RepoIndexPhase::Idle);
}
