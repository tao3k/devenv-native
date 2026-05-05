//! Domain router extension contract for the Zhenfa gateway.

use axum::Router;

use super::registry::MethodRegistry;

/// Trait implemented by domain crates to extend the Zhenfa gateway.
pub trait ZhenfaRouter: Send + Sync {
    /// Base URL prefix owned by this router (for example `/v1/wendao`).
    fn prefix(&self) -> &'static str;

    /// Mount domain routes into the shared Axum router.
    fn mount(&self, router: Router) -> Router;

    /// Register JSON-RPC methods handled by this router.
    fn register_methods(&self, _registry: &mut MethodRegistry) {}
}
