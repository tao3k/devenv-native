//! JSON-RPC method handler trait.

use async_trait::async_trait;
use serde_json::Value;

use crate::contracts::{JsonRpcErrorObject, JsonRpcMeta};

/// Async handler for one JSON-RPC `method`.
#[async_trait]
pub trait ZhenfaMethodHandler: Send + Sync {
    /// Execute method with JSON params and optional metadata.
    async fn call(
        &self,
        params: Value,
        meta: Option<JsonRpcMeta>,
    ) -> Result<String, JsonRpcErrorObject>;
}
