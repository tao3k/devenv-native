use std::path::PathBuf;
use std::sync::Arc;

use crate::analyzers::PluginRegistry;
use crate::search::SearchPlaneService;

use super::state::RepoIndexCoordinator;

/// Start the background repository index coordinator for the configured project.
#[must_use]
pub fn start_repo_index_coordinator(
    project_root: PathBuf,
    plugin_registry: Arc<PluginRegistry>,
    search_plane: SearchPlaneService,
) -> Arc<RepoIndexCoordinator> {
    let repo_index = Arc::new(RepoIndexCoordinator::new(
        project_root,
        plugin_registry,
        search_plane,
    ));
    repo_index.start();
    repo_index
}
