//! Scheduler checkpoint serialization and keying tests.

use serde_json::json;
use std::collections::{HashMap, HashSet};
use xiuxian_qianji::contracts::NodeStatus;
use xiuxian_qianji::scheduler::QianjiStateSnapshot;

#[test]
fn test_qianji_checkpoint_redis_key() {
    let key = QianjiStateSnapshot::redis_key("session_123");
    assert_eq!(key, "xq:qianji:checkpoint:session_123");
}

#[test]
fn test_qianji_checkpoint_serialization() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut active_branches = HashSet::new();
    active_branches.insert("branch_a".to_string());

    let mut node_statuses = HashMap::new();
    node_statuses.insert("NodeA".to_string(), NodeStatus::Completed);
    node_statuses.insert("NodeB".to_string(), NodeStatus::Executing);

    let snapshot = QianjiStateSnapshot {
        session_id: "test_session".to_string(),
        total_steps: 42,
        active_branches,
        context: json!({"key": "value"}),
        node_statuses,
    };

    let serialized = serde_json::to_string(&snapshot)?;
    assert!(serialized.contains("test_session"));
    assert!(serialized.contains("branch_a"));
    assert!(serialized.contains("completed"));
    assert!(serialized.contains("executing"));

    let raw_val: serde_json::Value = serde_json::from_str(&serialized)?;
    assert_eq!(raw_val["session_id"], "test_session");
    assert_eq!(raw_val["total_steps"], 42);
    assert_eq!(raw_val["node_statuses"]["NodeA"], "completed");
    assert_eq!(raw_val["node_statuses"]["NodeB"], "executing");

    let deserialized: QianjiStateSnapshot = serde_json::from_str(&serialized)?;
    assert_eq!(deserialized.session_id, "test_session");
    assert_eq!(deserialized.total_steps, 42);
    assert!(deserialized.active_branches.contains("branch_a"));
    assert_eq!(deserialized.context["key"], "value");
    assert_eq!(deserialized.node_statuses["NodeA"], NodeStatus::Completed);
    Ok(())
}

#[tokio::test]
#[ignore = "requires live valkey server"]
async fn test_qianji_checkpoint_redis_roundtrip()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut active_branches = HashSet::new();
    active_branches.insert("branch_test".to_string());

    let snapshot = QianjiStateSnapshot {
        session_id: "test_redis_roundtrip".to_string(),
        total_steps: 1,
        active_branches,
        context: json!({}),
        node_statuses: HashMap::new(),
    };

    let redis_url = "redis://127.0.0.1:6379/0";

    // 1. Save
    snapshot
        .save(redis_url)
        .await
        .map_err(std::io::Error::other)?;

    // 2. Load
    let loaded = QianjiStateSnapshot::load("test_redis_roundtrip", redis_url)
        .await
        .map_err(std::io::Error::other)?;
    assert!(loaded.is_some());
    let loaded = loaded.ok_or_else(|| std::io::Error::other("expected saved checkpoint"))?;
    assert_eq!(loaded.session_id, "test_redis_roundtrip");
    assert_eq!(loaded.total_steps, 1);

    // 3. Delete
    QianjiStateSnapshot::delete("test_redis_roundtrip", redis_url)
        .await
        .map_err(std::io::Error::other)?;

    // 4. Load after delete
    let loaded_after = QianjiStateSnapshot::load("test_redis_roundtrip", redis_url)
        .await
        .map_err(std::io::Error::other)?;
    assert!(loaded_after.is_none());
    Ok(())
}
