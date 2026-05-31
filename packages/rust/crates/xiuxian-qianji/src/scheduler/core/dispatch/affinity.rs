use crate::engine::NodeExecutionAffinity;
use crate::scheduler::core::QianjiScheduler;
use crate::scheduler_policy::SchedulerExcludedClusterRef;
use petgraph::stable_graph::NodeIndex;

pub(in crate::scheduler::core) enum AuthVerdict {
    ExecuteLocally,
    ExecuteAsProxy,
    WaitForRemote,
}

impl QianjiScheduler {
    pub(in crate::scheduler::core) async fn resolve_execution_auth(
        &self,
        node_idx: NodeIndex,
    ) -> AuthVerdict {
        if !self.execution_identity.is_configured() {
            return AuthVerdict::ExecuteLocally;
        }

        let affinity = {
            let engine = self.engine.read().await;
            engine
                .graph
                .node_weight(node_idx)
                .map(|node| node.execution_affinity.clone())
        };

        let Some(affinity) = affinity else {
            return AuthVerdict::WaitForRemote;
        };

        if self.identity_matches_affinity(&affinity) {
            return AuthVerdict::ExecuteLocally;
        }

        if !self.execution_policy.allow_local_proxy_delegation {
            return AuthVerdict::WaitForRemote;
        }
        if !self
            .execution_policy
            .is_proxy_role_allowed(self.execution_identity.role_class.as_deref())
        {
            return AuthVerdict::WaitForRemote;
        }
        if affinity.agent_id.is_some() {
            return AuthVerdict::WaitForRemote;
        }
        let Some(required_role) = affinity.role_class.as_deref() else {
            return AuthVerdict::WaitForRemote;
        };

        if self.has_remote_candidate_for_role(required_role).await {
            return AuthVerdict::WaitForRemote;
        }

        log::warn!(
            "affinity failover activated: local role {:?} proxying missing role '{}' for node routing",
            self.execution_identity.role_class,
            required_role
        );
        let node_id = self
            .deferred_node_id(node_idx)
            .await
            .unwrap_or_else(|| format!("node#{}", node_idx.index()));
        self.emit_affinity_alert(node_id, required_role, None);
        AuthVerdict::ExecuteAsProxy
    }

    pub(in crate::scheduler::core) async fn should_execute_node(
        &self,
        node_idx: NodeIndex,
    ) -> bool {
        matches!(
            self.resolve_execution_auth(node_idx).await,
            AuthVerdict::ExecuteLocally | AuthVerdict::ExecuteAsProxy
        )
    }

    pub(in crate::scheduler::core) async fn deferred_node_role(
        &self,
        node_idx: NodeIndex,
    ) -> Option<String> {
        let engine = self.engine.read().await;
        engine
            .graph
            .node_weight(node_idx)
            .and_then(|node| node.execution_affinity.role_class.clone())
            .map(|value| value.to_ascii_lowercase())
    }

    pub(in crate::scheduler::core) async fn deferred_node_id(
        &self,
        node_idx: NodeIndex,
    ) -> Option<String> {
        let engine = self.engine.read().await;
        engine
            .graph
            .node_weight(node_idx)
            .map(|node| node.id.clone())
    }

    fn identity_matches_affinity(&self, affinity: &NodeExecutionAffinity) -> bool {
        let agent_allowed = match (
            affinity.agent_id.as_deref(),
            self.execution_identity.agent_id.as_deref(),
        ) {
            (Some(required), Some(active)) => required == active,
            _ => true,
        };
        let role_allowed = match (
            affinity.role_class.as_deref(),
            self.execution_identity.role_class.as_deref(),
        ) {
            (Some(required), Some(active)) => required.eq_ignore_ascii_case(active),
            _ => true,
        };
        agent_allowed && role_allowed
    }

    async fn has_remote_candidate_for_role(&self, role_class: &str) -> bool {
        let Some(registry) = &self.role_registry else {
            return true;
        };
        registry
            .has_role(
                role_class,
                Some(SchedulerExcludedClusterRef::new(self.cluster_id.as_str())),
            )
            .await
    }
}
