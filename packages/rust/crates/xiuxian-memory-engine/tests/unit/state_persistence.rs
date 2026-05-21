//! State persistence tests for `EpisodeStore`.

use xiuxian_memory_engine::{Episode, EpisodeDraft, EpisodeStore, StoreConfig};

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn save_state_creates_parent_dirs_and_loads_roundtrip() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    let store_root = temp_dir.path().join("nested").join("memory-state");

    let config = StoreConfig {
        path: store_root.to_string_lossy().to_string(),
        embedding_dim: 128,
        table_name: "episodes".to_string(),
    };

    let store = EpisodeStore::new(config.clone());
    let episode = Episode::new(EpisodeDraft {
        id: ("ep-1".to_string()).into(),
        intent: "fix timeout".to_string(),
        intent_embedding: store.encoder().encode("fix timeout"),
        experience: "Raised timeout and retried".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });
    store.store(episode)?;
    store.update_q("ep-1", 1.0);

    store.save_state()?;

    let episodes_path = store.episodes_state_path();
    let q_path = store.q_table_state_path();
    assert!(episodes_path.exists(), "episodes state file should exist");
    assert!(q_path.exists(), "q-table state file should exist");

    let mut reloaded = EpisodeStore::new(config);
    reloaded.load_state()?;

    assert_eq!(reloaded.len(), 1);
    assert!(reloaded.get("ep-1").is_some());
    assert!(reloaded.q_table.get_q("ep-1") > 0.5);
    Ok(())
}

#[test]
fn save_state_uses_table_scoped_filenames() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    let root = temp_dir.path().join("memory");

    let alpha = EpisodeStore::new(StoreConfig {
        path: root.to_string_lossy().to_string(),
        embedding_dim: 128,
        table_name: "alpha".to_string(),
    });
    alpha.store(Episode::new(EpisodeDraft {
        id: ("alpha-1".to_string()).into(),
        intent: "alpha task".to_string(),
        intent_embedding: alpha.encoder().encode("alpha task"),
        experience: "alpha experience".to_string(),
        outcome: "success".to_string(),
        scope: None,
    }))?;
    alpha.save_state()?;

    let beta = EpisodeStore::new(StoreConfig {
        path: root.to_string_lossy().to_string(),
        embedding_dim: 128,
        table_name: "beta".to_string(),
    });
    beta.store(Episode::new(EpisodeDraft {
        id: ("beta-1".to_string()).into(),
        intent: "beta task".to_string(),
        intent_embedding: beta.encoder().encode("beta task"),
        experience: "beta experience".to_string(),
        outcome: "success".to_string(),
        scope: None,
    }))?;
    beta.save_state()?;

    assert!(root.join("alpha.episodes.json").exists());
    assert!(root.join("alpha.q_table.json").exists());
    assert!(root.join("beta.episodes.json").exists());
    assert!(root.join("beta.q_table.json").exists());
    Ok(())
}
