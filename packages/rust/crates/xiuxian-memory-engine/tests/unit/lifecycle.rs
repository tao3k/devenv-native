//! Memory lifecycle contract tests.

use xiuxian_memory_engine::{
    MemoryLayer, MemoryRecallDefault, MemoryStatus, infer_memory_layer,
    infer_memory_lifecycle_facts_from_properties, infer_memory_recall_default, infer_memory_status,
};

#[test]
fn lifecycle_properties_map_to_layers_status_and_recall_default() {
    let facts = infer_memory_lifecycle_facts_from_properties(
        [
            ("MEMORY_LAYER", "long-term knowledge"),
            ("MEMORY_STATUS", "active"),
            ("RECALL_DEFAULT", "yes"),
        ]
        .iter()
        .copied(),
    );

    assert_eq!(facts.layer, MemoryLayer::Knowledge);
    assert_eq!(facts.status, MemoryStatus::Active);
    assert_eq!(facts.recall_default, MemoryRecallDefault::Yes);

    let decision = facts.evaluate();
    assert!(decision.projection_allowed);
    assert!(decision.default_recall_allowed);
    assert!(decision.scoped_recall_allowed);
    assert_eq!(decision.reason_code, "default_recall_allowed");
    assert!((decision.recall_prior - 0.94).abs() < f32::EPSILON);
}

#[test]
fn cache_layer_blocks_projection_and_recall() {
    let facts = infer_memory_lifecycle_facts_from_properties(
        [
            ("MEMORY_LAYER", "cache"),
            ("MEMORY_STATUS", "active"),
            ("RECALL_DEFAULT", "yes"),
        ]
        .iter()
        .copied(),
    );

    let decision = facts.evaluate();
    assert!(!decision.projection_allowed);
    assert!(!decision.default_recall_allowed);
    assert!(!decision.scoped_recall_allowed);
    assert_eq!(decision.recall_prior, 0.0);
    assert_eq!(decision.reason_code, "cache_not_memory");
}

#[test]
fn superseded_status_blocks_projection_even_when_recall_default_is_yes() {
    let facts = infer_memory_lifecycle_facts_from_properties(
        [
            ("MEMORY_LAYER", "episodic"),
            ("MEMORY_STATUS", "superseded"),
            ("RECALL_DEFAULT", "yes"),
        ]
        .iter()
        .copied(),
    );

    let decision = facts.evaluate();
    assert!(!decision.projection_allowed);
    assert!(!decision.default_recall_allowed);
    assert!(!decision.scoped_recall_allowed);
    assert_eq!(decision.reason_code, "status_blocks_projection");
}

#[test]
fn scoped_episodic_memory_projects_without_default_recall() {
    let facts = infer_memory_lifecycle_facts_from_properties(
        [
            ("MEMORY_LAYER", "episodic"),
            ("MEMORY_STATUS", "closed"),
            ("RECALL_DEFAULT", "scoped"),
        ]
        .iter()
        .copied(),
    );

    let decision = facts.evaluate();
    assert!(decision.projection_allowed);
    assert!(!decision.default_recall_allowed);
    assert!(decision.scoped_recall_allowed);
    assert_eq!(decision.reason_code, "scoped_recall_only");
    assert!((decision.recall_prior - (0.58 * 0.78 * 0.82)).abs() < f32::EPSILON);
}

#[test]
fn parser_accepts_common_lifecycle_aliases() {
    assert_eq!(
        infer_memory_layer("short term"),
        Some(MemoryLayer::Temporary)
    );
    assert_eq!(infer_memory_layer("deadline"), Some(MemoryLayer::Scheduled));
    assert_eq!(infer_memory_status("outdated"), Some(MemoryStatus::Expired));
    assert_eq!(
        infer_memory_recall_default("explicit"),
        Some(MemoryRecallDefault::Scoped)
    );
}
