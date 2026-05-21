use super::{Episode, EpisodeDraft, TestResult, test_store, test_store_with_dim};

#[test]
fn test_self_evolution_from_feedback() -> TestResult {
    let store = test_store("test");
    let encoder = store.encoder();

    let ep1 = Episode::new(EpisodeDraft {
        id: ("ep-1".to_string()).into(),
        intent: "fix network timeout".to_string(),
        intent_embedding: encoder.encode("fix network timeout"),
        experience: "Increased timeout to 30s".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });
    store.store(ep1.clone())?;

    let initial_q = store.q_table.get_q("ep-1");
    assert!(
        (initial_q - 0.5).abs() < 0.01,
        "Initial Q-value should be 0.5"
    );

    store.update_q("ep-1", 1.0);
    let after_success_q = store.q_table.get_q("ep-1");
    assert!(
        after_success_q > 0.5,
        "Q-value should increase after success"
    );

    let ep2 = Episode::new(EpisodeDraft {
        id: ("ep-2".to_string()).into(),
        intent: "debug api crash".to_string(),
        intent_embedding: encoder.encode("debug api crash"),
        experience: "Restarted service".to_string(),
        outcome: "failure".to_string(),
        scope: None,
    });
    store.store(ep2)?;

    store.update_q("ep-2", 0.0);
    let after_failure_q = store.q_table.get_q("ep-2");
    assert!(
        after_failure_q < 0.5,
        "Q-value should decrease after failure"
    );

    println!("✓ Self-evolution: Q-values adapted based on feedback");
    println!("  - Success episode Q: {initial_q} → {after_success_q}");
    println!("  - Failure episode Q: {} → {}", 0.5, after_failure_q);
    Ok(())
}

#[test]
fn test_two_phase_noise_reduction() -> TestResult {
    let store = test_store("test");
    let encoder = store.encoder();

    let intents = [
        ("fix database connection", "Restarted DB", "success"),
        (
            "fix database connection",
            "Changed connection string",
            "success",
        ),
        ("fix database connection", "Increased pool size", "success"),
        ("fix database connection", "Reinstalled driver", "failure"),
        ("fix database connection", "Cleared cache", "failure"),
        ("fix database connection", "Rebooted server", "failure"),
    ];

    for (i, (intent, exp, outcome)) in intents.iter().enumerate() {
        let ep = Episode::new(EpisodeDraft {
            id: (format!("ep-{i}")).into(),
            intent: intent.to_string(),
            intent_embedding: encoder.encode(intent),
            experience: exp.to_string(),
            outcome: outcome.to_string(),
            scope: None,
        });
        store.store(ep)?;
    }

    for i in 0..3 {
        store.update_q(format!("ep-{i}"), 1.0);
    }
    for i in 3..6 {
        store.update_q(format!("ep-{i}"), 0.0);
    }

    let query_emb = encoder.encode("database connection error");
    let phase1 = store.recall_with_embedding(&query_emb, 10);
    let phase2 = store.two_phase_recall_with_embedding(&query_emb, 10, 3, 0.5);
    let phase2_successes: usize = phase2.iter().filter(|(ep, _)| ep.q_value > 0.5).count();

    println!("✓ Two-phase noise reduction:");
    println!("  - Phase 1 (semantic): {} results", phase1.len());
    println!("  - Phase 2 (with Q-rerank): {} results", phase2.len());
    println!("  - High-utility in top-3: {phase2_successes}/3");

    assert!(
        phase2_successes >= 2,
        "Two-phase should return mostly successful experiences"
    );
    Ok(())
}

#[test]
fn test_q_learning_convergence_scenario() -> TestResult {
    let store = test_store("test");
    let encoder = store.encoder();

    let ep = Episode::new(EpisodeDraft {
        id: ("converge-test".to_string()).into(),
        intent: "test task".to_string(),
        intent_embedding: encoder.encode("test task"),
        experience: "Did X".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });
    store.store(ep)?;

    let mut q_values = vec![0.5];
    for _ in 0..20 {
        store.update_q("converge-test", 1.0);
        q_values.push(store.q_table.get_q("converge-test"));
    }

    let final_q = *q_values
        .last()
        .ok_or_else(|| std::io::Error::other("q-values should contain at least one value"))?;

    println!("✓ Q-learning convergence:");
    println!("  - Initial Q: 0.5");
    println!("  - After 20 success updates: {final_q:.4}");
    println!("  - Converged towards 1.0: {}", final_q > 0.9);

    assert!(
        final_q > 0.9,
        "Q-value should converge towards reward (1.0)"
    );
    Ok(())
}

#[test]
fn test_conflicting_experiences() -> TestResult {
    let store = test_store("test");
    let encoder = store.encoder();
    let intent = "fix critical bug";

    let ep1 = Episode::new(EpisodeDraft {
        id: ("fix-1".to_string()).into(),
        intent: intent.to_string(),
        intent_embedding: encoder.encode(intent),
        experience: "Solution A worked".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });
    let ep2 = Episode::new(EpisodeDraft {
        id: ("fix-2".to_string()).into(),
        intent: intent.to_string(),
        intent_embedding: encoder.encode(intent),
        experience: "Solution B failed".to_string(),
        outcome: "failure".to_string(),
        scope: None,
    });
    let ep3 = Episode::new(EpisodeDraft {
        id: ("fix-3".to_string()).into(),
        intent: intent.to_string(),
        intent_embedding: encoder.encode(intent),
        experience: "Solution C worked better".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });

    store.store(ep1)?;
    store.store(ep2)?;
    store.store(ep3)?;

    store.update_q("fix-1", 1.0);
    store.update_q("fix-2", 0.0);
    store.update_q("fix-3", 1.0);

    let query_emb = encoder.encode(intent);
    let results = store.two_phase_recall_with_embedding(&query_emb, 3, 3, 0.8);
    let top_episode_q = results[0].0.q_value;

    println!("✓ Conflicting experiences handling:");
    println!("  - Stored 3 experiences for same intent");
    println!("  - Updated with rewards: fix-1=1.0, fix-2=0.0, fix-3=1.0");
    println!(
        "  - Two-phase top result: {} with q_value={:.2}",
        results[0].0.id, top_episode_q
    );

    assert!(
        top_episode_q >= 0.5,
        "Should prefer successful experience, got {top_episode_q}"
    );
    Ok(())
}

#[test]
fn test_utility_similarity_tradeoff() -> TestResult {
    let store = test_store_with_dim("test", 4);

    let ep1 = Episode::new(EpisodeDraft {
        id: ("high-sim-low-q".to_string()).into(),
        intent: "debug api timeout".to_string(),
        intent_embedding: vec![0.95, 0.05, 0.0, 0.0],
        experience: "Old failed fix".to_string(),
        outcome: "failure".to_string(),
        scope: None,
    });
    let ep2 = Episode::new(EpisodeDraft {
        id: ("low-sim-high-q".to_string()).into(),
        intent: "fix database connection pool".to_string(),
        intent_embedding: vec![0.0, 1.0, 0.0, 0.0],
        experience: "New successful fix".to_string(),
        outcome: "success".to_string(),
        scope: None,
    });

    store.store(ep1)?;
    store.store(ep2)?;
    store.update_q("high-sim-low-q", 0.1);
    store.update_q("low-sim-high-q", 0.95);

    let query = vec![1.0, 0.0, 0.0, 0.0];
    let results_lambda_0 = store.two_phase_recall_with_embedding(&query, 2, 2, 0.0);
    let results_lambda_1 = store.two_phase_recall_with_embedding(&query, 2, 2, 1.0);
    let results_lambda_5 = store.two_phase_recall_with_embedding(&query, 2, 2, 0.5);

    println!("✓ Utility vs Similarity trade-off:");
    println!(
        "  - λ=0 (similarity only): {:?}",
        results_lambda_0
            .iter()
            .map(|(e, _)| e.id.as_str())
            .collect::<Vec<_>>()
    );
    println!(
        "  - λ=0.5 (balanced): {:?}",
        results_lambda_5
            .iter()
            .map(|(e, _)| e.id.as_str())
            .collect::<Vec<_>>()
    );
    println!(
        "  - λ=1 (Q only): {:?}",
        results_lambda_1
            .iter()
            .map(|(e, _)| e.id.as_str())
            .collect::<Vec<_>>()
    );

    assert_eq!(
        results_lambda_0[0].0.id, "high-sim-low-q",
        "λ=0 should prefer similarity"
    );
    assert_eq!(
        results_lambda_1[0].0.id, "low-sim-high-q",
        "λ=1 should prefer Q-value"
    );
    Ok(())
}
