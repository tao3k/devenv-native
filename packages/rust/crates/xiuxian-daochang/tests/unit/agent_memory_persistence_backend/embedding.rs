use std::time::{Duration, Instant};

use super::support::{
    build_agent_with_optional_session_valkey_url, create_temp_dir, require_ok, reserve_local_addr,
    spawn_slow_embedding_server, state_paths,
};
use xiuxian_daochang::MemoryConfig;

#[tokio::test]
async fn memory_turn_store_skips_episode_when_embedding_endpoint_is_unavailable() {
    let temp_dir = create_temp_dir();
    let table_name = "embed_endpoint_down".to_string();
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        embedding_backend: Some("http".to_string()),
        embedding_base_url: Some("http://127.0.0.1:3302".to_string()),
        embedding_model: Some("ollama/qwen3-embedding:0.6b".to_string()),
        embedding_dim: 1024,
        persistence_backend: "local".to_string(),
        ..MemoryConfig::default()
    };
    let (episodes_path, q_path) = state_paths(&memory.path, &memory.table_name);
    let agent = require_ok(
        build_agent_with_optional_session_valkey_url(memory, None).await,
        "agent should initialize when embedding endpoint is unavailable",
    );

    let started = Instant::now();
    require_ok(
        agent
            .append_turn_for_session("embed-unavailable-session", "u1", "a1")
            .await,
        "turn append should still succeed when embedding service is unavailable",
    );
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "embedding unavailable path should not block turn append unexpectedly"
    );

    assert!(
        episodes_path.exists(),
        "episode snapshot should be created via hash fallback when embedding is unavailable"
    );
    assert!(
        q_path.exists(),
        "q-table snapshot should be created via hash fallback when embedding is unavailable"
    );
    let metrics = agent.inspect_memory_recall_metrics().await;
    assert_eq!(metrics.embedding_success_total, 0);
    assert_eq!(
        metrics
            .embedding_unavailable_total
            .saturating_add(metrics.embedding_timeout_total),
        1
    );
    assert_eq!(metrics.embedding_cooldown_reject_total, 0);
}

#[tokio::test]
async fn memory_turn_store_skips_episode_when_embedding_unavailable_even_with_tools() {
    let temp_dir = create_temp_dir();
    let table_name = "embed_endpoint_down_tool_skip".to_string();
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        embedding_backend: Some("http".to_string()),
        embedding_base_url: Some("http://127.0.0.1:3302".to_string()),
        embedding_model: Some("ollama/qwen3-embedding:0.6b".to_string()),
        embedding_dim: 1024,
        persistence_backend: "local".to_string(),
        ..MemoryConfig::default()
    };
    let (episodes_path, q_path) = state_paths(&memory.path, &memory.table_name);
    let agent = require_ok(
        build_agent_with_optional_session_valkey_url(memory, None).await,
        "agent should initialize when embedding endpoint is unavailable",
    );

    require_ok(
        agent
            .append_turn_with_tool_count_for_session(
                "embed-unavailable-tool-skip-session",
                "u1",
                "analysis completed with fallback",
                2,
            )
            .await,
        "turn append should still succeed when embedding is unavailable",
    );

    assert!(
        episodes_path.exists(),
        "episode snapshot should be created via hash fallback when embedding is unavailable"
    );
    assert!(
        q_path.exists(),
        "q-table snapshot should be created via hash fallback when embedding is unavailable"
    );

    let metrics = agent.inspect_memory_recall_metrics().await;
    assert_eq!(metrics.embedding_success_total, 0);
    assert_eq!(
        metrics
            .embedding_unavailable_total
            .saturating_add(metrics.embedding_timeout_total),
        1
    );
    assert_eq!(metrics.embedding_cooldown_reject_total, 0);
}

#[tokio::test]
async fn memory_embedding_timeout_cooldown_skips_repeated_waits() {
    let temp_dir = create_temp_dir();
    let table_name = "embed_timeout_cooldown".to_string();
    let embedding_dim = 64;
    let addr = reserve_local_addr().await;
    let server_handle = spawn_slow_embedding_server(addr, 10_000, embedding_dim).await;
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        embedding_backend: Some("http".to_string()),
        embedding_base_url: Some(format!("http://{addr}")),
        embedding_dim,
        embedding_timeout_ms: Some(2_000),
        embedding_timeout_cooldown_ms: Some(20_000),
        persistence_backend: "local".to_string(),
        ..MemoryConfig::default()
    };
    let agent = require_ok(
        build_agent_with_optional_session_valkey_url(memory, None).await,
        "agent should initialize with slow embedding endpoint",
    );

    let first_started = Instant::now();
    require_ok(
        agent
            .append_turn_for_session("embed-cooldown-session", "first-timeout-intent", "a1")
            .await,
        "first turn append should still succeed when embedding times out",
    );
    let first_elapsed = first_started.elapsed();

    let second_started = Instant::now();
    require_ok(
        agent
            .append_turn_for_session("embed-cooldown-session", "second-timeout-intent", "a2")
            .await,
        "second turn append should still succeed during cooldown reject",
    );
    let second_elapsed = second_started.elapsed();

    assert!(
        first_elapsed >= Duration::from_millis(1_500),
        "first turn should include embedding timeout wait; elapsed={first_elapsed:?}"
    );
    assert!(
        second_elapsed + Duration::from_millis(300) < first_elapsed,
        "second turn should bypass most embedding wait during cooldown; first={first_elapsed:?}, second={second_elapsed:?}"
    );
    let metrics = agent.inspect_memory_recall_metrics().await;
    assert_eq!(
        metrics.embedding_timeout_total, 1,
        "first turn should record timeout"
    );
    assert_eq!(
        metrics.embedding_cooldown_reject_total, 1,
        "second turn should record cooldown reject"
    );
    assert_eq!(
        metrics.embedding_success_total, 0,
        "slow server should not produce successful embeddings in this scenario"
    );
    assert_eq!(metrics.embedding_unavailable_total, 0);

    server_handle.abort();
    let _ = server_handle.await;
}
