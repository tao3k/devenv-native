//! Runtime configuration loader for `qianji.toml`.
//!
//! Start in `api`; resolver entry points stay there while loader/pathing stay private.
//!
//! Resolution order:
//! 1. System config: `<PRJ_ROOT>/packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml`
//! 2. User config: `<PRJ_CONFIG_HOME>/xiuxian-artisan-workshop/qianji.toml`
//! 3. Explicit config path: `$QIANJI_CONFIG_PATH`
//! 4. Environment overrides:
//!    - `QIANJI_LLM_MODEL`
//!    - `OPENAI_API_BASE`
//!    - `OPENAI_API_KEY`
//!    - `QIANJI_VALKEY_URL`
//!    - `VALKEY_URL`
//!    - `REDIS_URL`
//!    - `QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE`
//!    - `QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE_KEY`
//!    - `QIANJI_MEMORY_PROMOTION_GRAPH_DIMENSION`
//!    - `QIANJI_MEMORY_PROMOTION_PERSIST`
//!    - `QIANJI_MEMORY_PROMOTION_PERSIST_BEST_EFFORT`

#[path = "runtime_config_api.rs"]
mod api;
mod constants;
mod env_vars;
mod loader;
#[path = "runtime_config_model.rs"]
mod model;
mod pathing;
#[path = "runtime_config_resolve.rs"]
mod resolve;
mod toml_config;

pub use api::{
    QianjiRuntimeCheckpointConfig, QianjiRuntimeEnv, QianjiRuntimeLlmConfig,
    QianjiRuntimeServerConfig, QianjiRuntimeWendaoIngesterConfig,
    resolve_qianji_runtime_checkpoint_config, resolve_qianji_runtime_checkpoint_config_with_env,
    resolve_qianji_runtime_llm_config, resolve_qianji_runtime_llm_config_with_env,
    resolve_qianji_runtime_server_config, resolve_qianji_runtime_server_config_with_env,
    resolve_qianji_runtime_wendao_ingester_config,
    resolve_qianji_runtime_wendao_ingester_config_with_env,
};
pub(crate) use api::{
    resolve_process_env_path, resolve_process_project_root, resolve_process_project_root_from_cwd,
};
