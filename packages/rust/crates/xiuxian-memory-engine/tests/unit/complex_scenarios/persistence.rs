use tempfile::TempDir;

use super::{Episode, EpisodeDraft, TestResult, test_store};

#[test]
fn test_persistence_and_recovery() -> TestResult {
    let temp_dir = TempDir::new()?;
    let store_path = temp_dir.path().join("store.json");
    let q_path = temp_dir.path().join("qtable.json");
    let store_path_str = store_path.to_string_lossy().to_string();
    let q_path_str = q_path.to_string_lossy().to_string();

    {
        let store = test_store("test");
        let ep = Episode::new(EpisodeDraft {
            id: ("persist-1".to_string()).into(),
            intent: "important task".to_string(),
            intent_embedding: store.encoder().encode("important task"),
            experience: "Critical fix".to_string(),
            outcome: "success".to_string(),
            scope: None,
        });
        store.store(ep)?;
        store.update_q("persist-1", 1.0);
        store.update_q("persist-1", 0.85);
        store.save(&store_path_str)?;
        store.save_q_table(&q_path_str)?;

        let saved_q = store.q_table.get_q("persist-1");
        println!("  Saved Q-value: {saved_q:.2}");
    }

    {
        let mut store = test_store("test");
        store.load(&store_path_str)?;
        store.load_q_table(&q_path_str)?;

        assert_eq!(store.len(), 1, "Should have 1 episode");

        let loaded_ep = store
            .get("persist-1")
            .ok_or_else(|| std::io::Error::other("Episode should exist"))?;
        println!("  Loaded episode Q-value: {:.2}", loaded_ep.q_value);

        let q = store.q_table.get_q("persist-1");
        println!("  Loaded Q-table value: {q:.2}");

        assert!((q - 0.65).abs() < 0.1, "Q-value should persist");
        println!("✓ Persistence and recovery: Episode and Q-value persisted correctly");
    }

    Ok(())
}
