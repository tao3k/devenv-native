//! Unified configuration model aligned with Sovereign xiuxian.toml structure.

use super::env_vars::normalize_non_empty;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Default, Deserialize)]
pub(super) struct QianjiToml {
    #[serde(default)]
    pub(super) llm: QianjiTomlLlm,
    #[serde(default)]
    pub(super) memory_promotion: QianjiTomlMemoryPromotion,
    #[serde(default)]
    pub(super) checkpoint: QianjiTomlCheckpoint,
    #[serde(default)]
    pub(super) server: QianjiTomlServer,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct QianjiTomlLlm {
    /// Support for legacy flat structure
    pub(super) model: Option<String>,
    pub(super) default_model: Option<String>,
    pub(super) base_url: Option<String>,
    pub(super) api_key: Option<String>,
    pub(super) api_key_env: Option<String>,
    pub(super) wire_api: Option<String>,

    /// Support for Sovereign multi-provider structure
    pub(super) default_provider: Option<String>,
    pub(super) providers: Option<HashMap<String, ProviderConfig>>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub(super) struct ProviderConfig {
    pub(super) model: Option<String>,
    pub(super) base_url: Option<String>,
    pub(super) api_key: Option<String>,
    pub(super) api_key_env: Option<String>,
    pub(super) wire_api: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct QianjiTomlMemoryPromotion {
    #[serde(default)]
    pub(super) wendao: QianjiTomlWendaoIngester,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct QianjiTomlWendaoIngester {
    pub(super) graph_scope: Option<String>,
    pub(super) graph_scope_key: Option<String>,
    pub(super) graph_dimension: Option<usize>,
    pub(super) persist: Option<bool>,
    pub(super) persist_best_effort: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct QianjiTomlCheckpoint {
    pub(super) valkey_url: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct QianjiTomlServer {
    pub(super) bind_addr: Option<String>,
    pub(super) require_valkey_ready: Option<bool>,
}

pub(super) fn apply_llm_overlay(target: &mut QianjiTomlLlm, overlay: QianjiTomlLlm) {
    if let Some(default_provider) = normalize_non_empty(overlay.default_provider) {
        target.default_provider = Some(default_provider);
    }

    if let Some(model) = normalize_non_empty(overlay.model) {
        target.model = Some(model);
    }
    if let Some(default_model) = normalize_non_empty(overlay.default_model) {
        target.default_model = Some(default_model);
    }
    if let Some(base_url) = normalize_non_empty(overlay.base_url) {
        target.base_url = Some(base_url);
    }
    if let Some(api_key) = normalize_non_empty(overlay.api_key) {
        target.api_key = Some(api_key);
    }
    if let Some(api_key_env) = normalize_non_empty(overlay.api_key_env) {
        target.api_key_env = Some(api_key_env);
    }
    if let Some(wire_api) = normalize_non_empty(overlay.wire_api) {
        target.wire_api = Some(wire_api);
    }

    if let Some(providers) = overlay.providers {
        let merged = target.providers.get_or_insert_with(HashMap::new);
        for (name, provider_overlay) in providers {
            let normalized_name = name.trim().to_string();
            if normalized_name.is_empty() {
                continue;
            }
            let entry = merged.entry(normalized_name).or_default();
            if let Some(model) = normalize_non_empty(provider_overlay.model) {
                entry.model = Some(model);
            }
            if let Some(base_url) = normalize_non_empty(provider_overlay.base_url) {
                entry.base_url = Some(base_url);
            }
            if let Some(api_key) = normalize_non_empty(provider_overlay.api_key) {
                entry.api_key = Some(api_key);
            }
            if let Some(api_key_env) = normalize_non_empty(provider_overlay.api_key_env) {
                entry.api_key_env = Some(api_key_env);
            }
            if let Some(wire_api) = normalize_non_empty(provider_overlay.wire_api) {
                entry.wire_api = Some(wire_api);
            }
        }
    }
}

pub(super) fn apply_memory_promotion_overlay(
    target: &mut QianjiTomlMemoryPromotion,
    overlay: QianjiTomlMemoryPromotion,
) {
    if let Some(graph_scope) = normalize_non_empty(overlay.wendao.graph_scope) {
        target.wendao.graph_scope = Some(graph_scope);
    }
    if let Some(graph_scope_key) = normalize_non_empty(overlay.wendao.graph_scope_key) {
        target.wendao.graph_scope_key = Some(graph_scope_key);
    }
    if let Some(graph_dimension) = overlay.wendao.graph_dimension {
        target.wendao.graph_dimension = Some(graph_dimension);
    }
    if let Some(persist) = overlay.wendao.persist {
        target.wendao.persist = Some(persist);
    }
    if let Some(persist_best_effort) = overlay.wendao.persist_best_effort {
        target.wendao.persist_best_effort = Some(persist_best_effort);
    }
}

pub(super) fn apply_checkpoint_overlay(
    target: &mut QianjiTomlCheckpoint,
    overlay: QianjiTomlCheckpoint,
) {
    if let Some(valkey_url) = normalize_non_empty(overlay.valkey_url) {
        target.valkey_url = Some(valkey_url);
    }
}

pub(super) fn apply_server_overlay(target: &mut QianjiTomlServer, overlay: QianjiTomlServer) {
    if let Some(bind_addr) = normalize_non_empty(overlay.bind_addr) {
        target.bind_addr = Some(bind_addr);
    }
    if let Some(require_valkey_ready) = overlay.require_valkey_ready {
        target.require_valkey_ready = Some(require_valkey_ready);
    }
}
