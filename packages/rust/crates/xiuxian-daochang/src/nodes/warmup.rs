use std::time::Instant;

use anyhow::{Result, anyhow};
use xiuxian_daochang::warmup_options::{WarmupEnvOverrides, resolve_warmup_options};
use xiuxian_daochang::{EmbeddingClient, RuntimeSettings};

use crate::resolve::{parse_positive_u64_from_env, parse_positive_usize_from_env};

pub(crate) async fn run_embedding_warmup(
    runtime_settings: &RuntimeSettings,
    text: String,
    model_override: Option<String>,
) -> Result<()> {
    let env = warmup_env_overrides_from_process_env();
    let options = resolve_warmup_options(runtime_settings, &env, model_override.as_deref());

    println!(
        "Embedding warmup starting: backend='{}' model='{}' timeout_secs={} base_url='{}'",
        options.backend_hint.as_deref().unwrap_or("auto"),
        options.model.as_deref().unwrap_or("<default>"),
        options.timeout_secs,
        options.base_url
    );

    let client = EmbeddingClient::new_with_backend_and_tuning(
        options.base_url.as_str(),
        options.timeout_secs,
        options.backend_hint.as_deref(),
        options.batch_max_size,
        options.batch_max_concurrency,
    );
    let started = Instant::now();
    let maybe_vector = client
        .embed_with_model(text.as_str(), options.model.as_deref())
        .await;
    let elapsed_ms = started.elapsed().as_millis();
    match maybe_vector {
        Some(vector) => {
            println!(
                "Embedding warmup succeeded: dim={} elapsed_ms={elapsed_ms}",
                vector.len()
            );
            Ok(())
        }
        None => Err(anyhow!(
            "embedding warmup failed: backend='{}' model='{}' base_url='{}'",
            options.backend_hint.as_deref().unwrap_or("auto"),
            options.model.as_deref().unwrap_or("<default>"),
            options.base_url
        )),
    }
}

fn warmup_env_overrides_from_process_env() -> WarmupEnvOverrides {
    WarmupEnvOverrides {
        memory_embedding_backend: non_empty_env("XIUXIAN_DAOCHANG_MEMORY_EMBEDDING_BACKEND"),
        embedding_backend: non_empty_env("XIUXIAN_DAOCHANG_EMBED_BACKEND"),
        llm_backend: non_empty_env("XIUXIAN_DAOCHANG_LLM_BACKEND"),
        memory_embedding_model: non_empty_env("XIUXIAN_DAOCHANG_MEMORY_EMBEDDING_MODEL"),
        embedding_model: non_empty_env("XIUXIAN_DAOCHANG_EMBED_MODEL"),
        memory_embedding_base_url: non_empty_env("XIUXIAN_DAOCHANG_MEMORY_EMBEDDING_BASE_URL"),
        embedding_base_url: non_empty_env("XIUXIAN_DAOCHANG_EMBED_BASE_URL"),
        embed_timeout_secs: parse_positive_u64_from_env("XIUXIAN_DAOCHANG_EMBED_TIMEOUT_SECS"),
        memory_embed_batch_max_size: parse_positive_usize_from_env(
            "XIUXIAN_DAOCHANG_MEMORY_EMBED_BATCH_MAX_SIZE",
        ),
        embed_batch_max_size: parse_positive_usize_from_env(
            "XIUXIAN_DAOCHANG_EMBED_BATCH_MAX_SIZE",
        ),
        memory_embed_batch_max_concurrency: parse_positive_usize_from_env(
            "XIUXIAN_DAOCHANG_MEMORY_EMBED_BATCH_MAX_CONCURRENCY",
        ),
        embed_batch_max_concurrency: parse_positive_usize_from_env(
            "XIUXIAN_DAOCHANG_EMBED_BATCH_MAX_CONCURRENCY",
        ),
    }
}

fn non_empty_env(name: &str) -> Option<String> {
    std::env::var(name).ok().and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}
