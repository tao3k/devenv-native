use std::collections::HashMap;

use anyhow::{Result, anyhow};

use crate::swarm::discovery_model::ClusterNodeIdentity;
use crate::swarm::discovery_util::current_unix_millis;

pub(super) fn heartbeat_payload(
    identity: &ClusterNodeIdentity,
    metadata: &serde_json::Value,
) -> Result<HashMap<String, String>> {
    if identity.cluster_id.trim().is_empty() {
        return Err(anyhow!("cluster_id must not be empty"));
    }
    if identity.agent_id.trim().is_empty() {
        return Err(anyhow!("agent_id must not be empty"));
    }
    if identity.role_class.trim().is_empty() {
        return Err(anyhow!("role_class must not be empty"));
    }

    let capabilities = serde_json::to_string(&identity.capabilities)?;
    let metadata_json = serde_json::to_string(metadata)?;
    let mut fields = HashMap::new();
    fields.insert("cluster_id".to_string(), identity.cluster_id.clone());
    fields.insert("agent_id".to_string(), identity.agent_id.clone());
    fields.insert("role_class".to_string(), identity.role_class.clone());
    fields.insert(
        "region".to_string(),
        identity.region.clone().unwrap_or_default(),
    );
    fields.insert(
        "endpoint".to_string(),
        identity.endpoint.clone().unwrap_or_default(),
    );
    fields.insert("capabilities".to_string(), capabilities);
    fields.insert("metadata".to_string(), metadata_json);
    fields.insert(
        "last_seen_ms".to_string(),
        current_unix_millis().to_string(),
    );
    Ok(fields)
}
