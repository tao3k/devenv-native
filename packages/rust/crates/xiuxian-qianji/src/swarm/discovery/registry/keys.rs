use crate::swarm::discovery_model::ClusterNodeIdentity;

pub(super) fn node_key(identity: &ClusterNodeIdentity) -> String {
    format!(
        "xiuxian:swarm:registry:{}:{}",
        identity.cluster_id, identity.agent_id
    )
}
