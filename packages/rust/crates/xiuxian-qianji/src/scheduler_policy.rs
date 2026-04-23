//! Scheduler execution policy and role-availability probing contracts.

use crate::swarm::GlobalSwarmRegistry;
use async_trait::async_trait;

/// Runtime policy controlling affinity failover behavior.
#[derive(Debug, Clone)]
pub struct SchedulerExecutionPolicy {
    /// Enables local proxy execution when the required role is unavailable globally.
    pub allow_local_proxy_delegation: bool,
    /// Role classes allowed to act as emergency proxies.
    pub proxy_role_classes: Vec<String>,
}

impl SchedulerExecutionPolicy {
    /// Creates a policy with default proxy role classes (`manager`, `auditor`).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether local proxy delegation is allowed.
    #[must_use]
    pub fn with_local_proxy_delegation(mut self, enabled: bool) -> Self {
        self.allow_local_proxy_delegation = enabled;
        self
    }

    /// Returns true when `active_role` is authorized to proxy missing roles.
    #[must_use]
    pub fn is_proxy_role_allowed(&self, active_role: Option<&str>) -> bool {
        let Some(active_role) = active_role else {
            return false;
        };
        self.proxy_role_classes
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(active_role))
    }
}

impl Default for SchedulerExecutionPolicy {
    fn default() -> Self {
        Self {
            allow_local_proxy_delegation: false,
            proxy_role_classes: vec!["manager".to_string(), "auditor".to_string()],
        }
    }
}

/// Probe interface used by scheduler affinity logic to check global role availability.
#[async_trait]
pub trait RoleAvailabilityRegistry: Send + Sync {
    /// Returns true when at least one candidate exists for `role_class`.
    async fn has_role(&self, role_class: &str, exclude_cluster_id: Option<&str>) -> bool;
}

#[async_trait]
impl RoleAvailabilityRegistry for GlobalSwarmRegistry {
    async fn has_role(&self, role_class: &str, exclude_cluster_id: Option<&str>) -> bool {
        match self.pick_candidate(role_class, exclude_cluster_id).await {
            Ok(candidate) => candidate.is_some(),
            Err(error) => {
                log::warn!("role availability probe failed for role '{role_class}': {error}");
                true
            }
        }
    }
}
