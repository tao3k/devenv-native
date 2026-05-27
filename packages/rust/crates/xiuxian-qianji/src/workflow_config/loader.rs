//! Loader for Qianji workflow/task route configs.

use super::model::QianjiWorkflowLlmTaskConfig;
use crate::runtime_config::QianjiRuntimeEnv;
use std::env;
use std::io;
use std::path::{Path, PathBuf};
use xiuxian_config_core::{
    ConfigCoreError, load_toml_value_with_imports, resolve_path_from_value,
    resolve_project_root_or_cwd_from_value,
};

/// Default workflow-task profile for BPMN host-work routed to LLM activities.
pub const DEFAULT_BPMN_HOST_WORK_LLM_WORKFLOW_PROFILE: &str = "bpmn-host-work-llm";

/// Resolve the default BPMN host-work LLM workflow-task route config.
///
/// # Errors
///
/// Returns [`io::Error`] when the profile id is invalid or a discovered config
/// file cannot be read or parsed.
pub fn resolve_qianji_workflow_llm_task_config() -> io::Result<QianjiWorkflowLlmTaskConfig> {
    resolve_qianji_workflow_llm_task_config_with_env(
        DEFAULT_BPMN_HOST_WORK_LLM_WORKFLOW_PROFILE,
        &QianjiRuntimeEnv::default(),
    )
}

/// Resolve a named workflow-task LLM route config with explicit runtime env.
///
/// # Errors
///
/// Returns [`io::Error`] when the profile id is invalid or a discovered config
/// file cannot be read or parsed.
pub fn resolve_qianji_workflow_llm_task_config_with_env(
    profile: &str,
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiWorkflowLlmTaskConfig> {
    validate_profile_id(profile)?;
    let project_root = resolve_project_root(runtime_env);
    let config_home = resolve_prj_config_home(runtime_env, &project_root);
    let mut merged = QianjiWorkflowLlmTaskConfig::default();

    for path in workflow_llm_task_config_candidates(profile, &project_root, &config_home) {
        if !path.exists() {
            continue;
        }
        let parsed = read_workflow_llm_task_config(&path)?;
        merged.apply_overlay(parsed);
    }

    Ok(merged)
}

fn workflow_llm_task_config_candidates(
    profile: &str,
    project_root: &Path,
    config_home: &Path,
) -> Vec<PathBuf> {
    vec![
        project_root
            .join("packages/rust/crates/xiuxian-qianji/resources/config/workflows")
            .join(format!("{profile}.toml")),
        config_home
            .join("xiuxian-artisan-workshop/workflows")
            .join(format!("{profile}.toml")),
    ]
}

fn read_workflow_llm_task_config(path: &Path) -> io::Result<QianjiWorkflowLlmTaskConfig> {
    let value = load_toml_value_with_imports(path).map_err(|error| {
        let kind = match &error {
            ConfigCoreError::ReadFile { source, .. } => source.kind(),
            _ => io::ErrorKind::InvalidData,
        };
        io::Error::new(
            kind,
            format!(
                "failed to load qianji workflow task config {}: {error}",
                path.display()
            ),
        )
    })?;
    value
        .try_into::<QianjiWorkflowLlmTaskConfig>()
        .map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "failed to parse qianji workflow task config {}: {error}",
                    path.display()
                ),
            )
        })
}

fn validate_profile_id(profile: &str) -> io::Result<()> {
    let valid = !profile.trim().is_empty()
        && profile
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'));
    if valid {
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("invalid qianji workflow config profile `{profile}`"),
    ))
}

fn resolve_project_root(runtime_env: &QianjiRuntimeEnv) -> PathBuf {
    if let Some(path) = &runtime_env.prj_root {
        return path.clone();
    }
    let raw_project_root = env_var_or_override(runtime_env, "PRJ_ROOT");
    resolve_project_root_or_cwd_from_value(
        raw_project_root.as_deref(),
        env::current_dir().ok().as_deref(),
    )
}

fn resolve_prj_config_home(runtime_env: &QianjiRuntimeEnv, project_root: &Path) -> PathBuf {
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

fn env_var_or_override(runtime_env: &QianjiRuntimeEnv, key: &str) -> Option<String> {
    runtime_env
        .extra_env
        .iter()
        .rev()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value.trim().to_owned())
        .or_else(|| env::var(key).ok())
}
