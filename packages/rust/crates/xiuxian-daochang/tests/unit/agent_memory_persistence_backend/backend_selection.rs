use super::support::{
    build_agent_with_optional_session_valkey_url, create_temp_dir, require_ok, reserve_local_addr,
    spawn_slow_embedding_server, state_paths,
};
use xiuxian_daochang::MemoryConfig;

#[tokio::test]
async fn local_memory_backend_initializes_without_valkey() {
    let temp_dir = create_temp_dir();
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        persistence_backend: "local".to_string(),
        ..MemoryConfig::default()
    };
    let agent = build_agent_with_optional_session_valkey_url(memory, None).await;
    assert!(
        agent.is_ok(),
        "local memory backend should initialize without valkey"
    );
}

#[tokio::test]
async fn strict_valkey_memory_backend_fails_when_unreachable() {
    let temp_dir = create_temp_dir();
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        persistence_backend: "valkey".to_string(),
        ..MemoryConfig::default()
    };
    match build_agent_with_optional_session_valkey_url(memory, Some("redis://127.0.0.1:1/0")).await
    {
        Ok(_) => panic!("strict valkey backend should fail when redis is unreachable"),
        Err(err) => {
            assert!(
                err.to_string()
                    .contains("strict valkey memory backend failed during startup"),
                "unexpected error: {err}"
            );
        }
    }
}

#[tokio::test]
async fn auto_memory_backend_without_valkey_url_persists_locally() {
    let temp_dir = create_temp_dir();
    let table_name = "auto_local".to_string();
    let mut memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        persistence_backend: "auto".to_string(),
        ..MemoryConfig::default()
    };
    let addr = reserve_local_addr().await;
    let server_handle = spawn_slow_embedding_server(addr, 0, memory.embedding_dim).await;
    memory.embedding_base_url = Some(format!("http://{addr}"));
    let (episodes_path, q_path) = state_paths(&memory.path, &memory.table_name);
    let agent = require_ok(
        build_agent_with_optional_session_valkey_url(memory, None).await,
        "auto backend without redis url should initialize",
    );

    require_ok(
        agent
            .append_turn_for_session("auto-local-session", "u1", "a1")
            .await,
        "append turn should succeed",
    );

    assert!(
        episodes_path.exists(),
        "auto backend without redis url should persist local episode snapshot"
    );
    assert!(
        q_path.exists(),
        "auto backend without redis url should persist local q-table snapshot"
    );

    server_handle.abort();
    let _ = server_handle.await;
}

#[tokio::test]
async fn auto_memory_backend_with_unreachable_valkey_fails_by_default() {
    let temp_dir = create_temp_dir();
    let table_name = "auto_valkey".to_string();
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        persistence_backend: "auto".to_string(),
        ..MemoryConfig::default()
    };
    let (episodes_path, q_path) = state_paths(&memory.path, &memory.table_name);
    match build_agent_with_optional_session_valkey_url(memory, Some("redis://127.0.0.1:1/0")).await
    {
        Ok(_) => panic!("auto backend with valkey url should fail startup by default"),
        Err(err) => {
            assert!(
                err.to_string()
                    .contains("strict valkey memory backend failed during startup"),
                "unexpected error: {err}"
            );
        }
    }

    assert!(
        !episodes_path.exists(),
        "failed strict startup should not create local episode snapshot files"
    );
    assert!(
        !q_path.exists(),
        "failed strict startup should not create local q-table snapshot files"
    );
}

#[tokio::test]
async fn auto_memory_backend_can_relax_strict_startup_without_local_fallback() {
    let temp_dir = create_temp_dir();
    let table_name = "auto_valkey_relaxed".to_string();
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        table_name,
        persistence_backend: "auto".to_string(),
        persistence_strict_startup: Some(false),
        ..MemoryConfig::default()
    };
    let (episodes_path, q_path) = state_paths(&memory.path, &memory.table_name);
    let agent = require_ok(
        build_agent_with_optional_session_valkey_url(memory, Some("redis://127.0.0.1:1/0")).await,
        "auto backend should allow relaxed startup when explicitly configured",
    );

    require_ok(
        agent
            .append_turn_for_session("auto-valkey-relaxed-session", "u1", "a1")
            .await,
        "append turn should still succeed with relaxed startup",
    );

    assert!(
        !episodes_path.exists(),
        "auto backend with configured valkey should not silently fall back to local episode snapshot"
    );
    assert!(
        !q_path.exists(),
        "auto backend with configured valkey should not silently fall back to local q-table snapshot"
    );
}

#[tokio::test]
async fn auto_memory_backend_with_invalid_valkey_url_fails_fast() {
    let temp_dir = create_temp_dir();
    let memory = MemoryConfig {
        path: temp_dir.path().join("memory").to_string_lossy().to_string(),
        persistence_backend: "auto".to_string(),
        ..MemoryConfig::default()
    };
    match build_agent_with_optional_session_valkey_url(memory, Some("http://127.0.0.1:6379/0"))
        .await
    {
        Ok(_) => panic!("auto backend should fail when valkey url is invalid"),
        Err(err) => {
            assert!(
                err.to_string()
                    .contains("invalid redis url for memory persistence"),
                "unexpected error: {err}"
            );
        }
    }
}
