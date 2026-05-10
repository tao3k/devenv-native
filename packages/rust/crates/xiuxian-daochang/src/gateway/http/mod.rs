//! HTTP gateway: POST /message -> agent turn -> JSON response.
//!
//! Request validation (400 for empty `session_id` or message), 500 on agent error.
//! Each request is limited by a timeout to avoid stuck connections.

pub(crate) mod handlers;
pub(crate) mod llm_proxy;
mod routes;
pub(crate) mod runtime;
mod server;
mod types;

pub use self::routes::router;
pub(crate) use self::routes::{embedding_routes, new_embedding_runtime};
pub use self::server::run_http;
pub use self::types::{
    GatewayExternalToolHealthResponse, GatewayHealthResponse, GatewayState, MessageRequest,
    MessageResponse,
};
pub use handlers::{ValidatedMessageRequest, validate_message_request};
