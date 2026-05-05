//! HTTP router composition for gateway and embedding endpoints.

use std::sync::Arc;

use axum::{
    Extension, Router,
    routing::{get, post},
};
use tokio::sync::Semaphore;

use crate::agent::Agent;

use super::handlers::{
    handle_embed, handle_embed_batch, handle_health, handle_message, handle_openai_embeddings,
};
use super::llm_proxy;
use super::runtime::build_embedding_runtime;
use super::types::{GatewayEmbeddingRuntime, GatewayState};

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

fn proxy_routes<S>() -> Router<S>
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

/// Builds an HTTP router with an explicit embedding runtime.
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
        .merge(embedding_routes::<GatewayState>())
        .merge(proxy_routes::<GatewayState>())
        .layer(Extension(embedding_runtime))
        .with_state(state)
}
