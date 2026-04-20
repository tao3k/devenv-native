//! Discover-cache Valkey precedence tests for config and env resolution.

use std::path::Path;
use std::process::Command;

use crate::unit::tool_runtime_mock::{
    MockCallToolReply, MockToolRuntimeConfig, call_handler, reserve_local_addr,
    spawn_mock_tool_runtime, text_result, tool_definition,
};
use anyhow::{Context, Result, bail};
use tempfile::TempDir;
use xiuxian_daochang::{ToolPoolConnectConfig, connect_tool_pool};

const CHILD_ENV_KEY: &str = "XIUXIAN_DAOCHANG_DISCOVER_CACHE_PRECEDENCE_CHILD";
const CHILD_CASE_KEY: &str = "XIUXIAN_DAOCHANG_DISCOVER_CACHE_PRECEDENCE_CASE";

fn write_runtime_settings(root: &Path, system_toml: &str) -> Result<()> {
    let system_path =
        root.join("packages/rust/crates/xiuxian-daochang/resources/config/xiuxian.toml");
    let user_path = root.join(".config/xiuxian-artisan-workshop/xiuxian.toml");
    if let Some(parent) = system_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if let Some(parent) = user_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(system_path, system_toml)?;
    std::fs::write(user_path, "")?;
    Ok(())
}

fn reconnect_test_config() -> ToolPoolConnectConfig {
    ToolPoolConnectConfig {
        pool_size: 1,
        handshake_timeout_secs: 2,
        connect_retries: 6,
        connect_retry_backoff_ms: 100,
        tool_timeout_secs: 10,
        list_tools_cache_ttl_ms: 1_000,
    }
}

async fn spawn_mock_server(addr: std::net::SocketAddr) -> tokio::task::JoinHandle<()> {
    spawn_mock_tool_runtime(
        addr,
        MockToolRuntimeConfig::with_static_tools(
            vec![tool_definition(
                "skill.discover",
                "Mock discover tool",
                &serde_json::json!({
                    "type": "object",
                    "properties": {
                        "intent": { "type": "string" }
                    },
                    "required": ["intent"]
                }),
            )],
            call_handler(|request| async move {
                if request.name != "skill.discover" {
                    return MockCallToolReply::RpcError {
                        code: -32_603,
                        message: "unsupported tool in discover cache precedence test".to_string(),
                        data: None,
                    };
                }
                MockCallToolReply::Result(text_result("ok"))
            }),
        ),
    )
    .await
}

fn run_child_case(root: &Path, case: &str, valkey_url: &str) -> Result<()> {
    let test_binary = std::env::current_exe().context("resolve current test binary path")?;
    let output = Command::new(test_binary)
        .arg("--exact")
        .arg("discover_cache_valkey_precedence_child_probe")
        .arg("--nocapture")
        .env(CHILD_ENV_KEY, "1")
        .env(CHILD_CASE_KEY, case)
        .env("PRJ_ROOT", root)
        .env("PRJ_CONFIG_HOME", root.join(".config"))
        .env("VALKEY_URL", valkey_url)
        .env("XIUXIAN_DAOCHANG_TOOL_DISCOVER_CACHE_ENABLED", "true")
        .output()
        .with_context(|| format!("spawn child probe for case={case}"))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "child probe failed for case={case} exit_code={:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            stdout,
            stderr
        );
    }
    Ok(())
}

#[test]
fn discover_cache_valkey_url_resolution_prefers_settings_and_keeps_env_fallback() -> Result<()> {
    let case_settings_first = TempDir::new()?;
    write_runtime_settings(
        case_settings_first.path(),
        r#"
[tool_runtime]
discover_cache_enabled = true

[session]
valkey_url = "redis://127.0.0.1:6379/0"
"#,
    )?;
    run_child_case(
        case_settings_first.path(),
        "settings_first",
        "://invalid-url-should-not-win",
    )?;

    let case_env_fallback = TempDir::new()?;
    write_runtime_settings(
        case_env_fallback.path(),
        r"
[tool_runtime]
discover_cache_enabled = true
",
    )?;
    run_child_case(
        case_env_fallback.path(),
        "env_fallback",
        "redis://127.0.0.1:6379/1",
    )?;

    Ok(())
}

#[tokio::test]
async fn discover_cache_valkey_precedence_child_probe() -> Result<()> {
    if std::env::var(CHILD_ENV_KEY).ok().as_deref() != Some("1") {
        return Ok(());
    }

    let case = std::env::var(CHILD_CASE_KEY).unwrap_or_else(|_| "unknown".to_string());
    match case.as_str() {
        "settings_first" | "env_fallback" => {}
        other => bail!("unknown child probe case: {other}"),
    }

    let addr = reserve_local_addr().await;
    let handle = spawn_mock_server(addr).await;
    let url = format!("http://{addr}/sse");

    let pool = connect_tool_pool(&url, reconnect_test_config())
        .await
        .context("connect pool in child probe")?;
    let snapshot = pool.discover_cache_stats_snapshot();
    assert!(
        snapshot.is_some(),
        "discover cache should be enabled for case={case}"
    );

    handle.abort();
    let _ = handle.await;
    Ok(())
}
