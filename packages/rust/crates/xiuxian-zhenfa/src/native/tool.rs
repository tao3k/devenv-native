//! Native tool trait and dispatch result contracts.

use async_trait::async_trait;
use serde_json::Value;

use super::{ZhenfaContext, ZhenfaError};

/// In-process native tool contract mounted in `ZhenfaRegistry`.
#[async_trait]
pub trait ZhenfaTool: Send + Sync {
    /// Unique tool identifier exposed to the orchestrator and LLM.
    fn id(&self) -> &str;

    /// Tool definition payload used by caller-side tool registration.
    fn definition(&self) -> Value;

    /// Execute tool logic in-process and return stripped payload for LLM context.
    ///
    /// # Errors
    /// Returns [`ZhenfaError`] when argument validation or domain execution fails.
    async fn call_native(&self, ctx: &ZhenfaContext, args: Value) -> Result<String, ZhenfaError>;

    /// Optional cache key used by the orchestrator for memoized dispatch.
    ///
    /// Tools should return `Some(key)` only when the output is deterministic for
    /// the provided context and args.
    #[must_use]
    fn cache_key(&self, _ctx: &ZhenfaContext, _args: &Value) -> Option<String> {
        None
    }

    /// Optional mutation scope used by the orchestrator for distributed locking.
    ///
    /// Tools that mutate shared state should return a stable lock scope string.
    #[must_use]
    fn mutation_scope(&self, _ctx: &ZhenfaContext, _args: &Value) -> Option<String> {
        None
    }
}
