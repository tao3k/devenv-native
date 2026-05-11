use super::{Episode, EpisodeDraft, TestResult, test_store};

#[test]
fn test_memory_decay_scenario() -> TestResult {
    let store = test_store("test");
    let encoder = store.encoder();

    let ep1 = Episode::new(EpisodeDraft {
        id: ("fresh-high".to_string()).into(),
        intent: "recent success".to_string(),
        intent_embedding: encoder.encode("recent success"),
        experience: "Did X".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });
    store.store(ep1)?;
    store.update_q("fresh-high", 0.9);

    let ep2 = Episode::new(EpisodeDraft {
        id: ("old-low".to_string()).into(),
        intent: "old failure".to_string(),
        intent_embedding: encoder.encode("old failure"),
        experience: "Did Y".to_string(),
        outcome: "failure".to_string(),
        scope: None,
    });
    store.store(ep2)?;
    store.update_q("old-low", 0.1);

    let q_before = store.q_table.get_q("fresh-high");
    let q_before_low = store.q_table.get_q("old-low");

    store.apply_decay(0.5);

    let q_after = store.q_table.get_q("fresh-high");
    let q_after_low = store.q_table.get_q("old-low");

    println!("✓ Memory decay (Q-value decay towards 0.5):");
    println!("  - High Q before: {q_before:.3} -> after: {q_after:.3}");
    println!("  - Low Q before: {q_before_low:.3} -> after: {q_after_low:.3}");

    assert!(q_after < q_before, "High Q should decay towards 0.5");
    assert!(q_after_low > q_before_low, "Low Q should decay towards 0.5");
    Ok(())
}

#[test]
fn test_multi_hop_reasoning_scenario() -> TestResult {
    let store = test_store("test");
    let encoder = store.encoder();

    let chain = [
        ("api error", "Checked logs", "success", 0.8),
        ("timeout fix", "Increased timeout", "success", 0.9),
        ("network issue", "Checked firewall", "success", 0.7),
        ("unrelated task", "Random fix", "failure", 0.2),
    ];

    for (i, (intent, exp, outcome, q)) in chain.iter().enumerate() {
        let ep = Episode::new(EpisodeDraft {
            id: (format!("ep-{i}")).into(),
            intent: intent.to_string(),
            intent_embedding: encoder.encode(intent),
            experience: exp.to_string(),
            outcome: outcome.to_string(),
            scope: None,
        });
        store.store(ep)?;
        store.update_q(&format!("ep-{i}"), *q);
    }

    let queries = vec![
        encoder.encode("api error"),
        encoder.encode("timeout fix"),
        encoder.encode("network issue"),
    ];
    let results = store.multi_hop_recall_with_embeddings(&queries, 3, 0.3);

    println!("✓ Multi-hop reasoning:");
    println!("  - Query chain: api error → timeout fix → network issue");
    println!("  - Results: {} episodes", results.len());

    assert!(
        !results.is_empty(),
        "Multi-hop should find related experiences"
    );
    Ok(())
}

#[test]
fn test_incremental_learning() -> TestResult {
    let store = test_store("test");
    let encoder = store.encoder();

    let ep = Episode::new(EpisodeDraft {
        id: ("learn-1".to_string()).into(),
        intent: "initial approach".to_string(),
        intent_embedding: encoder.encode("initial approach"),
        experience: "Initial solution".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });
    store.store(ep)?;
    store.update_q("learn-1", 0.6);

    store.update_episode("learn-1", "improved approach", "success");

    let retrieved = store
        .get("learn-1")
        .ok_or_else(|| std::io::Error::other("learn-1 should exist after update"))?;
    assert_eq!(
        retrieved.experience, "improved approach",
        "Experience should be updated"
    );
    assert!(
        (retrieved.q_value - 0.6).abs() < 0.1,
        "Q-value should remain unchanged"
    );

    store.mark_accessed("learn-1");

    let ep_old = Episode::new(EpisodeDraft {
        id: ("old-1".to_string()).into(),
        intent: "deprecated".to_string(),
        intent_embedding: encoder.encode("deprecated"),
        experience: "Old".to_string(),
        outcome: "failure".to_string(),
        scope: None,
    });
    store.store(ep_old)?;
    store.delete_episode("old-1");

    assert!(
        store.get("old-1").is_none(),
        "Deleted episode should be gone"
    );

    println!("✓ Incremental learning: Update and delete work correctly");
    Ok(())
}
