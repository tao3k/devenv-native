use crate::runtime_config::constants::{DEFAULT_API_KEY_ENV, DEFAULT_BASE_URL, DEFAULT_MODEL};
use crate::runtime_config::env_vars::env_var_or_override;
use crate::runtime_config::model::{QianjiRuntimeEnv, QianjiRuntimeLlmConfig};
use crate::runtime_config::toml_config::ProviderConfig;
use crate::runtime_config::toml_config::QianjiTomlLlm;
use std::io;
use xiuxian_macros::string_first_non_empty;

pub(super) fn resolve_qianji_runtime_llm(
    file_llm: &QianjiTomlLlm,
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeLlmConfig> {
    resolve_qianji_runtime_llm_from_local_config(file_llm, runtime_env)
}

fn selected_provider_config<'a>(
    file_llm: &'a QianjiTomlLlm,
    runtime_env: &QianjiRuntimeEnv,
) -> Option<&'a ProviderConfig> {
    let provider_name = runtime_env
        .qianji_llm_provider
        .clone()
        .or_else(|| env_var_or_override(runtime_env, "QIANJI_LLM_PROVIDER"))
        .or_else(|| file_llm.default_provider.clone())
        .unwrap_or_else(|| "openai".to_string());

    file_llm.providers.as_ref().and_then(|providers| {
        providers
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(provider_name.as_str()))
            .map(|(_, config)| config)
    })
}

fn parse_env_reference(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix("env:")
        && is_env_key_token(rest)
    {
        return Some(rest);
    }
    if trimmed.starts_with("${")
        && trimmed.ends_with('}')
        && trimmed.len() > 3
        && is_env_key_token(&trimmed[2..trimmed.len() - 1])
    {
        return Some(&trimmed[2..trimmed.len() - 1]);
    }
    None
}

fn is_env_key_token(raw: &str) -> bool {
    let trimmed = raw.trim();
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_uppercase()) {
        return false;
    }
    chars.all(|ch| ch == '_' || ch.is_ascii_uppercase() || ch.is_ascii_digit())
}

fn resolve_qianji_runtime_llm_from_local_config(
    file_llm: &QianjiTomlLlm,
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeLlmConfig> {
    let provider_cfg = selected_provider_config(file_llm, runtime_env);
    let model = string_first_non_empty!(
        runtime_env.qianji_llm_model.as_deref(),
        env_var_or_override(runtime_env, "QIANJI_LLM_MODEL").as_deref(),
        provider_cfg.and_then(|cfg| cfg.model.as_deref()),
        file_llm.model.as_deref(),
        file_llm.default_model.as_deref(),
        Some(DEFAULT_MODEL),
    );
    let base_url = string_first_non_empty!(
        runtime_env.openai_api_base.as_deref(),
        env_var_or_override(runtime_env, "OPENAI_API_BASE").as_deref(),
        provider_cfg.and_then(|cfg| cfg.base_url.as_deref()),
        file_llm.base_url.as_deref(),
        Some(DEFAULT_BASE_URL),
    );
    let key_selector = string_first_non_empty!(
        provider_cfg.and_then(|cfg| cfg.api_key_env.as_deref()),
        provider_cfg.and_then(|cfg| cfg.api_key.as_deref()),
        file_llm.api_key_env.as_deref(),
        file_llm.api_key.as_deref(),
        Some(DEFAULT_API_KEY_ENV),
    );
    let api_key_env = parse_env_reference(key_selector.as_str())
        .map(ToString::to_string)
        .or_else(|| {
            if is_env_key_token(key_selector.as_str()) {
                Some(key_selector.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| DEFAULT_API_KEY_ENV.to_string());
    let maybe_api_key = string_first_non_empty!(
        runtime_env.openai_api_key.as_deref(),
        env_var_or_override(runtime_env, "OPENAI_API_KEY").as_deref(),
        env_var_or_override(runtime_env, api_key_env.as_str()).as_deref(),
        if api_key_env == key_selector {
            None
        } else {
            Some(key_selector.as_str())
        },
    );
    if maybe_api_key.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "missing Qianji API key; set OPENAI_API_KEY or {api_key_env} (resolved from qianji.toml)"
            ),
        ));
    }
    Ok(QianjiRuntimeLlmConfig {
        model,
        base_url,
        api_key_env,
        wire_api: string_first_non_empty!(
            provider_cfg.and_then(|cfg| cfg.wire_api.as_deref()),
            file_llm.wire_api.as_deref(),
            Some("chat_completions"),
        ),
        api_key: maybe_api_key,
    })
}
