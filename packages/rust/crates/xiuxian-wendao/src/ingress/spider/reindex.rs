//! `ingress::spider::reindex` owns Wendao ingress spider reindex behavior.

use super::errors::SpiderIngressError;

/// Hook for scheduling namespace-level partial re-index.
pub trait PartialReindexHook: Send + Sync {
    /// Trigger one partial re-index for changed semantic URIs.
    ///
    /// # Errors
    ///
    /// Returns [`SpiderIngressError`] when scheduling fails.
    fn trigger_partial_reindex(
        &self,
        namespace: &str,
        changed_uris: &[String],
    ) -> Result<(), SpiderIngressError>;
}

/// No-op partial re-index hook.
#[derive(Debug, Default)]
pub struct NoopPartialReindexHook;

impl PartialReindexHook for NoopPartialReindexHook {
    fn trigger_partial_reindex(
        &self,
        _namespace: &str,
        _changed_uris: &[String],
    ) -> Result<(), SpiderIngressError> {
        Ok(())
    }
}
