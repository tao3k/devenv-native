use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use xiuxian_wendao_server::transport::{
    EffectiveRerankFlightHostSettings, EffectiveRerankFlightHostSettingsInput, RerankScoreWeights,
    rerank_score_weights_from_env, resolve_effective_rerank_flight_host_settings,
    split_rerank_flight_host_overrides,
};

use crate::link_graph::{
    resolve_link_graph_rerank_flight_runtime_settings, set_link_graph_wendao_config_override,
};

#[derive(Debug, Clone)]
pub(super) struct RepoSearchFlightHostConfig {
    pub(super) bind_addr: SocketAddr,
    pub(super) repo_id: String,
    pub(super) project_root: PathBuf,
    pub(super) config_path: Option<PathBuf>,
    pub(super) effective_settings: EffectiveRerankFlightHostSettings,
}

impl RepoSearchFlightHostConfig {
    pub(super) fn from_args(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut args = args.into_iter();
        let bind_addr = args
            .next()
            .unwrap_or_else(|| "127.0.0.1:0".to_string())
            .parse::<SocketAddr>()
            .map_err(|error| anyhow!("invalid bind address: {error}"))?;
        let parsed_overrides =
            split_rerank_flight_host_overrides(args).map_err(anyhow::Error::msg)?;
        let mut positional_args = parsed_overrides.positional_args.into_iter();
        let repo_id = positional_args
            .next()
            .unwrap_or_else(|| "alpha/repo".to_string());
        let project_root = positional_args
            .next()
            .map(PathBuf::from)
            .map_or_else(resolve_current_dir, Ok)?;
        let positional_rerank_dimension = positional_args
            .next()
            .map(|value| {
                value
                    .parse::<usize>()
                    .map_err(|error| anyhow!("invalid rerank dimension: {error}"))
            })
            .transpose()?
            .unwrap_or(3);
        let config_path = resolve_runtime_config_path(project_root.as_path());
        if let Some(config_path) = config_path.as_ref()
            && let Some(path_str) = config_path.to_str()
        {
            set_link_graph_wendao_config_override(path_str);
        }
        let effective_settings = resolve_effective_search_host_settings(
            parsed_overrides.schema_version_override,
            parsed_overrides.rerank_dimension_override,
            positional_rerank_dimension,
        )?;
        Ok(Self {
            bind_addr,
            repo_id,
            project_root,
            config_path,
            effective_settings,
        })
    }
}

pub(super) fn resolve_runtime_config_path(project_root: &Path) -> Option<PathBuf> {
    let local_config = Path::new("wendao.toml");
    if local_config.exists() {
        return std::env::current_dir()
            .ok()
            .map(|cwd| cwd.join(local_config));
    }

    let project_config = project_root.join("wendao.toml");
    project_config.exists().then_some(project_config)
}

fn resolve_current_dir() -> Result<PathBuf> {
    std::env::current_dir().map_err(|error| anyhow!("failed to resolve current dir: {error}"))
}

fn resolve_effective_search_host_settings(
    schema_version_override: Option<String>,
    rerank_dimension_override: Option<usize>,
    fallback_rerank_dimension: usize,
) -> Result<EffectiveRerankFlightHostSettings> {
    let file_backed_settings = resolve_link_graph_rerank_flight_runtime_settings();
    let file_backed_weights = file_backed_settings
        .score_weights
        .map(|weights| RerankScoreWeights::new(weights.vector_weight, weights.semantic_weight))
        .transpose()
        .map_err(anyhow::Error::msg)?;
    Ok(resolve_effective_rerank_flight_host_settings(
        EffectiveRerankFlightHostSettingsInput {
            schema_version_override,
            rerank_dimension_override,
            file_backed_schema_version: file_backed_settings.schema_version,
            file_backed_weights,
            fallback_dimension: fallback_rerank_dimension,
            fallback_weights: rerank_score_weights_from_env().map_err(anyhow::Error::msg)?,
        },
    ))
}
