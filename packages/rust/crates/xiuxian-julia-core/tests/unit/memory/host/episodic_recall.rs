use crate::memory::host::episodic_recall::{
    EpisodicRecallQueryInputs, build_episodic_recall_request_batch_from_projection,
    build_episodic_recall_request_rows_from_projection,
};
use crate::memory::host::MemoryProjectionRow;

fn sample_query() -> EpisodicRecallQueryInputs {
    EpisodicRecallQueryInputs {
        query_id: "query-1".to_string(),
        scenario_pack: Some("searchinfra".to_string()),
        query_text: Some("how do we fix this".to_string()),
        query_embedding: vec![0.2, 0.4, 0.6],
        k1: 1.0,
        k2: 0.5,
        lambda: 0.7,
        min_score: 0.2,
    }
}

fn projection_row(
    episode_id: impl Into<String>,
    intent_embedding: Vec<f32>,
    scope: impl Into<String>,
) -> MemoryProjectionRow {
    MemoryProjectionRow {
        episode_id: episode_id.into(),
        scope: scope.into(),
        intent_embedding,
        q_value: 0.91,
        success_count: 3,
        failure_count: 1,
        retrieval_count: 4,
        created_at_ms: 100.into(),
        updated_at_ms: 200.into(),
    }
}

#[test]
fn build_episodic_recall_request_rows_from_projection_maps_host_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let projection_rows = vec![projection_row(
        "episode-alpha",
        vec![1.0, 0.0, 0.0],
        "alpha",
    )];
    let request_rows =
        build_episodic_recall_request_rows_from_projection(&sample_query(), &projection_rows)?;

    assert_eq!(request_rows.len(), 1);
    let row = &request_rows[0];
    assert_eq!(row.query_id, "query-1");
    assert_eq!(row.scenario_pack.as_deref(), Some("searchinfra"));
    assert_eq!(row.query_text.as_deref(), Some("how do we fix this"));
    assert_eq!(row.scope, "alpha");
    assert_eq!(row.candidate_id, "episode-alpha");
    assert_eq!(row.intent_embedding, vec![1.0, 0.0, 0.0]);
    assert!((row.q_value - 0.91).abs() < 1e-6);
    assert_eq!(row.success_count, 3);
    assert_eq!(row.failure_count, 1);
    assert_eq!(row.retrieval_count, 4);
    assert_eq!(row.created_at_ms, 100);
    assert_eq!(row.updated_at_ms, 200);

    Ok(())
}

#[test]
fn build_episodic_recall_request_batch_from_projection_materializes_staged_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let projection_rows = vec![
        projection_row("episode-alpha", vec![0.1, 0.2, 0.3], "alpha"),
        projection_row("episode-beta", vec![0.1, 0.2, 0.3], "alpha"),
    ];
    let batch =
        build_episodic_recall_request_batch_from_projection(&sample_query(), &projection_rows)?;

    assert_eq!(batch.num_rows(), 2);
    assert_eq!(batch.schema().fields().len(), 17);
    assert!(batch.column_by_name("query_id").is_some());
    assert!(batch.column_by_name("candidate_id").is_some());
    assert!(batch.column_by_name("intent_embedding").is_some());

    Ok(())
}

#[test]
fn build_episodic_recall_request_batch_from_projection_rejects_invalid_query_inputs() {
    let mut query = sample_query();
    query.query_id = "   ".to_string();
    let Err(error) = build_episodic_recall_request_batch_from_projection(&query, &[]) else {
        panic!("blank query_id must fail");
    };

    assert!(error.to_string().contains("query_id"));
}

#[test]
fn build_episodic_recall_request_batch_from_projection_keeps_host_filtered_rows()
-> Result<(), Box<dyn std::error::Error>> {
    let projection_rows = vec![projection_row("episode-alpha", vec![0.3, 0.2, 0.1], "alpha")];
    let batch =
        build_episodic_recall_request_batch_from_projection(&sample_query(), &projection_rows)?;
    let request_rows =
        build_episodic_recall_request_rows_from_projection(&sample_query(), &projection_rows)?;

    assert_eq!(batch.num_rows(), 1);
    assert_eq!(request_rows.len(), 1);
    assert_eq!(request_rows[0].candidate_id, "episode-alpha");
    assert_eq!(request_rows[0].scope, "alpha");

    Ok(())
}
