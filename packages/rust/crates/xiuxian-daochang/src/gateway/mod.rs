//! Gateway namespace: HTTP and stdio entrypoints.

pub(crate) mod http;
mod stdio;

pub use http::{
    GatewayExternalToolHealthResponse, GatewayHealthResponse, GatewayState, MessageRequest,
    MessageResponse, ValidatedMessageRequest, router, run_http, validate_message_request,
};
pub(crate) use http::{embedding_routes, new_embedding_runtime};
pub use stdio::{DEFAULT_STDIO_SESSION_ID, StdioSessionId, run_stdio};
