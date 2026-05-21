use super::{Episode, EpisodeDraft, TestResult, test_store};

#[test]
fn test_multi_hop_recall() -> TestResult {
    let store = test_store("test");

    let episodes = [
        ("debug api timeout", "Increased timeout to 60s", "success"),
        ("debug database slow", "Added index on column", "success"),
        ("debug memory leak", "Replaced HashMap with LRU", "success"),
        ("debug network error", "Checked DNS settings", "success"),
        ("debug file upload", "Increased size limit", "success"),
    ];

    for (i, (intent, exp, outcome)) in episodes.iter().enumerate() {
        let ep = Episode::new(EpisodeDraft {
            id: (format!("ep-{i}")).into(),
            intent: intent.to_string(),
            intent_embedding: store.encoder().encode(intent),
            experience: exp.to_string(),
            outcome: outcome.to_string(),
            scope: None,
        });
        store.store(ep)?;
    }

    let single_hop = store.multi_hop_recall(&["database problem".to_string()], 3, 0.5);
    assert!(!single_hop.is_empty(), "Should have results for single hop");

    let multi_hop = store.multi_hop_recall(
        &[
            "database problem".to_string(),
            "performance optimization".to_string(),
        ],
        3,
        0.5,
    );
    assert!(!multi_hop.is_empty(), "Should have results for multi-hop");

    println!("Single hop results: {:?}", single_hop.len());
    println!("Multi-hop results: {:?}", multi_hop.len());
    Ok(())
}

#[test]
fn test_multi_hop_recall_with_embeddings() -> TestResult {
    let store = test_store("test");
    let encoder = store.encoder();

    let episodes = [
        ("debug api timeout", "Increased timeout"),
        ("debug database slow", "Added index"),
        ("fix performance issue", "Used caching"),
    ];

    for (i, (intent, exp)) in episodes.iter().enumerate() {
        let ep = Episode::new(EpisodeDraft {
            id: (format!("ep-{i}")).into(),
            intent: intent.to_string(),
            intent_embedding: encoder.encode(intent),
            experience: exp.to_string(),
            outcome: "success".to_string(),
            scope: None,
        });
        store.store(ep)?;
    }

    let embeddings = vec![
        encoder.encode("api problem"),
        encoder.encode("timeout issue"),
    ];

    let results = store.multi_hop_recall_with_embeddings(&embeddings, 3, 0.5);
    assert!(!results.is_empty(), "Should have results");
    Ok(())
}
