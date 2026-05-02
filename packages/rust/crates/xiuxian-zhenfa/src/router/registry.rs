//! JSON-RPC method registry and closure adapter.

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::contracts::{JsonRpcErrorObject, JsonRpcMeta};

use super::traits::ZhenfaMethodHandler;

type Handler = Arc<dyn ZhenfaMethodHandler>;

/// Method registry used by `/rpc` dispatch.
#[derive(Clone, Default)]
pub struct MethodRegistry {
    handlers: HashMap<String, Handler>,
}

impl MethodRegistry {
    /// Register a concrete method handler.
    pub fn register_handler(
        &mut self,
        method: impl Into<String>,
        handler: Handler,
    ) -> Option<Handler> {
        self.handlers.insert(method.into(), handler)
    }

    /// Register an async closure as a method handler.
    pub fn register_fn<F, Fut>(&mut self, method: impl Into<String>, handler: F)
    where
        F: Fn(Value, Option<JsonRpcMeta>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<String, JsonRpcErrorObject>> + Send + 'static,
    {
        self.register_handler(method, method_handler(handler));
    }

    /// Call a registered method.
    ///
    /// # Errors
    /// Returns a JSON-RPC error when the method is not registered or when
    /// the underlying handler returns an error.
    pub async fn call(
        &self,
        method: &str,
        params: Value,
        meta: Option<JsonRpcMeta>,
    ) -> Result<String, JsonRpcErrorObject> {
        let Some(handler) = self.handlers.get(method) else {
            return Err(JsonRpcErrorObject::method_not_found(method));
        };
        handler.call(params, meta).await
    }

    /// Returns true when no methods are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
}

/// Convert an async closure into a boxed method handler.
pub fn method_handler<F, Fut>(handler: F) -> Handler
where
    F: Fn(Value, Option<JsonRpcMeta>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<String, JsonRpcErrorObject>> + Send + 'static,
{
    Arc::new(FnMethodHandler { handler })
}

struct FnMethodHandler<F> {
    handler: F,
}

#[async_trait]
impl<F, Fut> ZhenfaMethodHandler for FnMethodHandler<F>
where
    F: Fn(Value, Option<JsonRpcMeta>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<String, JsonRpcErrorObject>> + Send + 'static,
{
    async fn call(
        &self,
        params: Value,
        meta: Option<JsonRpcMeta>,
    ) -> Result<String, JsonRpcErrorObject> {
        (self.handler)(params, meta).await
    }
}
