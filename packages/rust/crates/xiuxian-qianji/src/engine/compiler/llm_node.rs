use crate::contracts::{NodeDefinition, NodeLlmBinding};
use crate::error::QianjiError;
use crate::runtime_config::resolve_qianji_runtime_llm_config;
use xiuxian_llm::llm::{LlmBackendKind, parse_llm_backend_kind};

pub(super) struct LlmMechanismConfig {
    pub(super) model: String,
    pub(super) context_keys: Vec<String>,
    pub(super) prompt_template: String,
    pub(super) output_key: String,
    pub(super) parse_json_output: bool,
    pub(super) fallback_repo_tree_on_parse_failure: bool,
}

pub(super) fn mechanism_config(node_def: &NodeDefinition) -> LlmMechanismConfig {
    LlmMechanismConfig {
        model: model(node_def),
        context_keys: string_list_param(node_def, "context_keys"),
        prompt_template: string_param(node_def, "prompt").unwrap_or_default(),
        output_key: node_def
            .params
            .get("output_key")
            .and_then(|value| value.as_str())
            .unwrap_or("analysis_conclusion")
            .to_string(),
        parse_json_output: node_def
            .params
            .get("parse_json_output")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        fallback_repo_tree_on_parse_failure: node_def
            .params
            .get("fallback_repo_tree_on_parse_failure")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
    }
}

fn string_param(node_def: &NodeDefinition, key: &str) -> Option<String> {
    node_def
        .params
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn string_list_param(node_def: &NodeDefinition, key: &str) -> Vec<String> {
    node_def
        .params
        .get(key)
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn resolve_node_llm_endpoint(
    binding: Option<&NodeLlmBinding>,
) -> Result<Option<(String, String)>, QianjiError> {
    if !has_dedicated_llm_endpoint(binding) {
        return Ok(None);
    }

    let provider = provider_kind(binding)?;
    if provider == LlmBackendKind::LiteLlmRs {
        return Err(QianjiError::Topology(
            "Node-level provider 'litellm_rs' is not yet supported in xiuxian-qianji; use openai-compatible endpoints for now.".to_string(),
        ));
    }

    let runtime = resolve_qianji_runtime_llm_config().ok();
    let binding_base_url = binding.and_then(|config| non_empty(config.base_url.as_deref()));
    let base_url = binding_base_url
        .or_else(|| runtime.as_ref().map(|cfg| cfg.base_url.clone()))
        .ok_or_else(|| {
            QianjiError::Topology(
                "Node-level LLM endpoint requires `base_url` in [nodes.llm] or global qianji runtime config.".to_string(),
            )
        })?;

    let binding_api_key_env = binding.and_then(|config| non_empty(config.api_key_env.as_deref()));
    let api_key_env = binding_api_key_env
        .or_else(|| runtime.as_ref().map(|cfg| cfg.api_key_env.clone()))
        .unwrap_or_else(|| "OPENAI_API_KEY".to_string());

    let api_key = std::env::var(&api_key_env)
        .ok()
        .and_then(|value| non_empty(Some(value.as_str())))
        .or_else(|| {
            std::env::var("OPENAI_API_KEY")
                .ok()
                .and_then(|value| non_empty(Some(value.as_str())))
        })
        .or_else(|| runtime.map(|cfg| cfg.api_key))
        .ok_or_else(|| {
            QianjiError::Topology(format!(
                "Missing API key for node-level LLM endpoint; set {api_key_env} or OPENAI_API_KEY."
            ))
        })?;

    Ok(Some((base_url, api_key)))
}

fn model(node_def: &NodeDefinition) -> String {
    non_empty(
        node_def
            .llm
            .as_ref()
            .and_then(|binding| binding.model.as_deref()),
    )
    .or_else(|| {
        non_empty(
            node_def
                .params
                .get("model")
                .and_then(serde_json::Value::as_str),
        )
    })
    .unwrap_or_default()
}

fn provider_kind(binding: Option<&NodeLlmBinding>) -> Result<LlmBackendKind, QianjiError> {
    let raw = binding.and_then(|config| config.provider.as_deref());
    if let Some(provider) = raw {
        return parse_llm_backend_kind(Some(provider)).ok_or_else(|| {
            QianjiError::Topology(format!("Unsupported LLM provider for node: {provider}"))
        });
    }
    Ok(LlmBackendKind::OpenAiCompatibleHttp)
}

fn has_dedicated_llm_endpoint(binding: Option<&NodeLlmBinding>) -> bool {
    binding.is_some_and(|config| {
        non_empty(config.base_url.as_deref()).is_some()
            || non_empty(config.api_key_env.as_deref()).is_some()
    })
}

fn non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|raw| !raw.is_empty())
        .map(ToString::to_string)
}
