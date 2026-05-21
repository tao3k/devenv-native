//! Bpmn runtime ownership surface for `xiuxian-qianji`.

use super::error::BpmnOrchestrationError;
use crate::scheduler_identity::SchedulerAgentIdentity;

/// Valkey-backed lease configuration for one BPMN scheduler owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QianjiBpmnSchedulerLeaseConfig {
    owner_token: String,
    lease_ttl_ms: u64,
}

impl QianjiBpmnSchedulerLeaseConfig {
    /// Creates one BPMN scheduler lease configuration.
    #[must_use]
    pub fn new(owner_token: impl Into<String>, lease_ttl_ms: u64) -> Self {
        Self {
            owner_token: owner_token.into(),
            lease_ttl_ms,
        }
    }

    /// Derives one BPMN scheduler lease configuration from the host scheduler
    /// execution identity.
    ///
    /// `role_class` is intentionally excluded from the owner token because it
    /// can describe routing policy but is not a stable single-writer identity
    /// axis for checkpoint ownership.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError::CheckpointLeaseAgentIdRequired`] when
    /// the scheduler identity does not expose a non-empty `agent_id`.
    pub fn from_scheduler_identity(
        identity: &SchedulerAgentIdentity,
        lease_ttl_ms: u64,
    ) -> Result<Self, BpmnOrchestrationError> {
        let Some(agent_id) = identity
            .agent_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Err(BpmnOrchestrationError::CheckpointLeaseAgentIdRequired);
        };

        Ok(Self::new(
            format!("bpmn-scheduler:{agent_id}"),
            lease_ttl_ms,
        ))
    }

    /// Returns the owner token used for lease acquire/renew/release.
    #[must_use]
    pub fn owner_token(&self) -> &str {
        self.owner_token.as_str()
    }

    /// Returns the lease TTL in milliseconds.
    #[must_use]
    pub fn lease_ttl_ms(&self) -> u64 {
        self.lease_ttl_ms
    }
}
