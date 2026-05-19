//! Agent Org read-model configuration resolution.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use xiuxian_config_core::{
    load_toml_value_with_imports_and_paths, resolve_cache_home, resolve_config_home,
    resolve_path_from_value,
};

use crate::ClientContext;

use super::model::ResolvedReadModelSettings;

#[derive(Debug, Clone, Default, Deserialize)]
struct WendaoTomlConfig {
    #[serde(default)]
    agent: AgentTomlConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct AgentTomlConfig {
    #[serde(default)]
    org_read_model: AgentOrgReadModelTomlConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct AgentOrgReadModelTomlConfig {
    database_path: Option<String>,
    temp_directory: Option<String>,
    threads: Option<u64>,
    memory_limit: Option<String>,
    max_temp_directory_size: Option<String>,
    materialize_threshold_rows: Option<u64>,
}

pub(super) fn resolve_read_model_settings(
    context: &ClientContext,
) -> Result<ResolvedReadModelSettings> {
    let cache_home = resolve_cache_home(Some(context.root()))
        .with_context(|| "failed to resolve PRJ_CACHE_HOME for agent read model")?;
    let default_database_path = cache_home
        .join("agent")
        .join("readmodels")
        .join("org_agent_tasks.duckdb");
    let default_temp_directory = cache_home
        .join("agent")
        .join("readmodels")
        .join("duckdb-tmp");
    let default_threads =
        std::thread::available_parallelism().map_or(1, |count| count.get() as u64);

    let toml_config = load_agent_org_read_model_config(context)?;
    let database_path = toml_config
        .database_path
        .as_deref()
        .map_or(default_database_path, |value| {
            resolve_config_path_value(value, context.root(), cache_home.as_path())
        });
    let temp_directory = toml_config
        .temp_directory
        .as_deref()
        .map_or(default_temp_directory, |value| {
            resolve_config_path_value(value, context.root(), cache_home.as_path())
        });

    Ok(ResolvedReadModelSettings {
        cache_home,
        database_path,
        temp_directory,
        threads: toml_config.threads.unwrap_or(default_threads).max(1),
        memory_limit: normalized_optional_string(toml_config.memory_limit),
        max_temp_directory_size: normalized_optional_string(toml_config.max_temp_directory_size),
        materialize_threshold_rows: toml_config.materialize_threshold_rows.unwrap_or(1),
    })
}

fn load_agent_org_read_model_config(
    context: &ClientContext,
) -> Result<AgentOrgReadModelTomlConfig> {
    let Some(config_path) = resolve_config_path(context)? else {
        return Ok(AgentOrgReadModelTomlConfig::default());
    };
    let config_home = resolve_config_home(Some(context.root()));
    let merged = load_toml_value_with_imports_and_paths(
        config_path.as_path(),
        Some(context.root()),
        config_home.as_deref(),
    )
    .with_context(|| {
        format!(
            "failed to load agent read-model config from `{}`",
            config_path.display()
        )
    })?;
    let parsed: WendaoTomlConfig = merged.try_into().with_context(|| {
        format!(
            "failed to parse agent read-model config from `{}`",
            config_path.display()
        )
    })?;
    Ok(parsed.agent.org_read_model)
}

fn resolve_config_path(context: &ClientContext) -> Result<Option<PathBuf>> {
    if let Some(config_path) = context.config_file() {
        let config_path = config_path.to_path_buf();
        if !config_path.is_file() {
            anyhow::bail!(
                "configured agent read-model config `{}` does not exist or is not a file",
                config_path.display()
            );
        }
        return Ok(Some(config_path));
    }

    let default_path = context.root().join("wendao.toml");
    if default_path.is_file() {
        Ok(Some(default_path))
    } else {
        Ok(None)
    }
}

pub(super) fn resolve_source_paths(
    paths: &[PathBuf],
    context: &ClientContext,
    cache_home: &Path,
) -> Vec<PathBuf> {
    if paths.is_empty() {
        return vec![cache_home.join("agent").join("org")];
    }
    paths
        .iter()
        .map(|path| {
            if path.is_absolute() {
                path.clone()
            } else {
                context.root().join(path)
            }
        })
        .collect()
}

pub(super) fn resolve_config_path_value(
    value: &str,
    project_root: &Path,
    cache_home: &Path,
) -> PathBuf {
    let expanded = expand_project_path_variables(value, project_root, cache_home);
    resolve_path_from_value(Some(project_root), Some(expanded.as_str()))
        .unwrap_or_else(|| project_root.to_path_buf())
}

fn expand_project_path_variables(value: &str, project_root: &Path, cache_home: &Path) -> String {
    let trimmed = value.trim();
    let mut expanded = trimmed.to_string();
    let replacements = [
        ("${PRJ_CACHE_HOME}", cache_home),
        ("$PRJ_CACHE_HOME", cache_home),
        ("${PRJ_ROOT}", project_root),
        ("$PRJ_ROOT", project_root),
    ];
    for (token, path) in replacements {
        let path = path.to_string_lossy();
        if expanded == token {
            expanded = path.into_owned();
        } else if let Some(rest) = expanded.strip_prefix(&format!("{token}/")) {
            expanded = format!("{path}/{rest}");
        }
    }
    expanded
}

fn normalized_optional_string(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
