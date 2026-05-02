//! Zhenfa bridge and valkey-hook helpers exposed for integration tests.

use std::sync::Arc;

use serde_json::Value;
use xiuxian_memory_engine::EpisodeStore;
use xiuxian_qianhuan::ManifestationManager;
use xiuxian_zhenfa::{ZhenfaOrchestratorHooks, ZhenfaSignalSink};

use crate::agent::zhenfa as internal;
use crate::config::XiuxianConfig;

/// Runtime dependencies used to build the zhenfa native tool bridge.
#[derive(Clone, Default)]
pub struct ZhenfaRuntimeDeps {
    /// Optional qianhuan manifestation manager used by zhenfa tools.
    pub manifestation_manager: Option<Arc<ManifestationManager>>,
    /// Optional memory store used by reward signal tools.
    pub memory_store: Option<Arc<EpisodeStore>>,
}

/// Test-facing wrapper over the internal zhenfa tool bridge.
#[derive(Clone)]
pub struct ZhenfaToolBridge {
    inner: internal::ZhenfaToolBridge,
}

impl ZhenfaToolBridge {
    /// Builds a test-facing bridge from project configuration and runtime deps.
    #[must_use]
    pub fn from_xiuxian_config(config: &XiuxianConfig, deps: &ZhenfaRuntimeDeps) -> Option<Self> {
        let internal_deps = internal::test_runtime_deps(
            deps.manifestation_manager.as_ref().map(Arc::clone),
            deps.memory_store.as_ref().map(Arc::clone),
        );
        internal::ZhenfaToolBridge::from_xiuxian_config(config, &internal_deps)
            .map(|inner| Self { inner })
    }

    /// Returns the configured zhenfa base URL.
    #[must_use]
    pub fn base_url(&self) -> Option<&str> {
        self.inner.base_url()
    }

    /// Returns the number of native tools exposed through the bridge.
    #[must_use]
    pub fn tool_count(&self) -> usize {
        self.inner.tool_count()
    }

    /// Returns whether valkey-backed zhenfa hooks are enabled.
    #[must_use]
    pub fn valkey_hooks_enabled(&self) -> bool {
        self.inner.valkey_hooks_enabled()
    }

    /// Renders the bridge tool definitions for LLM tool registration.
    #[must_use]
    pub fn list_for_llm(&self) -> Vec<Value> {
        self.inner.list_for_llm()
    }

    /// Returns whether the bridge can handle the named tool.
    #[must_use]
    pub fn handles_tool(&self, name: &str) -> bool {
        self.inner.handles_tool(name)
    }

    /// Execute one bridged tool call.
    ///
    /// # Errors
    ///
    /// Returns an error when arguments are invalid, tool is disabled, or
    /// native dispatch fails.
    pub async fn call_tool(
        &self,
        session_id: Option<&str>,
        name: &str,
        arguments: Option<Value>,
    ) -> anyhow::Result<String> {
        self.inner.call_tool(session_id, name, arguments).await
    }
}

/// Resolved valkey hook config used by zhenfa orchestrator hooks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZhenfaValkeyHookConfig {
    /// Valkey connection URL.
    pub url: String,
    /// Key prefix used for zhenfa hook state.
    pub key_prefix: String,
    /// Cache entry TTL in seconds.
    pub cache_ttl_seconds: u64,
    /// Distributed lock TTL in seconds.
    pub lock_ttl_seconds: u64,
    /// Audit stream key.
    pub audit_stream: String,
}

/// Resolves the valkey hook configuration from project config.
#[must_use]
pub fn resolve_zhenfa_valkey_hook_config(config: &XiuxianConfig) -> Option<ZhenfaValkeyHookConfig> {
    internal::valkey_hooks::resolve_zhenfa_valkey_hook_config(config).map(|resolved| {
        ZhenfaValkeyHookConfig {
            url: resolved.url,
            key_prefix: resolved.key_prefix,
            cache_ttl_seconds: resolved.cache_ttl_seconds,
            lock_ttl_seconds: resolved.lock_ttl_seconds,
            audit_stream: resolved.audit_stream,
        }
    })
}

/// Builds zhenfa orchestrator hooks from project config.
#[must_use]
pub fn build_zhenfa_orchestrator_hooks(config: &XiuxianConfig) -> Option<ZhenfaOrchestratorHooks> {
    internal::valkey_hooks::build_zhenfa_orchestrator_hooks(config)
}

/// Builds a memory-backed reward signal sink.
#[must_use]
pub fn memory_reward_signal_sink(memory_store: Arc<EpisodeStore>) -> Arc<dyn ZhenfaSignalSink> {
    internal::test_memory_reward_signal_sink(memory_store)
}

/// Build reward signal sink with valkey-backed atomic Q persistence.
///
/// # Errors
///
/// Returns an error when valkey backend initialization fails.
pub fn memory_reward_signal_sink_with_valkey_backend(
    memory_store: Arc<EpisodeStore>,
    redis_url: &str,
    state_key: String,
    strict_startup: bool,
) -> anyhow::Result<Arc<dyn ZhenfaSignalSink>> {
    internal::test_memory_reward_signal_sink_with_valkey_backend(
        memory_store,
        redis_url,
        state_key,
        strict_startup,
    )
}
