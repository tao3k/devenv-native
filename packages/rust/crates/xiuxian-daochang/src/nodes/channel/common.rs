use xiuxian_daochang::RuntimeSettings;
use xiuxian_macros::env_non_empty;

fn trim_non_empty(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

pub(super) fn apply_channel_embedding_memory_guard(
    runtime_settings: &RuntimeSettings,
) -> RuntimeSettings {
    apply_channel_embedding_memory_guard_for_tests(
        runtime_settings,
        env_non_empty!("XIUXIAN_DAOCHANG_MEMORY_EMBEDDING_BACKEND").as_deref(),
        env_non_empty!("XIUXIAN_DAOCHANG_EMBED_BACKEND").as_deref(),
        false,
    )
}

pub(super) fn apply_channel_embedding_memory_guard_for_tests(
    runtime_settings: &RuntimeSettings,
    env_memory_backend: Option<&str>,
    env_embed_backend: Option<&str>,
    allow_inproc_embed: bool,
) -> RuntimeSettings {
    let mut guarded = runtime_settings.clone();

    if let Some(backend) = trim_non_empty(env_memory_backend) {
        guarded.memory.embedding_backend = Some(backend);
        return guarded;
    }

    if let Some(backend) = trim_non_empty(env_embed_backend) {
        guarded.memory.embedding_backend = Some(backend);
        return guarded;
    }

    if allow_inproc_embed {
        return guarded;
    }

    guarded
}

pub(super) fn log_control_command_allow_override(provider: &str, entries: Option<&[String]>) {
    if let Some(entries) = entries {
        if entries.is_empty() {
            tracing::warn!(
                provider = %provider,
                "{provider}.control_command_allow_from is configured but empty; privileged control commands are denied for all senders"
            );
        } else {
            tracing::info!(
                provider = %provider,
                entries = entries.len(),
                "{provider}.control_command_allow_from override is active"
            );
        }
    }
}

pub(super) fn log_slash_command_allow_override(provider: &str, entries: Option<&[String]>) {
    if let Some(entries) = entries {
        if entries.is_empty() {
            tracing::warn!(
                provider = %provider,
                "{provider}.slash_command_allow_from is configured but empty; managed slash commands are denied for all non-admin senders"
            );
        } else {
            tracing::info!(
                provider = %provider,
                entries = entries.len(),
                "{provider}.slash_command_allow_from override is active"
            );
        }
    }
}
