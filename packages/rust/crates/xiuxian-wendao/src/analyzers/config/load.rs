use std::path::{Path, PathBuf};

use crate::analyzers::RepoIntelligenceError;
use xiuxian_config_core::load_toml_value_with_imports;

use super::parse::{
    normalize_path, parse_refresh_policy, parse_repository_plugins, parse_repository_ref,
};
use super::toml::WendaoTomlConfig;
use super::types::{RegisteredRepository, RepoIntelligenceConfig};

/// Load the repo intelligence configuration from the project.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when configuration cannot be loaded.
pub fn load_repo_intelligence_config(
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoIntelligenceConfig, RepoIntelligenceError> {
    let config_path = config_path.map_or_else(|| cwd.join("wendao.toml"), Path::to_path_buf);
    let merged = load_toml_value_with_imports(config_path.as_path()).map_err(|error| {
        RepoIntelligenceError::ConfigLoad {
            message: format!("failed to read `{}`: {error}", config_path.display()),
        }
    })?;
    let parsed: WendaoTomlConfig =
        merged
            .try_into()
            .map_err(|error| RepoIntelligenceError::ConfigLoad {
                message: format!("failed to parse `{}`: {error}", config_path.display()),
            })?;

    let config_root = config_path
        .parent()
        .map_or_else(|| cwd.to_path_buf(), Path::to_path_buf);

    let repos = parsed
        .link_graph
        .projects
        .into_iter()
        .map(|(id, project)| {
            let plugins = parse_repository_plugins(project.plugins, &id, &config_path)?;
            let path = project
                .root
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
                .map(|path| {
                    if path.is_absolute() {
                        normalize_path(path.as_path())
                    } else {
                        normalize_path(config_root.join(path).as_path())
                    }
                });
            let url = project
                .url
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            if path.is_none() && url.is_none() {
                return Ok(None);
            }

            let mut repository = RegisteredRepository {
                id,
                path,
                url,
                git_ref: project.git_ref.as_deref().and_then(parse_repository_ref),
                refresh: parse_refresh_policy(project.refresh.as_deref()),
                plugins,
            };
            if !repository.has_repo_intelligence_plugins() {
                return Ok(None);
            }
            repository.plugins = repository.repo_intelligence_plugins().cloned().collect();

            Ok(Some(repository))
        })
        .collect::<Result<Vec<_>, RepoIntelligenceError>>()?
        .into_iter()
        .flatten()
        .collect();

    Ok(RepoIntelligenceConfig { repos })
}
