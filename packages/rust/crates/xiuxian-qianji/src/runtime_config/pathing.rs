use super::constants::DEFAULT_QIANJI_DATA_NAMESPACE;
use super::env_vars::env_var_or_override;
use super::model::QianjiRuntimeEnv;
use std::env;
use std::path::{Path, PathBuf};
use xiuxian_config_core::{resolve_path_from_value, resolve_project_root_or_cwd_from_value};

pub(super) fn resolve_project_root(runtime_env: &QianjiRuntimeEnv) -> PathBuf {
    if let Some(path) = &runtime_env.prj_root {
        return path.clone();
    }
    let raw_project_root = env_var_or_override(runtime_env, "PRJ_ROOT");
    resolve_project_root_from_value(
        raw_project_root.as_deref(),
        env::current_dir().ok().as_deref(),
    )
}

pub(super) fn resolve_prj_config_home(
    runtime_env: &QianjiRuntimeEnv,
    project_root: &Path,
) -> PathBuf {
    if let Some(path) = &runtime_env.prj_config_home {
        return path.clone();
    }

    if let Some(path) = resolve_path_from_value(
        Some(project_root),
        env_var_or_override(runtime_env, "PRJ_CONFIG_HOME").as_deref(),
    ) {
        return path;
    }

    project_root.join(".config")
}

pub(super) fn resolve_prj_data_home(
    runtime_env: &QianjiRuntimeEnv,
    project_root: &Path,
) -> PathBuf {
    if let Some(path) = &runtime_env.prj_data_home {
        return path.clone();
    }

    if let Some(path) = resolve_path_from_value(
        Some(project_root),
        env_var_or_override(runtime_env, "PRJ_DATA_HOME").as_deref(),
    ) {
        return path;
    }

    project_root.join(".data")
}

pub(super) fn resolve_qianji_data_root(
    runtime_env: &QianjiRuntimeEnv,
    project_root: &Path,
) -> PathBuf {
    resolve_prj_data_home(runtime_env, project_root).join(DEFAULT_QIANJI_DATA_NAMESPACE)
}

pub(crate) fn resolve_project_root_from_value(
    raw_project_root: Option<&str>,
    current_dir: Option<&Path>,
) -> PathBuf {
    resolve_project_root_or_cwd_from_value(raw_project_root, current_dir)
}

#[cfg(feature = "qianji-full")]
pub(crate) fn resolve_process_project_root() -> Option<PathBuf> {
    let current_dir = env::current_dir().ok();
    let raw_project_root = env::var("PRJ_ROOT").ok();
    if raw_project_root.is_none() && current_dir.is_none() {
        return None;
    }
    Some(resolve_project_root_from_value(
        raw_project_root.as_deref(),
        current_dir.as_deref(),
    ))
}

#[cfg(feature = "qianji-full")]
pub(crate) fn resolve_process_project_root_from_cwd(current_dir: &Path) -> PathBuf {
    let raw_project_root = env::var("PRJ_ROOT").ok();
    resolve_project_root_from_value(raw_project_root.as_deref(), Some(current_dir))
}

#[cfg(feature = "qianji-full")]
pub(crate) fn resolve_process_env_path(key: &str, project_root: &Path) -> Option<PathBuf> {
    let raw_value = env::var(key).ok();
    resolve_path_from_value(Some(project_root), raw_value.as_deref())
}

#[cfg(test)]
#[path = "../../tests/unit/runtime_config/pathing.rs"]
mod tests;
