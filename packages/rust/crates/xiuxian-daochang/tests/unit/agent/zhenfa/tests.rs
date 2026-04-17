use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use num_traits::ToPrimitive;
use redis::Commands;
use serde_json::json;
use tokio::sync::mpsc::UnboundedSender;
use xiuxian_daochang::XiuxianConfig;
use xiuxian_daochang::test_support::{
    ZhenfaRuntimeDeps, ZhenfaToolBridge, memory_reward_signal_sink,
    memory_reward_signal_sink_with_valkey_backend,
};
use xiuxian_memory_engine::{
    StoreConfig, default_valkey_state_hash_keys, default_valkey_state_key,
};
use xiuxian_qianhuan::ManifestationManager;
use xiuxian_zhenfa::{
    ZhenfaContext, ZhenfaError, ZhenfaOrchestrator, ZhenfaOrchestratorHooks, ZhenfaRegistry,
    ZhenfaSignal, ZhenfaTool,
};

struct RewardEmitterTool;

#[async_trait]
impl ZhenfaTool for RewardEmitterTool {
    fn id(&self) -> &'static str {
        "reward.emitter"
    }

    fn definition(&self) -> serde_json::Value {
        json!({
            "name": "reward.emitter",
            "description": "Emit one reward signal for memory sink tests",
            "parameters": {
                "type": "object",
                "properties": {
                    "episode_id": { "type": "string" },
                    "value": { "type": "number" }
                }
            }
        })
    }

    async fn call_native(
        &self,
        ctx: &ZhenfaContext,
        args: serde_json::Value,
    ) -> Result<String, ZhenfaError> {
        let episode_id = args
            .get("episode_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string();
        let value = args
            .get("value")
            .and_then(serde_json::Value::as_f64)
            .and_then(|raw| raw.to_f32())
            .unwrap_or(0.0);
        let signal_sender = ctx
            .get_extension::<UnboundedSender<ZhenfaSignal>>()
            .ok_or_else(|| ZhenfaError::execution("signal sender missing from context"))?;
        signal_sender
            .send(ZhenfaSignal::Reward {
                episode_id,
                value,
                source: "test.reward_emitter".to_string(),
            })
            .map_err(|_| ZhenfaError::execution("signal receiver closed"))?;
        Ok("<ok/>".to_string())
    }
}

fn build_manifestation_manager() -> Arc<ManifestationManager> {
    Arc::new(
        ManifestationManager::new_with_embedded_templates(&[], &[("bootstrap.md", "bootstrap")])
            .unwrap_or_else(|error| panic!("build manifestation manager: {error}")),
    )
}

#[test]
fn from_xiuxian_config_requires_explicit_supported_tool_enablement() {
    let config = XiuxianConfig::default();
    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &ZhenfaRuntimeDeps::default());
    assert!(bridge.is_none());
}

#[test]
fn from_xiuxian_config_skips_explicit_wendao_search_request() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.enabled_tools = Some(vec!["wendao.search".to_string()]);
    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &ZhenfaRuntimeDeps::default());
    assert!(bridge.is_none());
}

#[test]
fn from_xiuxian_config_filters_unknown_tools() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.enabled_tools = Some(vec!["unknown.tool".to_string()]);

    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &ZhenfaRuntimeDeps::default());
    assert!(bridge.is_none());
}

#[test]
fn from_xiuxian_config_skips_qianhuan_tools_without_runtime_dependency() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.enabled_tools = Some(vec![
        "qianhuan.render".to_string(),
        "qianhuan.reload".to_string(),
    ]);

    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &ZhenfaRuntimeDeps::default());
    assert!(bridge.is_none());
}

#[test]
fn from_xiuxian_config_enables_qianhuan_tools_when_runtime_dependency_is_available() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.enabled_tools = Some(vec![
        "qianhuan.render".to_string(),
        "qianhuan.reload".to_string(),
    ]);
    let manager = build_manifestation_manager();
    let deps = ZhenfaRuntimeDeps {
        manifestation_manager: Some(manager),
        memory_store: None,
    };

    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &deps)
        .unwrap_or_else(|| panic!("bridge should be enabled"));
    assert!(bridge.handles_tool("qianhuan.render"));
    assert!(bridge.handles_tool("qianhuan.reload"));
}

#[test]
fn from_xiuxian_config_enables_valkey_hooks_when_configured() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.enabled_tools = Some(vec!["qianhuan.reload".to_string()]);
    config.zhenfa.valkey.url = Some("redis://127.0.0.1:6379/0".to_string());
    let manager = build_manifestation_manager();
    let deps = ZhenfaRuntimeDeps {
        manifestation_manager: Some(manager),
        memory_store: None,
    };
    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &deps)
        .unwrap_or_else(|| panic!("bridge should be enabled"));
    assert!(bridge.valkey_hooks_enabled());
}

#[test]
fn from_xiuxian_config_ignores_wendao_search_when_mixed_with_supported_tools() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.enabled_tools = Some(vec![
        "wendao.search".to_string(),
        "qianhuan.reload".to_string(),
    ]);
    let manager = build_manifestation_manager();
    let deps = ZhenfaRuntimeDeps {
        manifestation_manager: Some(manager),
        memory_store: None,
    };
    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &deps)
        .unwrap_or_else(|| panic!("bridge should be enabled"));
    assert!(bridge.handles_tool("qianhuan.reload"));
    assert!(!bridge.handles_tool("wendao.search"));
}

#[tokio::test]
async fn call_tool_rejects_wendao_search_when_bridge_has_other_tools() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.enabled_tools = Some(vec![
        "wendao.search".to_string(),
        "qianhuan.reload".to_string(),
    ]);
    let manager = build_manifestation_manager();
    let deps = ZhenfaRuntimeDeps {
        manifestation_manager: Some(manager),
        memory_store: None,
    };
    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &deps)
        .unwrap_or_else(|| panic!("bridge should be enabled"));

    let Err(error) = bridge
        .call_tool(
            Some("telegram:12345"),
            "wendao.search",
            Some(json!({
                "request": "show me docs"
            })),
        )
        .await
    else {
        panic!("wendao.search should no longer be bridged through zhenfa");
    };
    assert!(error.to_string().contains("not enabled"));
}

#[tokio::test]
async fn call_tool_dispatches_qianhuan_reload_natively() {
    let mut config = XiuxianConfig::default();
    config.zhenfa.enabled_tools = Some(vec!["qianhuan.reload".to_string()]);
    let manager = build_manifestation_manager();
    let deps = ZhenfaRuntimeDeps {
        manifestation_manager: Some(manager),
        memory_store: None,
    };
    let bridge = ZhenfaToolBridge::from_xiuxian_config(&config, &deps)
        .unwrap_or_else(|| panic!("bridge should be enabled"));

    let output = bridge
        .call_tool(Some("telegram:12345"), "qianhuan.reload", None)
        .await
        .unwrap_or_else(|error| panic!("zhenfa native tool call should succeed: {error}"));
    assert!(output.contains("<qianhuan_reload"));
}

#[tokio::test]
async fn memory_reward_signal_sink_updates_q_value_through_orchestrator_signal_path() {
    let store = Arc::new(xiuxian_memory_engine::EpisodeStore::new(StoreConfig {
        path: tempfile::tempdir()
            .unwrap_or_else(|error| panic!("create temp dir: {error}"))
            .path()
            .to_string_lossy()
            .to_string(),
        ..StoreConfig::default()
    }));
    let sink = memory_reward_signal_sink(Arc::clone(&store));
    let mut registry = ZhenfaRegistry::new();
    registry.register(Arc::new(RewardEmitterTool));
    let orchestrator = ZhenfaOrchestrator::with_hooks(
        registry,
        ZhenfaOrchestratorHooks {
            cache: None,
            mutation_lock: None,
            audit_sink: None,
            signal_sink: Some(sink),
        },
    );

    let result = orchestrator
        .dispatch(
            "reward.emitter",
            &ZhenfaContext::default(),
            json!({
                "episode_id": "episode:signal-path",
                "value": 1.2
            }),
        )
        .await
        .unwrap_or_else(|error| panic!("dispatch should succeed: {error}"));
    assert_eq!(result, "<ok/>");

    for _ in 0..40 {
        let q = store.q_table.get_q("episode:signal-path");
        if (q - 0.6).abs() < 1e-4 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    let q = store.q_table.get_q("episode:signal-path");
    assert!(
        (q - 0.6).abs() < 1e-4,
        "unexpected q after reward signal: {q}"
    );
}

#[tokio::test]
async fn memory_reward_signal_sink_uses_correlation_id_when_episode_id_is_missing() {
    let store = Arc::new(xiuxian_memory_engine::EpisodeStore::new(StoreConfig {
        path: tempfile::tempdir()
            .unwrap_or_else(|error| panic!("create temp dir: {error}"))
            .path()
            .to_string_lossy()
            .to_string(),
        ..StoreConfig::default()
    }));
    let sink = memory_reward_signal_sink(Arc::clone(&store));
    let mut registry = ZhenfaRegistry::new();
    registry.register(Arc::new(RewardEmitterTool));
    let orchestrator = ZhenfaOrchestrator::with_hooks(
        registry,
        ZhenfaOrchestratorHooks {
            cache: None,
            mutation_lock: None,
            audit_sink: None,
            signal_sink: Some(sink),
        },
    );
    let mut ctx = ZhenfaContext::default();
    ctx.set_correlation_id_if_absent("episode:from-correlation");

    let result = orchestrator
        .dispatch(
            "reward.emitter",
            &ctx,
            json!({
                "episode_id": "",
                "value": -5.0
            }),
        )
        .await
        .unwrap_or_else(|error| panic!("dispatch should succeed: {error}"));
    assert_eq!(result, "<ok/>");

    for _ in 0..40 {
        let q = store.q_table.get_q("episode:from-correlation");
        if (q - 0.4).abs() < 1e-4 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    let q = store.q_table.get_q("episode:from-correlation");
    assert!(
        (q - 0.4).abs() < 1e-4,
        "unexpected q after correlation fallback signal: {q}"
    );
}

#[tokio::test]
async fn memory_reward_signal_bootcamp_penalize_then_recover() {
    let store = Arc::new(xiuxian_memory_engine::EpisodeStore::new(StoreConfig {
        path: tempfile::tempdir()
            .unwrap_or_else(|error| panic!("create temp dir: {error}"))
            .path()
            .to_string_lossy()
            .to_string(),
        ..StoreConfig::default()
    }));
    let sink = memory_reward_signal_sink(Arc::clone(&store));
    let mut registry = ZhenfaRegistry::new();
    registry.register(Arc::new(RewardEmitterTool));
    let orchestrator = ZhenfaOrchestrator::with_hooks(
        registry,
        ZhenfaOrchestratorHooks {
            cache: None,
            mutation_lock: None,
            audit_sink: None,
            signal_sink: Some(sink),
        },
    );

    let episode_id = "episode:bootcamp";
    for _ in 0..5 {
        let result = orchestrator
            .dispatch(
                "reward.emitter",
                &ZhenfaContext::default(),
                json!({
                    "episode_id": episode_id,
                    "value": 0.0
                }),
            )
            .await
            .unwrap_or_else(|error| panic!("dispatch should succeed: {error}"));
        assert_eq!(result, "<ok/>");
    }

    for _ in 0..40 {
        let q = store.q_table.get_q(episode_id);
        if q < 0.17 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let q_after_penalty = store.q_table.get_q(episode_id);
    assert!(
        q_after_penalty < 0.17,
        "q should drop after repeated penalties, got {q_after_penalty}"
    );

    let result = orchestrator
        .dispatch(
            "reward.emitter",
            &ZhenfaContext::default(),
            json!({
                "episode_id": episode_id,
                "value": 1.0
            }),
        )
        .await
        .unwrap_or_else(|error| panic!("dispatch should succeed: {error}"));
    assert_eq!(result, "<ok/>");

    for _ in 0..40 {
        let q = store.q_table.get_q(episode_id);
        if q > q_after_penalty {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let q_after_recovery = store.q_table.get_q(episode_id);
    assert!(
        q_after_recovery > q_after_penalty,
        "q should rebound after positive reward, before={q_after_penalty}, after={q_after_recovery}"
    );
}

#[tokio::test]
async fn memory_reward_signal_persists_q_to_valkey_when_backend_present() {
    let Ok(redis_url) = std::env::var("VALKEY_URL") else {
        return;
    };
    if redis_url.trim().is_empty() {
        return;
    }

    let temp_dir = tempfile::tempdir().unwrap_or_else(|error| panic!("create temp dir: {error}"));
    let store_config = StoreConfig {
        path: temp_dir.path().to_string_lossy().to_string(),
        ..StoreConfig::default()
    };
    let store = Arc::new(xiuxian_memory_engine::EpisodeStore::new(
        store_config.clone(),
    ));
    let key_prefix = format!(
        "xiuxian-daochang:memory:bootcamp-direct:{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|error| panic!("system time before UNIX_EPOCH: {error}"))
            .as_nanos()
    );
    let state_key = default_valkey_state_key(&key_prefix, &store_config);
    let (_episodes_hash_key, q_values_hash_key) = default_valkey_state_hash_keys(&state_key);
    let mut redis_connection = redis::Client::open(redis_url.as_str())
        .unwrap_or_else(|error| panic!("open redis client: {error}"))
        .get_connection()
        .unwrap_or_else(|error| panic!("open redis connection: {error}"));
    let episode_id = "episode:bootcamp:valkey";
    let _: () = redis_connection
        .hdel(&q_values_hash_key, episode_id)
        .unwrap_or_else(|error| panic!("clear q-value field before test: {error}"));

    let sink = memory_reward_signal_sink_with_valkey_backend(
        Arc::clone(&store),
        &redis_url,
        state_key,
        false,
    )
    .unwrap_or_else(|error| panic!("create valkey memory reward signal sink: {error}"));
    let mut registry = ZhenfaRegistry::new();
    registry.register(Arc::new(RewardEmitterTool));
    let orchestrator = ZhenfaOrchestrator::with_hooks(
        registry,
        ZhenfaOrchestratorHooks {
            cache: None,
            mutation_lock: None,
            audit_sink: None,
            signal_sink: Some(sink),
        },
    );

    let result = orchestrator
        .dispatch(
            "reward.emitter",
            &ZhenfaContext::default(),
            json!({
                "episode_id": episode_id,
                "value": 0.0
            }),
        )
        .await
        .unwrap_or_else(|error| panic!("dispatch should succeed: {error}"));
    assert_eq!(result, "<ok/>");

    for _ in 0..60 {
        let q = store.q_table.get_q(episode_id);
        if (q - 0.4).abs() < 1e-4 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let q_in_memory = store.q_table.get_q(episode_id);
    assert!(
        (q_in_memory - 0.4).abs() < 1e-4,
        "unexpected in-memory q after reward signal: {q_in_memory}"
    );

    let q_in_valkey: Option<f32> = redis_connection
        .hget(&q_values_hash_key, episode_id)
        .unwrap_or_else(|error| panic!("read valkey q-value field: {error}"));
    let Some(q_in_valkey) = q_in_valkey else {
        panic!("expected valkey q-value field to be written for {episode_id}");
    };
    assert!(
        (q_in_valkey - q_in_memory).abs() < 1e-4,
        "valkey q-value should match in-memory q, valkey={q_in_valkey}, memory={q_in_memory}"
    );
}
