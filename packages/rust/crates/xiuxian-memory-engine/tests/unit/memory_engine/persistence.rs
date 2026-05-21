use tempfile::TempDir;

use super::{Episode, EpisodeDraft, TestResult, test_store};

#[test]
fn test_episode_store_persistence() -> TestResult {
    let temp_dir = TempDir::new()?;
    let episodes_path = temp_dir.path().join("episodes.json");
    let qtable_path = temp_dir.path().join("qtable.json");
    let episodes_path_str = episodes_path.to_string_lossy().into_owned();
    let qtable_path_str = qtable_path.to_string_lossy().into_owned();

    let store = test_store("test");

    let ep1 = Episode::new(EpisodeDraft {
        id: ("ep-1".to_string()).into(),
        intent: "debug api timeout".to_string(),
        intent_embedding: store.encoder().encode("debug api timeout"),
        experience: "Increased timeout".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });
    let ep2 = Episode::new(EpisodeDraft {
        id: ("ep-2".to_string()).into(),
        intent: "fix memory leak".to_string(),
        intent_embedding: store.encoder().encode("fix memory leak"),
        experience: "Replaced HashMap".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });
    let ep3 = Episode::new(EpisodeDraft {
        id: ("ep-3".to_string()).into(),
        intent: "optimize query".to_string(),
        intent_embedding: store.encoder().encode("optimize query"),
        experience: "Added index".to_string(),
        outcome: "failure".to_string(),
        scope: None,
    });

    store.store(ep1)?;
    store.store(ep2)?;
    store.store(ep3)?;
    store.update_q("ep-1", 1.0);
    store.update_q("ep-2", 1.0);
    store.update_q("ep-3", 0.0);

    store.save(&episodes_path_str)?;
    store.save_q_table(&qtable_path_str)?;

    assert!(episodes_path.exists());
    assert!(qtable_path.exists());

    let mut store2 = test_store("test2");
    store2.load(&episodes_path_str)?;
    store2.load_q_table(&qtable_path_str)?;

    assert_eq!(store2.len(), 3);
    assert!((store2.q_table.get_q("ep-1") - 0.6).abs() < 0.01);
    assert!((store2.q_table.get_q("ep-2") - 0.6).abs() < 0.01);
    assert!((store2.q_table.get_q("ep-3") - 0.4).abs() < 0.01);

    let results = store2.two_phase_recall("api timeout", 3, 3, 0.5);
    assert!(!results.is_empty());
    Ok(())
}

#[test]
fn test_memory_decay() -> TestResult {
    let store = test_store("test");

    let ep1 = Episode::new(EpisodeDraft {
        id: ("ep-1".to_string()).into(),
        intent: "debug api timeout".to_string(),
        intent_embedding: store.encoder().encode("debug api timeout"),
        experience: "Increased timeout".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });
    let ep2 = Episode::new(EpisodeDraft {
        id: ("ep-2".to_string()).into(),
        intent: "fix memory leak".to_string(),
        intent_embedding: store.encoder().encode("fix memory leak"),
        experience: "Replaced HashMap".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });

    store.store(ep1)?;
    store.store(ep2)?;
    store.update_q("ep-1", 1.0);
    store.update_q("ep-2", 0.0);

    let q1_before = store.q_table.get_q("ep-1");
    let q2_before = store.q_table.get_q("ep-2");
    assert!(
        (q1_before - 0.6).abs() < 0.01,
        "Expected q1=0.6, got {q1_before}"
    );
    assert!(
        (q2_before - 0.4).abs() < 0.01,
        "Expected q2=0.4, got {q2_before}"
    );

    store.apply_decay(0.5);

    let q1_after = store.q_table.get_q("ep-1");
    let q2_after = store.q_table.get_q("ep-2");
    assert!(q1_after < q1_before, "Expected q1 {q1_after} < {q1_before}");
    assert!(q1_after > 0.5, "Expected q1 > 0.5, got {q1_after}");
    assert!(q2_after > q2_before, "Expected q2 {q2_after} > {q2_before}");
    assert!(q2_after < 0.5, "Expected q2 < 0.5, got {q2_after}");

    store.apply_decay(0.5);
    let q1_final = store.q_table.get_q("ep-1");
    assert!(
        q1_final > 0.5 && q1_final < q1_before,
        "Expected 0.5 < q1_final {q1_final} < q1_before {q1_before}"
    );
    Ok(())
}

#[test]
fn test_memory_stats() -> TestResult {
    let store = test_store("test");

    let stats = store.stats();
    assert_eq!(stats.total_episodes, 0);
    assert_eq!(stats.q_table_size, 0);

    let ep1 = Episode::new(EpisodeDraft {
        id: ("ep-1".to_string()).into(),
        intent: "debug api timeout".to_string(),
        intent_embedding: store.encoder().encode("debug api timeout"),
        experience: "Increased timeout".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });
    store.store(ep1)?;

    let stats = store.stats();
    assert_eq!(stats.total_episodes, 1);
    assert_eq!(stats.q_table_size, 1);
    Ok(())
}
