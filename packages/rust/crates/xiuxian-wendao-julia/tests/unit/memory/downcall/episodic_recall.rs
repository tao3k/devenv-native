use xiuxian_memory_engine::MemoryProjectionRow;

use super::fetch_episodic_recall_score_rows_from_projection;
use crate::memory::host::EpisodicRecallQueryInputs;
use crate::memory::test_support::{
    episodic_recall_response_batch, runtime_for_test, spawn_memory_service,
};

fn sample_query() -> EpisodicRecallQueryInputs {
    EpisodicRecallQueryInputs {
        query_id: "query-1".to_string(),
        scenario_pack: Some("searchinfra".to_string()),
        query_text: Some("fix memory lane".to_string()),
        query_embedding: vec![0.1, 0.2, 0.3],
        k1: 1.0,
        k2: 0.5,
        lambda: 0.6,
        min_score: 0.1,
    }
}

fn sample_projection_rows() -> Vec<MemoryProjectionRow> {
    vec![MemoryProjectionRow {
        episode_id: "episode-1".to_string().into(),
        scope: "repo".to_string(),
        intent_embedding: vec![0.1, 0.2, 0.3],
        q_value: 0.7,
        success_count: 3,
        failure_count: 1,
        retrieval_count: 4,
        created_at_ms: 100.into(),
        updated_at_ms: 200.into(),
    }]
}

#[tokio::test]
async fn fetch_episodic_recall_score_rows_from_projection_roundtrips() {
    let route = "/memory/episodic_recall";
    let (base_url, server) = spawn_memory_service(episodic_recall_response_batch()).await;
    let runtime = runtime_for_test(base_url, route);

    let rows = fetch_episodic_recall_score_rows_from_projection(
        &runtime,
        &sample_query(),
        &sample_projection_rows(),
    )
    .await
    .unwrap_or_else(|error| panic!("episodic recall downcall should succeed: {error}"));

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].candidate_id, "episode-1");

    server.abort();
}
