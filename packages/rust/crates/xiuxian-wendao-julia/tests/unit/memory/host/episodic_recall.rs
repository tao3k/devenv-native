use tempfile::TempDir;
use xiuxian_memory_engine::{
    Episode, EpisodeDraft, EpisodeStore, MemoryProjectionFilter, StoreConfig,
};

use crate::memory::host::episodic_recall::{
    EpisodicRecallQueryInputs, build_episodic_recall_request_batch_from_projection,
    build_episodic_recall_request_rows_from_projection,
};

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

fn make_store() -> Result<(TempDir, EpisodeStore), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let store = EpisodeStore::new(StoreConfig {
        path: temp.path().to_string_lossy().to_string(),
        embedding_dim: 3,
        table_name: "host-staging".to_string(),
    });
    Ok((temp, store))
}

fn scoped_episode(
    id: impl Into<String>,
    intent: impl Into<String>,
    intent_embedding: Vec<f32>,
    experience: impl Into<String>,
    outcome: impl Into<String>,
    scope: impl Into<String>,
) -> Episode {
    Episode::new_scoped(
        EpisodeDraft {
            id: id.into().into(),
            intent: intent.into(),
            intent_embedding,
            experience: experience.into(),
            outcome: outcome.into(),
            scope: None,
        }
        .with_scope(scope),
    )
}

#[test]
fn build_episodic_recall_request_rows_from_projection_maps_host_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let (_temp, store) = make_store()?;
    let mut episode = scoped_episode(
        "episode-alpha".to_string(),
        "alpha intent".to_string(),
        vec![1.0, 0.0, 0.0],
        "alpha experience".to_string(),
        "pending".to_string(),
        "alpha",
    );
    episode.success_count = 3;
    episode.failure_count = 1;
    episode.retrieval_count = 4;
    episode.created_at = 100;
    episode.updated_at = 200;
    store.store(episode)?;
    let q_value = store.update_q("episode-alpha", 1.0);

    let projection_rows = store.memory_projection_rows(&MemoryProjectionFilter::default());
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
    assert!((row.q_value - q_value).abs() < 1e-6);
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
    let (_temp, store) = make_store()?;

    for episode_id in ["episode-alpha", "episode-beta"] {
        let episode = scoped_episode(
            episode_id.to_string(),
            format!("{episode_id} intent"),
            vec![0.1, 0.2, 0.3],
            format!("{episode_id} experience"),
            "pending".to_string(),
            "alpha",
        );
        store.store(episode)?;
    }

    let projection_rows = store.memory_projection_rows(&MemoryProjectionFilter::default());
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
fn build_episodic_recall_request_batch_from_real_store_projection_respects_scope_filter()
-> Result<(), Box<dyn std::error::Error>> {
    let (_temp, store) = make_store()?;

    for (episode_id, scope) in [("episode-alpha", "alpha"), ("episode-beta", "beta")] {
        let episode = scoped_episode(
            episode_id.to_string(),
            format!("{scope} intent"),
            vec![0.3, 0.2, 0.1],
            format!("{scope} experience"),
            "pending".to_string(),
            scope,
        );
        store.store(episode)?;
    }

    let projection_rows = store.memory_projection_rows(&MemoryProjectionFilter {
        scope: Some("alpha".to_string()),
        limit: None,
    });
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
