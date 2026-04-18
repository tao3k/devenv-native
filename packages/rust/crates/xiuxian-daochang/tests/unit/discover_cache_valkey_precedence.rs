//! Discover-cache Valkey precedence tests for config and env resolution.

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use axum::Router;
use rmcp::ServerHandler;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Content, ErrorData, ListToolsResult,
    PaginatedRequestParams, ServerCapabilities, ServerInfo, Tool,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use tempfile::TempDir;
use xiuxian_daochang::{ToolPoolConnectConfig, connect_tool_pool};

const CHILD_ENV_KEY: &str = "XIUXIAN_DAOCHANG_DISCOVER_CACHE_PRECEDENCE_CHILD";
const CHILD_CASE_KEY: &str = "XIUXIAN_DAOCHANG_DISCOVER_CACHE_PRECEDENCE_CASE";

#[derive(Clone, Default)]
struct DiscoverMockServer;

impl DiscoverMockServer {
    fn discover_tool() -> Tool {
        let input_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "intent": { "type": "string" }
            },
            "required": ["intent"]
        });
        let map = input_schema.as_object().cloned().unwrap_or_default();
        Tool {
            name: "skill.discover".into(),
            title: Some("Skill Discover".into()),
            description: Some("Mock discover tool".into()),
            input_schema: Arc::new(map),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        }
    }
}

impl ServerHandler for DiscoverMockServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        std::future::ready(Ok(ListToolsResult::with_all_items(vec![
            Self::discover_tool(),
        ])))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, ErrorData>> + Send + '_ {
        if request.name != "skill.discover" {
            return std::future::ready(Err(ErrorData::internal_error(
                "unsupported tool in discover cache precedence test",
                None,
            )));
        }
        let result = CallToolResult::success(vec![Content::text("ok")]);
        std::future::ready(Ok(result))
    }
}

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

async fn reserve_local_addr() -> std::net::SocketAddr {
    let probe = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
        Ok(listener) => listener,
        Err(error) => panic!("reserve local addr: {error}"),
    };
    let addr = match probe.local_addr() {
        Ok(addr) => addr,
        Err(error) => panic!("read reserved local addr: {error}"),
    };
    drop(probe);
    addr
}

async fn spawn_mock_server(addr: std::net::SocketAddr) -> tokio::task::JoinHandle<()> {
    let service: StreamableHttpService<DiscoverMockServer, LocalSessionManager> =
        StreamableHttpService::new(
            || Ok(DiscoverMockServer),
            Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig {
                stateful_mode: true,
                sse_keep_alive: None,
                ..Default::default()
            },
        );
    let router = Router::new().nest_service("/sse", service);
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(error) => panic!("bind mock tool listener: {error}"),
    };
    tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    })
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
