use std::collections::HashMap;

use super::discovery_util::normalize_optional_text;
use crate::swarm::discovery_model::{ClusterNodeIdentity, ClusterNodeRecord};

pub(in crate::swarm) fn role_matches(role_filter: Option<&str>, role_class: &str) -> bool {
    match role_filter {
        Some("*") | None => true,
        Some(value) => value.eq_ignore_ascii_case(role_class),
    }
}

pub(in crate::swarm) fn parse_record(
    registry_key: String,
    fields: &HashMap<String, String>,
) -> Option<ClusterNodeRecord> {
    let cluster_id = fields.get("cluster_id")?.trim().to_string();
    let agent_id = fields.get("agent_id")?.trim().to_string();
    let role_class = fields.get("role_class")?.trim().to_ascii_lowercase();
    if cluster_id.is_empty() || agent_id.is_empty() || role_class.is_empty() {
        return None;
    }

    let capabilities = fields
        .get("capabilities")
        .and_then(|raw| serde_json::from_str::<Vec<String>>(raw).ok())
        .unwrap_or_default();
    let metadata = fields
        .get("metadata")
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
        .unwrap_or(serde_json::Value::Null);
    let last_seen_ms = fields
        .get("last_seen_ms")
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or_default();

    Some(ClusterNodeRecord {
        registry_key,
        identity: ClusterNodeIdentity {
            cluster_id,
            agent_id,
            role_class,
            region: normalize_optional_text(fields.get("region").cloned()),
            endpoint: normalize_optional_text(fields.get("endpoint").cloned()),
            capabilities,
        },
        last_seen_ms,
        metadata,
    })
}
