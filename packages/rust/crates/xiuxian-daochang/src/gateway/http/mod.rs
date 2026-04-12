//! HTTP gateway: POST /message → agent turn → JSON response.
//!
//! Request validation (400 for empty `session_id` or message), 500 on agent error.
//! Each request is limited by a timeout to avoid stuck connections.

mod ai_sdk_stream;
pub(crate) mod handlers;
pub(crate) mod llm_proxy;
pub(crate) mod runtime;
mod types;

use anyhow::Result;
use axum::{
    Extension, Router,
    routing::{get, post},
};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

use crate::agent::Agent;

use self::ai_sdk_stream::handle_vercel_stream;
use self::handlers::{
    handle_embed, handle_embed_batch, handle_health, handle_message, handle_openai_embeddings,
};
use self::runtime::{build_embedding_runtime, build_embedding_runtime_for_gateway};
pub(crate) use self::types::GatewayEmbeddingRuntime;
pub use self::types::{
    GatewayExternalToolHealthResponse, GatewayHealthResponse, GatewayState, MessageRequest,
    MessageResponse,
};
pub use handlers::validate_message_request;

/// Default timeout for one agent turn (LLM + tools); avoids stuck connections.
const TURN_TIMEOUT_SECS: u64 = 300;

pub(crate) fn new_embedding_runtime() -> Arc<GatewayEmbeddingRuntime> {
    Arc::new(build_embedding_runtime())
}

pub(crate) fn embedding_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/embed", post(handle_embed))
        .route("/embed/batch", post(handle_embed_batch))
        .route("/embed/single", post(handle_embed))
        .route("/v1/embeddings", post(handle_openai_embeddings))
}

pub(crate) fn proxy_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new().route(
        "/v1/chat/completions",
        post(llm_proxy::handle_chat_completions),
    )
}

/// Build the gateway router (POST /message).
pub fn router(agent: Agent, turn_timeout_secs: u64, max_concurrent_turns: Option<usize>) -> Router {
    router_with_embedding_runtime(
        agent,
        turn_timeout_secs,
        max_concurrent_turns,
        new_embedding_runtime(),
    )
}

pub fn router_with_embedding_runtime(
    agent: Agent,
    turn_timeout_secs: u64,
    max_concurrent_turns: Option<usize>,
    embedding_runtime: Arc<GatewayEmbeddingRuntime>,
) -> Router {
    let concurrency_semaphore = max_concurrent_turns.map(|n| Arc::new(Semaphore::new(n)));
    let state = GatewayState {
        agent: Arc::new(agent),
        turn_timeout_secs,
        concurrency_semaphore,
        max_concurrent_turns,
        embedding_runtime: Arc::clone(&embedding_runtime),
    };

    Router::new()
        .route("/health", get(handle_health))
        .route("/message", post(handle_message))
        .route("/vercel/stream", post(handle_vercel_stream))
        .merge(embedding_routes::<GatewayState>())
        .merge(proxy_routes::<GatewayState>())
        .layer(Extension(embedding_runtime))
        .with_state(state)
}

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
