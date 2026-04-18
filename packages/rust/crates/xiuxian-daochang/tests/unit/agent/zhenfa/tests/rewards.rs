use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use redis::Commands;
use serde_json::json;
use xiuxian_daochang::test_support::{
    memory_reward_signal_sink, memory_reward_signal_sink_with_valkey_backend,
};
use xiuxian_memory_engine::{
    StoreConfig, default_valkey_state_hash_keys, default_valkey_state_key,
};
use xiuxian_zhenfa::{ZhenfaContext, ZhenfaOrchestrator, ZhenfaOrchestratorHooks, ZhenfaRegistry};

use super::support::RewardEmitterTool;

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
