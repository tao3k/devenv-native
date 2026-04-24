use super::loader::load_qianji_toml;
use super::model::{
    QianjiRuntimeCheckpointConfig, QianjiRuntimeEnv, QianjiRuntimeLlmConfig,
    QianjiRuntimeServerConfig, QianjiRuntimeWendaoIngesterConfig, QianjiRuntimeWorkflowStateConfig,
};
use super::pathing::{resolve_prj_config_home, resolve_project_root};
use std::io;

#[path = "runtime_config/resolve/checkpoint.rs"]
mod checkpoint;
#[path = "runtime_config/resolve/llm.rs"]
mod llm;
#[path = "runtime_config/resolve/server.rs"]
mod server;
#[path = "runtime_config/resolve/wendao.rs"]
mod wendao;
#[path = "runtime_config/resolve/workflow_state.rs"]
mod workflow_state;

/// Resolve `qianji.toml` and environment into an effective LLM runtime config.
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_llm_config() -> io::Result<QianjiRuntimeLlmConfig> {
    resolve_qianji_runtime_llm_config_with_env(&QianjiRuntimeEnv::default())
}

/// Resolve config with explicit runtime environment overrides (for tests and tooling).
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_llm_config_with_env(
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeLlmConfig> {
    let project_root = resolve_project_root(runtime_env);
    let config_home = resolve_prj_config_home(runtime_env, &project_root);
    let file_cfg = load_qianji_toml(runtime_env, &project_root, &config_home)?;
    llm::resolve_qianji_runtime_llm(&file_cfg.llm, runtime_env)
}

/// Resolve `qianji.toml` and environment into native `Wendao` ingestion defaults.
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_wendao_ingester_config()
-> io::Result<QianjiRuntimeWendaoIngesterConfig> {
    resolve_qianji_runtime_wendao_ingester_config_with_env(&QianjiRuntimeEnv::default())
}

/// Resolve `Wendao` ingestion defaults with explicit runtime environment overrides.
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_wendao_ingester_config_with_env(
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeWendaoIngesterConfig> {
    let project_root = resolve_project_root(runtime_env);
    let config_home = resolve_prj_config_home(runtime_env, &project_root);
    let file_cfg = load_qianji_toml(runtime_env, &project_root, &config_home)?;
    Ok(wendao::resolve_qianji_runtime_wendao_ingester(
        &file_cfg.memory_promotion.wendao,
        runtime_env,
    ))
}

/// Resolve `qianji.toml` and environment into checkpoint persistence defaults.
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_checkpoint_config() -> io::Result<QianjiRuntimeCheckpointConfig> {
    resolve_qianji_runtime_checkpoint_config_with_env(&QianjiRuntimeEnv::default())
}

/// Resolve checkpoint persistence defaults with explicit runtime environment overrides.
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_checkpoint_config_with_env(
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeCheckpointConfig> {
    let project_root = resolve_project_root(runtime_env);
    let config_home = resolve_prj_config_home(runtime_env, &project_root);
    let file_cfg = load_qianji_toml(runtime_env, &project_root, &config_home)?;
    Ok(checkpoint::resolve_qianji_runtime_checkpoint(
        &file_cfg.checkpoint,
        runtime_env,
    ))
}

/// Resolve `qianji.toml` and environment into local workflow-state defaults.
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_workflow_state_config() -> io::Result<QianjiRuntimeWorkflowStateConfig>
{
    resolve_qianji_runtime_workflow_state_config_with_env(&QianjiRuntimeEnv::default())
}

/// Resolve local workflow-state defaults with explicit runtime environment overrides.
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_workflow_state_config_with_env(
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeWorkflowStateConfig> {
    let project_root = resolve_project_root(runtime_env);
    let config_home = resolve_prj_config_home(runtime_env, &project_root);
    let file_cfg = load_qianji_toml(runtime_env, &project_root, &config_home)?;
    Ok(workflow_state::resolve_qianji_runtime_workflow_state(
        &file_cfg.workflow_state,
        runtime_env,
        &project_root,
    ))
}

/// Resolve `qianji.toml` and environment into `qianji-server` defaults.
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_server_config() -> io::Result<QianjiRuntimeServerConfig> {
    resolve_qianji_runtime_server_config_with_env(&QianjiRuntimeEnv::default())
}

/// Resolve `qianji-server` defaults with explicit runtime environment overrides.
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_server_config_with_env(
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeServerConfig> {
    let project_root = resolve_project_root(runtime_env);
    let config_home = resolve_prj_config_home(runtime_env, &project_root);
    let file_cfg = load_qianji_toml(runtime_env, &project_root, &config_home)?;
    Ok(server::resolve_qianji_runtime_server(
        &file_cfg.server,
        runtime_env,
    ))
}
