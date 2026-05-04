//! Runtime agent factory memory branch for store and embedding setup.

use super::{env_overrides, runtime};
use crate::runtime_agent_factory::{
    MemoryRuntimeOptions, parse_embedding_backend_mode, resolve_runtime_embedding_backend_mode,
    resolve_runtime_embedding_base_url,
};
use crate::{MemoryConfig, RuntimeSettings};

pub(crate) fn resolve_runtime_memory_options(
    runtime_settings: &RuntimeSettings,
) -> MemoryRuntimeOptions {
    let mut memory = MemoryConfig::default();
    runtime::apply_memory_runtime_settings(&mut memory, runtime_settings);

    let embedding_backend_mode = parse_embedding_backend_mode(memory.embedding_backend.as_deref())
        .unwrap_or_else(|| resolve_runtime_embedding_backend_mode(runtime_settings));
    if let Some(base_url) =
        resolve_runtime_embedding_base_url(runtime_settings, embedding_backend_mode)
    {
        memory.embedding_base_url = Some(base_url);
    }

    env_overrides::apply_memory_env_overrides(&mut memory);

    MemoryRuntimeOptions {
        config: memory,
        embedding_backend_mode,
    }
}
