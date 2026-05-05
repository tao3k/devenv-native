//! HTTP gateway server lifecycle.

use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::agent::Agent;

use super::routes::router_with_embedding_runtime;
use super::runtime::build_embedding_runtime_for_gateway;

/// Default timeout for one agent turn (LLM + tools); avoids stuck connections.
pub(crate) const TURN_TIMEOUT_SECS: u64 = 300;

/// Run the HTTP server; binds to `bind_addr` (e.g. `0.0.0.0:8080`).
/// Graceful shutdown on Ctrl+C (SIGINT) and SIGTERM (Unix); in-flight requests complete before exit.
/// `turn_timeout_secs`: per-turn timeout (default 300 when None).
/// `max_concurrent_turns`: limit concurrent agent turns (None = no limit; Some(4) default from CLI).
///
/// # Errors
/// Returns an error when binding, serving, or graceful-shutdown serving fails.
pub async fn run_http(
    agent: Agent,
    bind_addr: &str,
    turn_timeout_secs: Option<u64>,
    max_concurrent_turns: Option<usize>,
) -> Result<()> {
    let timeout = turn_timeout_secs.unwrap_or(TURN_TIMEOUT_SECS);
    let embedding_runtime = Arc::new(build_embedding_runtime_for_gateway().await?);
    let app =
        router_with_embedding_runtime(agent, timeout, max_concurrent_turns, embedding_runtime);
    let listener = TcpListener::bind(bind_addr).await?;
    let max_str = max_concurrent_turns.map_or_else(|| "unlimited".to_string(), |n| n.to_string());

    tracing::info!(
        "gateway listening on {} (turn_timeout={}s, max_concurrent={}, Ctrl+C/SIGTERM to stop)",
        bind_addr,
        timeout,
        max_str
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("gateway stopped");
    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let ctrl_c = tokio::signal::ctrl_c();
        let mut sigterm = match signal(SignalKind::terminate()) {
            Ok(sigterm) => sigterm,
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    "failed to listen for SIGTERM; falling back to Ctrl+C only"
                );
                let _ = ctrl_c.await;
                return;
            }
        };

        tokio::select! {
            _ = ctrl_c => {}
            _ = sigterm.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for Ctrl+C");
    }
}
