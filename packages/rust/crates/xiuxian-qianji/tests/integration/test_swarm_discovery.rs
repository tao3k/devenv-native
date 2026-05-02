//! Integration tests for global swarm discovery via Valkey heartbeat registry.

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use xiuxian_qianji::{ClusterNodeIdentity, GlobalSwarmRegistry};

use crate::valkey_support::TestValkey;

fn unique(prefix: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{prefix}_{now}")
}

#[tokio::test]
async fn heartbeat_registers_nodes_and_discovers_by_role() -> Result<(), Box<dyn std::error::Error>>
{
    let valkey = TestValkey::spawn().await?;
    let registry = GlobalSwarmRegistry::new(valkey.url().to_string());
    let cluster = unique("cluster_discovery_role");

    let student = ClusterNodeIdentity {
        cluster_id: cluster.clone(),
        agent_id: unique("student"),
        role_class: "student".to_string(),
        region: Some("us-west".to_string()),
        endpoint: Some("http://student.local".to_string()),
        capabilities: vec!["draft".to_string()],
    };
    let teacher = ClusterNodeIdentity {
        cluster_id: cluster,
        agent_id: unique("teacher"),
        role_class: "teacher".to_string(),
        region: Some("us-east".to_string()),
        endpoint: Some("http://teacher.local".to_string()),
        capabilities: vec!["audit".to_string()],
    };

    registry
        .heartbeat(&student, &serde_json::json!({"kind": "worker"}), 30)
        .await?;
    registry
        .heartbeat(&teacher, &serde_json::json!({"kind": "worker"}), 30)
        .await?;

    let teachers = registry.discover_by_role("teacher").await?;
    assert!(
        teachers.iter().any(|record| {
            record.identity.agent_id == teacher.agent_id && record.identity.role_class == "teacher"
        }),
        "expected teacher node in discovery result"
    );

    let all_nodes = registry.discover_all().await?;
    assert!(
        all_nodes
            .iter()
            .any(|record| record.identity.agent_id == student.agent_id),
        "expected student node in global discovery result"
    );
    Ok(())
}

#[tokio::test]
async fn discovery_prunes_stale_entries_after_ttl() -> Result<(), Box<dyn std::error::Error>> {
    let valkey = TestValkey::spawn().await?;
    let registry = GlobalSwarmRegistry::new(valkey.url().to_string());

    let node = ClusterNodeIdentity {
        cluster_id: unique("cluster_discovery_stale"),
        agent_id: unique("ephemeral_teacher"),
        role_class: "teacher".to_string(),
        region: None,
        endpoint: None,
        capabilities: vec!["audit".to_string()],
    };
    registry
        .heartbeat(&node, &serde_json::json!({"ephemeral": true}), 1)
        .await?;

    let before = registry.discover_by_role("teacher").await?;
    assert!(
        before
            .iter()
            .any(|record| record.identity.agent_id == node.agent_id),
        "expected node to be visible before ttl expiration"
    );

    tokio::time::sleep(Duration::from_millis(2_100)).await;

    let after = registry.discover_by_role("teacher").await?;
    assert!(
        !after
            .iter()
            .any(|record| record.identity.agent_id == node.agent_id),
        "expected node to be pruned after ttl expiration"
    );
    Ok(())
}
