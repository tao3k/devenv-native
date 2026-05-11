use std::path::Path;

use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::{RegisteredRepository, load_repo_intelligence_config};

/// Load one registered repository from configuration by id.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when config loading fails or the
/// repository id is unknown.
pub fn load_registered_repository(
    repo_id: &str,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RegisteredRepository, RepoIntelligenceError> {
    let config = load_repo_intelligence_config(config_path, cwd)?;
    config
        .repos
        .into_iter()
        .find(|repository| repository.id == repo_id)
        .ok_or_else(|| RepoIntelligenceError::UnknownRepository {
            repo_id: repo_id.to_string().into(),
        })
}
