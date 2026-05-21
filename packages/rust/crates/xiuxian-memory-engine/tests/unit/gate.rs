//! Integration tests for 3-in-1 memory gate determinism and event shape.

use xiuxian_memory_engine::{
    Episode, EpisodeDraft, MemoryGateEvent, MemoryGateEventInput, MemoryGateMemoryId,
    MemoryGatePolicy, MemoryGateSessionId, MemoryGateTurnId, MemoryGateVerdict,
    MemoryLifecycleState, MemoryPromotionTarget, MemoryUtilityLedger,
};

fn episode_with_stats(
    id: &str,
    outcome: &str,
    q_value: f32,
    success_count: u32,
    failure_count: u32,
) -> Episode {
    let mut episode = Episode::new(EpisodeDraft {
        id: (id.to_string()).into(),
        intent: "intent".to_string(),
        intent_embedding: vec![0.1; 8],
        experience: "experience".to_string(),
        outcome: outcome.to_string(),
        scope: None,
    });
    episode.q_value = q_value;
    episode.retrieval_count = success_count + failure_count;
    episode.success_count = success_count;
    episode.failure_count = failure_count;
    episode
}

#[test]
fn gate_policy_promote_decision_is_deterministic() {
    let episode = episode_with_stats("mem-promote", "completed", 0.94, 8, 1);
    let ledger = MemoryUtilityLedger::from_episode(&episode, 0.95, 0.90, 0.92);
    let policy = MemoryGatePolicy::default();

    let decision_a = policy.evaluate(
        &ledger,
        vec!["react:validation:pass".to_string()],
        vec!["graph:path:plan->execute->verify".to_string()],
        vec!["omega:risk=low".to_string()],
    );
    let decision_b = policy.evaluate(
        &ledger,
        vec!["react:validation:pass".to_string()],
        vec!["graph:path:plan->execute->verify".to_string()],
        vec!["omega:risk=low".to_string()],
    );

    assert_eq!(decision_a, decision_b);
    assert_eq!(decision_a.verdict, MemoryGateVerdict::Promote);
    assert_eq!(decision_a.next_action, "promote");
    assert_eq!(
        decision_a.promotion_target,
        Some(MemoryPromotionTarget::WorkingKnowledge)
    );
    assert!(decision_a.confidence >= 0.55);
}

#[test]
fn gate_policy_obsolete_requires_threshold_and_min_usage() {
    let policy = MemoryGatePolicy::default();

    let bad_episode = episode_with_stats("mem-obsolete", "error", 0.12, 0, 6);
    let bad_ledger = MemoryUtilityLedger::from_episode(&bad_episode, 0.10, 0.18, 0.12);
    let bad_decision = policy.evaluate(&bad_ledger, vec![], vec![], vec![]);
    assert_eq!(bad_decision.verdict, MemoryGateVerdict::Obsolete);
    assert_eq!(bad_decision.next_action, "obsolete");

    // Same low quality signals but below min usage should not be purged.
    let fresh_episode = episode_with_stats("mem-fresh", "error", 0.12, 0, 1);
    let fresh_ledger = MemoryUtilityLedger::from_episode(&fresh_episode, 0.10, 0.18, 0.12);
    let fresh_decision = policy.evaluate(&fresh_ledger, vec![], vec![], vec![]);
    assert_eq!(fresh_decision.verdict, MemoryGateVerdict::Retain);
    assert_eq!(fresh_decision.promotion_target, None);
}

#[test]
fn gate_event_matches_contract_shape() {
    let episode = episode_with_stats("mem-event", "completed", 0.91, 7, 1);
    let ledger = MemoryUtilityLedger::from_episode(&episode, 0.92, 0.88, 0.90);
    let policy = MemoryGatePolicy::default();
    let decision = policy.evaluate(
        &ledger,
        vec!["react:ref:1".to_string()],
        vec!["graph:ref:1".to_string()],
        vec!["omega:factor:1".to_string()],
    );
    let event = MemoryGateEvent::from_decision(MemoryGateEventInput {
        session_id: MemoryGateSessionId::from("telegram:1304799691:1304799691"),
        turn_id: MemoryGateTurnId::from(42),
        memory_id: MemoryGateMemoryId::from(episode.id.as_str()),
        ledger: ledger.clone(),
        decision,
    });

    let value = serde_json::to_value(&event).unwrap_or_else(|error| {
        panic!("serialize memory gate event: {error}");
    });
    assert_eq!(value["session_id"], "telegram:1304799691:1304799691");
    assert_eq!(value["turn_id"], 42);
    assert_eq!(value["state_before"], "active");
    assert!(matches!(
        event.state_after,
        MemoryLifecycleState::Active
            | MemoryLifecycleState::Purged
            | MemoryLifecycleState::Promoted
    ));
    assert!(value["decision"]["next_action"].is_string());
    assert_eq!(value["decision"]["promotion_target"], "working_knowledge");
}
