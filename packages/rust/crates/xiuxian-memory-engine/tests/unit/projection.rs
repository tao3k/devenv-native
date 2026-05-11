//! Projection regression coverage for the Julia compute host read model.

use xiuxian_memory_engine::{
    Episode, EpisodeDraft, EpisodeStore, MemoryProjectionFilter, StoreConfig,
};

fn make_store() -> Result<(tempfile::TempDir, EpisodeStore), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let store = EpisodeStore::new(StoreConfig {
        path: temp.path().to_string_lossy().to_string(),
        embedding_dim: 4,
        table_name: "projection".to_string(),
    });
    Ok((temp, store))
}

#[test]
fn memory_projection_rows_export_read_only_episode_features()
-> Result<(), Box<dyn std::error::Error>> {
    let (_temp, store) = make_store()?;

    let mut alpha = Episode::new(EpisodeDraft {
        id: ("episode-alpha".to_string()).into(),
        intent: "alpha intent".to_string(),
        intent_embedding: vec![1.0, 0.0, 0.0, 0.0],
        experience: "alpha experience".to_string(),
        outcome: "success".to_string(),
        scope: Some(("alpha").to_string()),
    });
    alpha.retrieval_count = 7;
    alpha.success_count = 5;
    alpha.failure_count = 2;
    alpha.created_at = 1_000;
    alpha.updated_at = 2_000;
    store.store(alpha)?;
    let alpha_q = store.update_q("episode-alpha", 1.0);

    let mut beta = Episode::new(EpisodeDraft {
        id: ("episode-beta".to_string()).into(),
        intent: "beta intent".to_string(),
        intent_embedding: vec![0.0, 1.0, 0.0, 0.0],
        experience: "beta experience".to_string(),
        outcome: "pending".to_string(),
        scope: Some(("beta").to_string()),
    });
    beta.retrieval_count = 1;
    beta.created_at = 3_000;
    beta.updated_at = 4_000;
    store.store(beta)?;

    let rows = store.memory_projection_rows(&MemoryProjectionFilter::default());
    assert_eq!(rows.len(), 2);

    let alpha_row = &rows[0];
    assert_eq!(alpha_row.episode_id, "episode-alpha");
    assert_eq!(alpha_row.scope, "alpha");
    assert_eq!(alpha_row.intent_embedding, vec![1.0, 0.0, 0.0, 0.0]);
    assert!((alpha_row.q_value - alpha_q).abs() < 1e-6);
    assert_eq!(alpha_row.success_count, 5);
    assert_eq!(alpha_row.failure_count, 2);
    assert_eq!(alpha_row.retrieval_count, 7);
    assert_eq!(alpha_row.created_at_ms, 1_000);
    assert_eq!(alpha_row.updated_at_ms, 2_000);

    let beta_row = &rows[1];
    assert_eq!(beta_row.episode_id, "episode-beta");
    assert_eq!(beta_row.scope, "beta");
    assert_eq!(beta_row.intent_embedding, vec![0.0, 1.0, 0.0, 0.0]);
    assert!((beta_row.q_value - 0.5).abs() < 1e-6);
    assert_eq!(beta_row.success_count, 0);
    assert_eq!(beta_row.failure_count, 0);
    assert_eq!(beta_row.retrieval_count, 1);
    assert_eq!(beta_row.created_at_ms, 3_000);
    assert_eq!(beta_row.updated_at_ms, 4_000);

    Ok(())
}

#[test]
fn memory_projection_rows_filter_by_scope_and_limit() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp, store) = make_store()?;

    for (episode_id, scope) in [
        ("episode-alpha-1", "alpha"),
        ("episode-beta-1", "beta"),
        ("episode-alpha-2", "alpha"),
    ] {
        let episode = Episode::new(EpisodeDraft {
            id: (episode_id.to_string()).into(),
            intent: format!("{scope} intent"),
            intent_embedding: vec![0.1, 0.2, 0.3, 0.4],
            experience: format!("{scope} experience"),
            outcome: "pending".to_string(),
            scope: Some((scope).to_string()),
        });
        store.store(episode)?;
    }

    let alpha_rows = store.memory_projection_rows(&MemoryProjectionFilter {
        scope: Some("alpha".to_string()),
        limit: None,
    });
    assert_eq!(alpha_rows.len(), 2);
    assert_eq!(alpha_rows[0].episode_id, "episode-alpha-1");
    assert_eq!(alpha_rows[1].episode_id, "episode-alpha-2");

    let limited_rows = store.memory_projection_rows(&MemoryProjectionFilter {
        scope: Some("alpha".to_string()),
        limit: Some(1),
    });
    assert_eq!(limited_rows.len(), 1);
    assert_eq!(limited_rows[0].episode_id, "episode-alpha-1");

    Ok(())
}
