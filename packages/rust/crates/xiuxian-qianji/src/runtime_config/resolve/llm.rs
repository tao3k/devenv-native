use crate::runtime_config::constants::{DEFAULT_API_KEY_ENV, DEFAULT_BASE_URL, DEFAULT_MODEL};
use crate::runtime_config::env_vars::env_var_or_override;
use crate::runtime_config::model::{QianjiRuntimeEnv, QianjiRuntimeLlmConfig};
#[cfg(not(feature = "llm"))]
use crate::runtime_config::toml_config::ProviderConfig;
use crate::runtime_config::toml_config::QianjiTomlLlm;
use std::io;

#[cfg(feature = "llm")]
use std::collections::HashMap;
#[cfg(feature = "llm")]
use xiuxian_llm::llm::{
    LlmProviderProfileInput, LlmRuntimeDefaults, LlmRuntimeProfileEnv, LlmRuntimeProfileInput,
    OpenAIWireApi, resolve_openai_runtime_profile,
};
#[cfg(not(feature = "llm"))]
use xiuxian_macros::string_first_non_empty;

pub(super) fn resolve_qianji_runtime_llm(
    file_llm: &QianjiTomlLlm,
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeLlmConfig> {
    #[cfg(feature = "llm")]
    {
        resolve_qianji_runtime_llm_with_llm_feature(file_llm, runtime_env)
    }

    #[cfg(not(feature = "llm"))]
    {
        resolve_qianji_runtime_llm_without_llm_feature(file_llm, runtime_env)
    }
}

#[cfg(not(feature = "llm"))]
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

#[cfg(not(feature = "llm"))]
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

#[cfg(not(feature = "llm"))]
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

#[cfg(feature = "llm")]
fn resolve_qianji_runtime_llm_with_llm_feature(
    file_llm: &QianjiTomlLlm,
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeLlmConfig> {
    let providers = file_llm
        .providers
        .as_ref()
        .map_or_else(HashMap::new, |providers| {
            providers
                .iter()
                .map(|(name, config)| {
                    (
                        name.clone(),
                        LlmProviderProfileInput {
                            model: config.model.clone(),
                            base_url: config.base_url.clone(),
                            api_key: config.api_key.clone(),
                            api_key_env: config.api_key_env.clone(),
                            wire_api: config.wire_api.clone(),
                        },
                    )
                })
                .collect::<HashMap<_, _>>()
        });
    let profile_input = LlmRuntimeProfileInput {
        model: file_llm.model.clone(),
        default_model: file_llm.default_model.clone(),
        base_url: file_llm.base_url.clone(),
        api_key_env: file_llm.api_key_env.clone(),
        api_key: file_llm.api_key.clone(),
        wire_api: file_llm.wire_api.clone(),
        default_provider: file_llm.default_provider.clone(),
        providers,
    };
    let profile_env = LlmRuntimeProfileEnv {
        provider_override: runtime_env
            .qianji_llm_provider
            .clone()
            .or_else(|| env_var_or_override(runtime_env, "QIANJI_LLM_PROVIDER")),
        model_override: runtime_env
            .qianji_llm_model
            .clone()
            .or_else(|| env_var_or_override(runtime_env, "QIANJI_LLM_MODEL")),
        base_url_override: runtime_env
            .openai_api_base
            .clone()
            .or_else(|| env_var_or_override(runtime_env, "OPENAI_API_BASE")),
        api_key_override: runtime_env
            .openai_api_key
            .clone()
            .or_else(|| env_var_or_override(runtime_env, "OPENAI_API_KEY")),
        wire_api_override: runtime_env
            .qianji_llm_wire_api
            .clone()
            .or_else(|| env_var_or_override(runtime_env, "QIANJI_LLM_WIRE_API")),
        env_vars: runtime_env.extra_env.clone(),
    };
    let defaults = LlmRuntimeDefaults {
        provider: "openai".to_string(),
        model: DEFAULT_MODEL.to_string(),
        base_url: DEFAULT_BASE_URL.to_string(),
        api_key_env: DEFAULT_API_KEY_ENV.to_string(),
        wire_api: OpenAIWireApi::ChatCompletions,
    };
    let resolved = resolve_openai_runtime_profile(&profile_input, &profile_env, &defaults)
        .map_err(|error| {
            let message =
                format!("failed to resolve qianji runtime llm profile from xiuxian.toml: {error}");
            let kind = if message.contains("API key") {
                io::ErrorKind::NotFound
            } else {
                io::ErrorKind::InvalidData
            };
            io::Error::new(kind, message)
        })?;
    let api_key = runtime_env
        .openai_api_key
        .clone()
        .or_else(|| env_var_or_override(runtime_env, "OPENAI_API_KEY"))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(resolved.api_key);
    Ok(QianjiRuntimeLlmConfig {
        model: resolved.model,
        base_url: resolved.base_url,
        api_key_env: resolved.api_key_env,
        wire_api: resolved.wire_api.as_str().to_string(),
        api_key,
    })
}

#[cfg(not(feature = "llm"))]
fn resolve_qianji_runtime_llm_without_llm_feature(
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
