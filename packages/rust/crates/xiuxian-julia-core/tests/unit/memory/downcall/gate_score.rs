use xiuxian_memory_engine::{MemoryLifecycleState, MemoryUtilityLedger};

use super::fetch_gate_score_recommendation_rows_from_evidence;
use crate::memory::host::MemoryGateScoreEvidenceRow;
use crate::memory::test_support::{
    gate_score_response_batch, runtime_for_test, spawn_memory_service,
};

fn sample_evidence_rows() -> Vec<MemoryGateScoreEvidenceRow> {
    vec![MemoryGateScoreEvidenceRow {
        memory_id: "memory-1".to_string(),
        scenario_pack: Some("searchinfra".to_string()),
        ledger: MemoryUtilityLedger {
            react_revalidation_score: 0.9,
            graph_consistency_score: 0.8,
            omega_alignment_score: 0.85,
            ttl_score: 0.7,
            utility_score: 0.78,
            q_value: 0.75,
            usage_count: 4,
            failure_rate: 0.25,
        },
        current_state: MemoryLifecycleState::Active,
    }]
}

#[tokio::test]
async fn fetch_gate_score_recommendation_rows_from_evidence_roundtrips() {
    let route = "/memory/gate_score";
    let (base_url, server) = spawn_memory_service(gate_score_response_batch()).await;
    let runtime = runtime_for_test(base_url, route);

    let rows =
        fetch_gate_score_recommendation_rows_from_evidence(&runtime, &sample_evidence_rows())
            .await
            .unwrap_or_else(|error| panic!("gate-score downcall should succeed: {error}"));

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].verdict, "retain");

    server.abort();
}
