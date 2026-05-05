use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::analyzers::PluginRegistry;
use crate::analyzers::{RegisteredRepository, RepositoryPluginConfig, RepositoryRefreshPolicy};
use crate::repo_index::state::coordinator::RepoIndexCoordinator;
use crate::test_support::{commit_all, init_git_repository};

pub(crate) fn repo(id: &str, path: &str) -> RegisteredRepository {
    RegisteredRepository {
        id: id.to_string(),
        path: Some(PathBuf::from(path)),
        url: None,
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("julia".to_string())],
    }
}

pub(crate) fn remote_repo(id: &str, url: &str) -> RegisteredRepository {
    RegisteredRepository {
        id: id.to_string(),
        path: None,
        url: Some(url.to_string()),
        git_ref: None,
        refresh: RepositoryRefreshPolicy::Fetch,
        plugins: vec![RepositoryPluginConfig::Id("julia".to_string())],
    }
}

pub(crate) fn new_coordinator(
    search_plane: crate::search::SearchPlaneService,
) -> RepoIndexCoordinator {
    new_coordinator_with_registry(search_plane, Arc::new(PluginRegistry::new()))
}

pub(crate) fn new_coordinator_with_registry(
    search_plane: crate::search::SearchPlaneService,
    plugin_registry: Arc<PluginRegistry>,
) -> RepoIndexCoordinator {
    RepoIndexCoordinator::new(PathBuf::from("."), plugin_registry, search_plane)
}

pub(crate) fn init_test_repository(root: &std::path::Path) {
    init_git_repository(root);
    fs::write(root.join("Project.toml"), "name = \"RepoIndexWarmStart\"\n")
        .unwrap_or_else(|error| panic!("write project file: {error}"));
    commit_all(root, "init");
}
