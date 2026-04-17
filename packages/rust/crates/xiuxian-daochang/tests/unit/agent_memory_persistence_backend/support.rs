use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use axum::{Json, Router, extract::State, routing::post};
use xiuxian_daochang::{Agent, AgentConfig, MemoryConfig, set_config_home_override};

pub(super) fn require_ok<T, E>(result: std::result::Result<T, E>, context: &str) -> T
where
    E: std::fmt::Display,
{
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

pub(super) fn create_temp_dir() -> tempfile::TempDir {
    require_ok(tempfile::tempdir(), "failed to create temp dir")
}

fn base_agent_config(memory: MemoryConfig) -> AgentConfig {
    AgentConfig {
        inference_url: "http://127.0.0.1:4000/v1/chat/completions".to_string(),
        model: "test-model".to_string(),
        memory: Some(memory),
        ..AgentConfig::default()
    }
}

fn ensure_test_config_home_override() {
    static CONFIG_HOME: OnceLock<PathBuf> = OnceLock::new();
    let path = CONFIG_HOME.get_or_init(|| {
        let root = std::env::temp_dir()
            .join("xiuxian-daochang-tests")
            .join("agent_memory_persistence_backend");
        require_ok(
            std::fs::create_dir_all(&root),
            "create isolated config home for tests",
        );
        root
    });
    set_config_home_override(path.clone());
}

pub(super) async fn build_agent_with_optional_session_valkey_url(
    mut memory: MemoryConfig,
    session_valkey_url: Option<&str>,
) -> anyhow::Result<Agent> {
    // Isolate from developer-local ~/.config or PRJ_CONFIG_HOME overrides.
    ensure_test_config_home_override();
    if let Some(url) = session_valkey_url {
        memory.persistence_valkey_url = Some(url.to_string());
    }
    let config = base_agent_config(memory);
    Agent::from_config(config).await
}

pub(super) fn state_paths(memory_path: &str, table_name: &str) -> (PathBuf, PathBuf) {
    let root = Path::new(memory_path);
    (
        root.join(format!("{table_name}.episodes.json")),
        root.join(format!("{table_name}.q_table.json")),
    )
}

pub(super) async fn reserve_local_addr() -> std::net::SocketAddr {
    let probe = require_ok(
        tokio::net::TcpListener::bind("127.0.0.1:0").await,
        "reserve local addr",
    );
    let addr = require_ok(probe.local_addr(), "read reserved local addr");
    drop(probe);
    addr
}

async fn slow_embed_handler(
    State((sleep_ms, embedding_dim)): State<(u64, usize)>,
    Json(payload): Json<serde_json::Value>,
) -> Json<serde_json::Value> {
    let vector_count = payload
        .get("texts")
        .and_then(|value| value.as_array())
        .map_or(1, Vec::len);
    tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
    let vectors: Vec<Vec<f32>> = (0..vector_count)
        .map(|_| vec![0.0_f32; embedding_dim])
        .collect();
    Json(serde_json::json!({ "vectors": vectors }))
}

pub(super) async fn spawn_slow_embedding_server(
    addr: std::net::SocketAddr,
    sleep_ms: u64,
    embedding_dim: usize,
) -> tokio::task::JoinHandle<()> {
    let app = Router::new()
        .route("/embed/batch", post(slow_embed_handler))
        .with_state((sleep_ms, embedding_dim));
    let listener = require_ok(
        tokio::net::TcpListener::bind(addr).await,
        "bind slow embedding listener",
    );
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    })
}
