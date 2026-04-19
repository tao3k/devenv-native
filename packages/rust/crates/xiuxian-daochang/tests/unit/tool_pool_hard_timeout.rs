//! External tool pool hard-timeout smoke tests for xiuxian-daochang tool facade.
//!
//! Detailed timeout and timeout-budget behavior lives in the lower-level
//! tool-runtime transport test slice.

use std::future::pending;
use std::time::{Duration, Instant};

use crate::unit::tool_runtime_mock::{
    MockCallToolReply, MockListToolsReply, MockToolRuntimeConfig, call_handler, list_handler,
    reserve_local_addr, spawn_mock_tool_runtime,
};
use xiuxian_daochang::{ToolPoolConnectConfig, connect_tool_pool};

async fn spawn_hanging_server(addr: std::net::SocketAddr) -> tokio::task::JoinHandle<()> {
    spawn_mock_tool_runtime(
        addr,
        MockToolRuntimeConfig::with_handlers(
            list_handler(|_request| async move { MockListToolsReply::Hang }),
            call_handler(|_request| async move {
                let _ = pending::<()>().await;
                MockCallToolReply::Hang
            }),
        ),
    )
    .await
}

fn hard_timeout_test_config() -> ToolPoolConnectConfig {
    ToolPoolConnectConfig {
        pool_size: 1,
        handshake_timeout_secs: 1,
        connect_retries: 1,
        connect_retry_backoff_ms: 10,
        tool_timeout_secs: 1,
        list_tools_cache_ttl_ms: 1,
    }
}

#[tokio::test]
async fn tool_pool_list_tools_hard_timeout_returns_promptly() {
    let addr = reserve_local_addr().await;
    let server = spawn_hanging_server(addr).await;
    let url = format!("http://{addr}/sse");
    let pool = connect_tool_pool(&url, hard_timeout_test_config()).await;
    let pool = match pool {
        Ok(pool) => pool,
        Err(error) => panic!("connect pool: {error}"),
    };

    let started = Instant::now();
    let Err(error) = pool.list_tools(None).await else {
        panic!("list_tools should timeout");
    };
    let elapsed = started.elapsed();
    let message = format!("{error:#}");

    assert!(
        message.contains("timed out after 1s"),
        "unexpected error message: {message}"
    );
    assert!(
        elapsed < Duration::from_secs(8),
        "hard timeout should return promptly, elapsed={elapsed:?}"
    );

    server.abort();
    let _ = server.await;
}
