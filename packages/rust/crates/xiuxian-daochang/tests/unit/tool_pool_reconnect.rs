#![allow(
    missing_docs,
    unused_imports,
    dead_code,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::field_reassign_with_default,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::unnecessary_literal_bound,
    clippy::needless_pass_by_value,
    clippy::struct_field_names,
    clippy::similar_names
)]

//! External tool pool reconnect smoke tests for xiuxian-daochang tool facade.
//!
//! Detailed reconnect/cache/fallback behavior is covered in the lower-level
//! tool-runtime transport test slice.

use std::time::Duration;

use crate::unit::tool_runtime_mock::{
    MockCallToolReply, MockToolRuntimeConfig, call_handler, permissive_tool_definition,
    reserve_local_addr, spawn_mock_tool_runtime, text_result,
};
use xiuxian_daochang::{ToolPoolConnectConfig, connect_tool_pool};

async fn spawn_mock_server(addr: std::net::SocketAddr) -> tokio::task::JoinHandle<()> {
    spawn_mock_tool_runtime(
        addr,
        MockToolRuntimeConfig::with_static_tools(
            vec![permissive_tool_definition(
                "mock_echo",
                "Echo for reconnect smoke test",
            )],
            call_handler(|request| async move {
                let msg = request
                    .arguments
                    .as_ref()
                    .and_then(|arguments| arguments.get("message"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("ok");
                MockCallToolReply::Result(text_result(format!("echo: {msg}")))
            }),
        ),
    )
    .await
}

fn reconnect_test_config() -> ToolPoolConnectConfig {
    ToolPoolConnectConfig {
        pool_size: 1,
        handshake_timeout_secs: 1,
        connect_retries: 6,
        connect_retry_backoff_ms: 100,
        tool_timeout_secs: 10,
        list_tools_cache_ttl_ms: 1_000,
    }
}

#[tokio::test]
async fn tool_pool_call_tool_recovers_after_server_restart() {
    let addr = reserve_local_addr().await;
    let handle_1 = spawn_mock_server(addr).await;
    let url = format!("http://{addr}/sse");
    let pool = connect_tool_pool(&url, reconnect_test_config()).await;
    let pool = match pool {
        Ok(pool) => pool,
        Err(error) => panic!("connect pool: {error}"),
    };

    let initial = pool
        .call_tool(
            "mock_echo".to_string(),
            Some(serde_json::json!({ "message": "first" })),
        )
        .await;
    let initial = match initial {
        Ok(initial) => initial,
        Err(error) => panic!("initial call_tool: {error}"),
    };
    assert_eq!(initial.text_segments.len(), 1);

    handle_1.abort();
    let _ = handle_1.await;

    let (restart_tx, restart_rx) = tokio::sync::oneshot::channel();
    let restart = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(300)).await;
        let handle = spawn_mock_server(addr).await;
        let _ = restart_tx.send(handle);
    });

    let recovered = pool
        .call_tool(
            "mock_echo".to_string(),
            Some(serde_json::json!({ "message": "second" })),
        )
        .await;
    let recovered = match recovered {
        Ok(recovered) => recovered,
        Err(error) => panic!("call_tool should recover after reconnect: {error}"),
    };
    assert_eq!(recovered.text_segments.len(), 1);

    if let Err(error) = restart.await {
        panic!("restart task join: {error}");
    }
    let handle_2 = match restart_rx.await {
        Ok(handle_2) => handle_2,
        Err(error) => panic!("restart handle receive: {error}"),
    };
    handle_2.abort();
    let _ = handle_2.await;
}
