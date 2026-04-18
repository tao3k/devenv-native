//! Integration tests for distributed consensus behavior.

use std::fmt::Display;
use std::time::{SystemTime, UNIX_EPOCH};
use xiuxian_qianji::consensus::{
    AgentIdentity, ConsensusManager, ConsensusMode, ConsensusPolicy, ConsensusResult,
};

use crate::valkey_support::TestValkey;

fn must_ok<T, E: Display>(value: Result<T, E>, context: &str) -> T {
    match value {
        Ok(inner) => inner,
        Err(error) => panic!("{context}: {error}"),
    }
}

xiuxian_testing::crate_test_policy_harness!();

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

#[tokio::test]
async fn test_consensus_majority_logic() {
    let valkey = must_ok(
        TestValkey::spawn().await,
        "ephemeral valkey should start for consensus majority test",
    );
    let redis_url = valkey.url().to_string();
    let manager_agent_1 = ConsensusManager::with_agent_identity(
        redis_url.clone(),
        AgentIdentity {
            id: "agent_1".to_string(),
            weight: 1.0,
        },
    );
    let manager_agent_2 = ConsensusManager::with_agent_identity(
        redis_url,
        AgentIdentity {
            id: "agent_2".to_string(),
            weight: 1.0,
        },
    );
    let unique_suffix = now_millis();
    let session_id = format!("test_session_{unique_suffix}");
    let node_id = "Professor_Audit";
    let policy = ConsensusPolicy {
        mode: ConsensusMode::Majority,
        min_agents: 2,
        timeout_ms: 5000,
        weight_threshold: 0.5,
    };

    let hash = "hash_result_good";

    let result = must_ok(
        manager_agent_1
            .submit_vote(&session_id, node_id, hash.to_string(), &policy)
            .await,
        "submit_vote for agent_1 should succeed",
    );
    assert!(matches!(result, ConsensusResult::Pending));

    let result = must_ok(
        manager_agent_2
            .submit_vote(&session_id, node_id, hash.to_string(), &policy)
            .await,
        "submit_vote for agent_2 should succeed",
    );
    assert_eq!(result, ConsensusResult::Agreed(hash.to_string()));
}

#[tokio::test]
async fn test_consensus_timeout_returns_failed_without_quorum() {
    let valkey = must_ok(
        TestValkey::spawn().await,
        "ephemeral valkey should start for consensus timeout test",
    );
    let redis_url = valkey.url().to_string();
    let manager_agent_1 = ConsensusManager::with_agent_identity(
        redis_url.clone(),
        AgentIdentity {
            id: "agent_timeout_1".to_string(),
            weight: 1.0,
        },
    );
    let manager_agent_2 = ConsensusManager::with_agent_identity(
        redis_url,
        AgentIdentity {
            id: "agent_timeout_2".to_string(),
            weight: 1.0,
        },
    );
    let unique_suffix = now_millis();
    let session_id = format!("test_session_timeout_{unique_suffix}");
    let node_id = "Forge_Guard";
    let policy = ConsensusPolicy {
        mode: ConsensusMode::Majority,
        min_agents: 2,
        timeout_ms: 1,
        weight_threshold: 0.5,
    };

    let _ = must_ok(
        manager_agent_1
            .submit_vote(&session_id, node_id, "hash_timeout_a".to_string(), &policy)
            .await,
        "submit_vote first timeout vote should succeed",
    );
    tokio::time::sleep(std::time::Duration::from_millis(3)).await;
    let result = must_ok(
        manager_agent_2
            .submit_vote(&session_id, node_id, "hash_timeout_b".to_string(), &policy)
            .await,
        "submit_vote second timeout vote should succeed",
    );
    assert!(
        matches!(
            result,
            ConsensusResult::Failed(ref reason) if reason == "consensus_timeout"
        ),
        "expected timeout failure, got: {result:?}"
    );
}

#[tokio::test]
async fn test_consensus_weighted_mode_agrees_on_threshold() {
    let valkey = must_ok(
        TestValkey::spawn().await,
        "ephemeral valkey should start for weighted consensus test",
    );
    let redis_url = valkey.url().to_string();
    let manager_weighted_1 = ConsensusManager::with_agent_identity(
        redis_url.clone(),
        AgentIdentity {
            id: "agent_weighted_1".to_string(),
            weight: 1.0,
        },
    );
    let manager_weighted_2 = ConsensusManager::with_agent_identity(
        redis_url,
        AgentIdentity {
            id: "agent_weighted_2".to_string(),
            weight: 0.6,
        },
    );
    let unique_suffix = now_millis();
    let session_id = format!("test_session_weighted_{unique_suffix}");
    let node_id = "Soul_Forger";
    let policy = ConsensusPolicy {
        mode: ConsensusMode::Weighted,
        min_agents: 2,
        timeout_ms: 5000,
        weight_threshold: 1.5,
    };

    let result = must_ok(
        manager_weighted_1
            .submit_vote(
                &session_id,
                node_id,
                "hash_weighted_ok".to_string(),
                &policy,
            )
            .await,
        "submit_vote weighted agent_1 should succeed",
    );
    assert!(matches!(result, ConsensusResult::Pending));

    let result = must_ok(
        manager_weighted_2
            .submit_vote(
                &session_id,
                node_id,
                "hash_weighted_ok".to_string(),
                &policy,
            )
            .await,
        "submit_vote weighted agent_2 should succeed",
    );
    assert_eq!(
        result,
        ConsensusResult::Agreed("hash_weighted_ok".to_string())
    );
}
