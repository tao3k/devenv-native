use std::fs;
use std::path::{Path, PathBuf};

use crate::studio::perf_support::root::{
    DEFAULT_REAL_WORKSPACE_ROOT, GatewayPerfRoot, REAL_WORKSPACE_READY_TIMEOUT_ENV,
    REAL_WORKSPACE_ROOT_ENV, real_workspace_ready_timeout_with_lookup,
    resolve_real_workspace_root_with_lookup,
};
use crate::studio::perf_support::state::gateway_ui_config_for_project;
use crate::studio::perf_support::workspace::real_workspace_status_is_query_ready;
use xiuxian_wendao::repo_index::RepoIndexStatusResponse;

#[test]
fn resolve_real_workspace_root_prefers_explicit_env_override() {
    let resolved = resolve_real_workspace_root_with_lookup(Path::new("/tmp/project"), &|key| {
        (key == REAL_WORKSPACE_ROOT_ENV).then_some("/tmp/custom-workspace".to_string())
    });
    assert_eq!(resolved, Some(PathBuf::from("/tmp/custom-workspace")));
}

#[test]
fn resolve_real_workspace_root_resolves_relative_override_from_project_root() {
    let resolved = resolve_real_workspace_root_with_lookup(Path::new("/tmp/project"), &|key| {
        (key == REAL_WORKSPACE_ROOT_ENV).then_some(".data/wendao-frontend".to_string())
    });
    assert_eq!(
        resolved,
        Some(PathBuf::from("/tmp/project/.data/wendao-frontend"))
    );
}

#[test]
fn resolve_real_workspace_root_uses_default_frontend_workspace_when_present() {
    let root = std::env::temp_dir().join(format!(
        "xiuxian-wendao-perf-support-{}",
        uuid::Uuid::new_v4()
    ));
    let fallback = root.join(DEFAULT_REAL_WORKSPACE_ROOT);
    fs::create_dir_all(&fallback)
        .unwrap_or_else(|error| panic!("failed to create fallback workspace root: {error}"));

    let resolved = resolve_real_workspace_root_with_lookup(root.as_path(), &|_| None);
    assert_eq!(resolved, Some(fallback.clone()));

    fs::remove_dir_all(root)
        .unwrap_or_else(|error| panic!("failed to remove temporary perf support root: {error}"));
}

#[test]
fn real_workspace_ready_timeout_uses_positive_override() {
    let timeout = real_workspace_ready_timeout_with_lookup(&|key| {
        (key == REAL_WORKSPACE_READY_TIMEOUT_ENV).then(|| "120".to_string())
    });
    assert_eq!(timeout.as_secs(), 120);
}

#[test]
fn real_workspace_ready_timeout_ignores_zero_override() {
    let timeout = real_workspace_ready_timeout_with_lookup(&|key| {
        (key == REAL_WORKSPACE_READY_TIMEOUT_ENV).then(|| "0".to_string())
    });
    assert_eq!(timeout.as_secs(), 900);
}

#[test]
fn gateway_perf_root_preserves_external_paths() {
    let path = PathBuf::from("/tmp/external-workspace");
    let root = GatewayPerfRoot::External(path.clone());
    let resolved = match root {
        GatewayPerfRoot::Owned(inner) | GatewayPerfRoot::External(inner) => inner,
    };
    assert_eq!(resolved, path);
}

#[test]
fn gateway_ui_config_falls_back_to_repo_intelligence_projects() {
    let root = std::env::temp_dir().join(format!(
        "xiuxian-wendao-perf-ui-config-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&root)
        .unwrap_or_else(|error| panic!("failed to create temporary config root: {error}"));
    fs::write(
        root.join("wendao.toml"),
        r#"[link_graph.projects."ADTypes.jl"]
url = "https://github.com/SciML/ADTypes.jl.git"
refresh = "fetch"
plugins = ["julia-code-parser"]
"#,
    )
    .unwrap_or_else(|error| panic!("failed to write temporary wendao.toml: {error}"));

    let ui_config = gateway_ui_config_for_project(root.as_path())
        .unwrap_or_else(|error| panic!("failed to build fallback ui config: {error}"));
    assert_eq!(ui_config.repo_projects.len(), 1);
    assert_eq!(ui_config.repo_projects[0].id, "ADTypes.jl");
    assert_eq!(
        ui_config.repo_projects[0].url.as_deref(),
        Some("https://github.com/SciML/ADTypes.jl.git")
    );

    fs::remove_dir_all(root)
        .unwrap_or_else(|error| panic!("failed to remove temporary config root: {error}"));
}

#[test]
fn real_workspace_status_is_query_ready_requires_discovered_repositories() {
    let status = RepoIndexStatusResponse {
        total: 149,
        active: 12,
        queued: 11,
        checking: 0,
        syncing: 4,
        indexing: 8,
        ready: 9,
        unsupported: 0,
        failed: 0,
        target_concurrency: 1,
        max_concurrency: 4,
        sync_concurrency_limit: 1,
        current_repo_id: None,
        active_repo_ids: Vec::new(),
        repos: Vec::new(),
    };
    assert!(!real_workspace_status_is_query_ready(&status, 150));
}

#[test]
fn real_workspace_status_is_query_ready_accepts_partial_active_bootstrap() {
    let status = RepoIndexStatusResponse {
        total: 179,
        active: 57,
        queued: 23,
        checking: 4,
        syncing: 16,
        indexing: 37,
        ready: 12,
        unsupported: 1,
        failed: 0,
        target_concurrency: 1,
        max_concurrency: 4,
        sync_concurrency_limit: 1,
        current_repo_id: None,
        active_repo_ids: Vec::new(),
        repos: Vec::new(),
    };
    assert!(real_workspace_status_is_query_ready(&status, 150));
}
