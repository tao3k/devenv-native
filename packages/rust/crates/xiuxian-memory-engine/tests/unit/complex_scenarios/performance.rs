use std::time::Instant;

use super::{Episode, EpisodeDraft, TestResult, test_store};

#[test]
fn test_batch_operations_performance() -> TestResult {
    let store = test_store("test");
    let encoder = store.encoder();
    let start = Instant::now();

    for i in 0..1000 {
        let ep = Episode::new(EpisodeDraft {
            id: (format!("batch-{i}")).into(),
            intent: format!("task {}", i % 100),
            intent_embedding: encoder.encode(&format!("task {}", i % 100)),
            experience: format!("Solution {i}"),
            outcome: if i % 3 == 0 { "failure" } else { "success" }.to_string(),
            scope: None,
        });
        store.store(ep)?;
    }

    let store_time = start.elapsed();
    let query = encoder.encode("task 50");
    let recall_start = Instant::now();
    let results = store.recall_with_embedding(&query, 10);
    let recall_time = recall_start.elapsed();

    println!("✓ Batch operations performance:");
    println!("  - Store 1000 episodes: {store_time:?}");
    println!("  - Recall top-10: {recall_time:?}");
    println!("  - Results: {} episodes", results.len());

    assert!(store_time.as_millis() < 1000, "Store should be fast");
    assert!(recall_time.as_millis() < 100, "Recall should be fast");
    Ok(())
}
