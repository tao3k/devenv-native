use std::collections::HashMap;

use super::support::{
    build_agent_with_optional_session_valkey_url, create_temp_dir, require_ok, reserve_local_addr,
    spawn_slow_embedding_server, state_paths,
};
use xiuxian_daochang::MemoryConfig;

#[tokio::test]
async fn memory_decay_policy_applies_on_configured_interval() {
    let temp_dir = create_temp_dir();
    let table_name = "decay_interval".to_string();
    let mut memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        persistence_backend: "local".to_string(),
        decay_enabled: true,
        decay_every_turns: 1,
        decay_factor: 0.5,
        ..MemoryConfig::default()
    };
    let addr = reserve_local_addr().await;
    let server_handle = spawn_slow_embedding_server(addr, 0, memory.embedding_dim).await;
    memory.embedding_base_url = Some(format!("http://{addr}"));
    let (_episodes_path, q_path) = state_paths(&memory.path, &memory.table_name);
    let agent = require_ok(
        build_agent_with_optional_session_valkey_url(memory, None).await,
        "agent should initialize for decay test",
    );

    require_ok(
        agent
            .append_turn_for_session("decay-session", "u1", "a1")
            .await,
        "append turn should succeed",
    );

    let raw = require_ok(
        std::fs::read_to_string(&q_path),
        "q-table snapshot should exist",
    );
    let q_values: HashMap<String, f32> =
        require_ok(serde_json::from_str(&raw), "q-table json should parse");
    assert_eq!(q_values.len(), 1, "expected one q-table entry");
    let q = q_values.values().next().copied().unwrap_or_default();
    assert!(
        q < 0.6,
        "decay should reduce first-turn q value below non-decay baseline (q={q})"
    );

    server_handle.abort();
    let _ = server_handle.await;
}
