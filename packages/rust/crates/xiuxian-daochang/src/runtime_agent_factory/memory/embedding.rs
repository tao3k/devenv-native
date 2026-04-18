use crate::env_parse::{parse_positive_u64_from_env, parse_positive_usize_from_env};
use crate::runtime_agent_factory::shared::non_empty_env;
use crate::{MemoryConfig, RuntimeSettings};

pub(super) fn apply_memory_runtime_embedding_settings(
    memory: &mut MemoryConfig,
    runtime_settings: &RuntimeSettings,
) {
    apply_memory_runtime_path(memory, runtime_settings);
    apply_memory_runtime_embedding_backend(memory, runtime_settings);
    apply_memory_runtime_embedding_batch_settings(memory, runtime_settings);
    apply_memory_runtime_embedding_timeout_settings(memory, runtime_settings);
    apply_memory_runtime_embedding_model(memory, runtime_settings);
    apply_memory_runtime_embedding_dimension(memory, runtime_settings);
}

fn apply_memory_runtime_path(memory: &mut MemoryConfig, runtime_settings: &RuntimeSettings) {
    if let Some(path) = runtime_settings
        .memory
        .path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        memory.path = path.to_string();
    }
}

fn resolve_runtime_memory_embedding_backend(runtime_settings: &RuntimeSettings) -> Option<String> {
    non_empty_env("XIUXIAN_DAOCHANG_MEMORY_EMBEDDING_BACKEND")
        .or_else(|| {
            runtime_settings
                .memory
                .embedding_backend
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
        })
        .or_else(|| non_empty_env("XIUXIAN_DAOCHANG_EMBED_BACKEND"))
        .or_else(|| {
            runtime_settings
                .embedding
                .backend
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
        })
}

fn apply_memory_runtime_embedding_backend(
    memory: &mut MemoryConfig,
    runtime_settings: &RuntimeSettings,
) {
    if let Some(backend) = resolve_runtime_memory_embedding_backend(runtime_settings) {
        memory.embedding_backend = Some(backend);
    }
}

fn resolve_runtime_memory_batch_max_size(runtime_settings: &RuntimeSettings) -> Option<usize> {
    parse_positive_usize_from_env("XIUXIAN_DAOCHANG_MEMORY_EMBED_BATCH_MAX_SIZE")
        .or_else(|| parse_positive_usize_from_env("XIUXIAN_DAOCHANG_EMBED_BATCH_MAX_SIZE"))
        .or(runtime_settings
            .embedding
            .batch_max_size
            .filter(|value| *value > 0))
}

fn resolve_runtime_memory_batch_max_concurrency(
    runtime_settings: &RuntimeSettings,
) -> Option<usize> {
    parse_positive_usize_from_env("XIUXIAN_DAOCHANG_MEMORY_EMBED_BATCH_MAX_CONCURRENCY")
        .or_else(|| parse_positive_usize_from_env("XIUXIAN_DAOCHANG_EMBED_BATCH_MAX_CONCURRENCY"))
        .or(runtime_settings
            .embedding
            .batch_max_concurrency
            .filter(|value| *value > 0))
}

fn apply_memory_runtime_embedding_batch_settings(
    memory: &mut MemoryConfig,
    runtime_settings: &RuntimeSettings,
) {
    if let Some(batch_max_size) = resolve_runtime_memory_batch_max_size(runtime_settings) {
        memory.embedding_batch_max_size = Some(batch_max_size);
    }
    if let Some(batch_max_concurrency) =
        resolve_runtime_memory_batch_max_concurrency(runtime_settings)
    {
        memory.embedding_batch_max_concurrency = Some(batch_max_concurrency);
    }
}

fn resolve_runtime_memory_embedding_timeout_ms(runtime_settings: &RuntimeSettings) -> Option<u64> {
    runtime_settings
        .memory
        .embedding_timeout_ms
        .filter(|value| *value > 0)
        .or_else(|| {
            runtime_settings
                .embedding
                .timeout_secs
                .filter(|value| *value > 0)
                .and_then(|secs| secs.checked_mul(1_000))
        })
}

fn resolve_runtime_memory_embedding_timeout_cooldown_ms(
    runtime_settings: &RuntimeSettings,
) -> Option<u64> {
    runtime_settings
        .memory
        .embedding_timeout_cooldown_ms
        .filter(|value| *value > 0)
}

fn apply_memory_runtime_embedding_timeout_settings(
    memory: &mut MemoryConfig,
    runtime_settings: &RuntimeSettings,
) {
    if let Some(timeout_ms) = resolve_runtime_memory_embedding_timeout_ms(runtime_settings) {
        memory.embedding_timeout_ms = Some(timeout_ms);
    }
    if let Some(cooldown_ms) =
        resolve_runtime_memory_embedding_timeout_cooldown_ms(runtime_settings)
    {
        memory.embedding_timeout_cooldown_ms = Some(cooldown_ms);
    }
}

fn resolve_runtime_memory_embedding_model(runtime_settings: &RuntimeSettings) -> Option<String> {
    runtime_settings
        .memory
        .embedding_model
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| {
            runtime_settings
                .embedding
                .litellm_model
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .or_else(|| {
            runtime_settings
                .embedding
                .model
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
        })
        .map(ToString::to_string)
}

fn apply_memory_runtime_embedding_model(
    memory: &mut MemoryConfig,
    runtime_settings: &RuntimeSettings,
) {
    if let Some(model) = resolve_runtime_memory_embedding_model(runtime_settings) {
        memory.embedding_model = Some(model);
    }
}

fn resolve_runtime_memory_embedding_dimension(runtime_settings: &RuntimeSettings) -> Option<usize> {
    runtime_settings
        .memory
        .embedding_dim
        .filter(|value| *value > 0)
        .or(runtime_settings
            .embedding
            .dimension
            .filter(|value| *value > 0))
}

fn apply_memory_runtime_embedding_dimension(
    memory: &mut MemoryConfig,
    runtime_settings: &RuntimeSettings,
) {
    if let Some(embedding_dim) = resolve_runtime_memory_embedding_dimension(runtime_settings) {
        memory.embedding_dim = embedding_dim;
    }
}

pub(super) fn apply_memory_env_embedding_overrides(memory: &mut MemoryConfig) {
    if let Some(path) = non_empty_env("XIUXIAN_DAOCHANG_MEMORY_PATH") {
        memory.path = path;
    }
    if let Some(model) = non_empty_env("XIUXIAN_DAOCHANG_MEMORY_EMBEDDING_MODEL")
        .or_else(|| non_empty_env("XIUXIAN_DAOCHANG_EMBED_MODEL"))
    {
        memory.embedding_model = Some(model);
    }
    if let Some(base_url) = non_empty_env("XIUXIAN_DAOCHANG_MEMORY_EMBEDDING_BASE_URL")
        .or_else(|| non_empty_env("XIUXIAN_DAOCHANG_EMBED_BASE_URL"))
    {
        memory.embedding_base_url = Some(base_url);
    }
    if let Some(embedding_dim) =
        parse_positive_usize_from_env("XIUXIAN_DAOCHANG_MEMORY_EMBEDDING_DIM")
    {
        memory.embedding_dim = embedding_dim;
    }
    if let Some(timeout_ms) =
        parse_positive_u64_from_env("XIUXIAN_DAOCHANG_MEMORY_EMBED_TIMEOUT_MS")
            .or_else(|| parse_positive_u64_from_env("XIUXIAN_DAOCHANG_EMBED_TIMEOUT_MS"))
            .or_else(|| {
                parse_positive_u64_from_env("XIUXIAN_DAOCHANG_EMBED_TIMEOUT_SECS")
                    .and_then(|secs| secs.checked_mul(1_000))
            })
    {
        memory.embedding_timeout_ms = Some(timeout_ms);
    }
    if let Some(cooldown_ms) =
        parse_positive_u64_from_env("XIUXIAN_DAOCHANG_MEMORY_EMBED_TIMEOUT_COOLDOWN_MS")
            .or_else(|| parse_positive_u64_from_env("XIUXIAN_DAOCHANG_EMBED_TIMEOUT_COOLDOWN_MS"))
    {
        memory.embedding_timeout_cooldown_ms = Some(cooldown_ms);
    }
}
