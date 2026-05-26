//! Inferred memory object tests.

use xiuxian_memory_engine::{
    InferredMemoryObjectKind, infer_memory_object_from_property,
    infer_memory_object_from_reflection, infer_memory_object_kind_from_property_key,
    infer_memory_object_kind_from_question, infer_memory_objects_from_properties,
};

#[test]
fn reflection_questions_map_to_first_class_memory_kinds() {
    let cases = [
        (
            "What finality signal should future agents recall from this slice?",
            InferredMemoryObjectKind::Finality,
        ),
        (
            "Which claim became stronger, weaker, or superseded?",
            InferredMemoryObjectKind::Claim,
        ),
        (
            "Which evidence path proves the claim?",
            InferredMemoryObjectKind::Evidence,
        ),
        (
            "Which failure mode should future agents avoid?",
            InferredMemoryObjectKind::Failure,
        ),
        (
            "Which preference or naming correction should future generated plans preserve?",
            InferredMemoryObjectKind::Preference,
        ),
    ];

    for (question, expected) in cases {
        assert_eq!(
            infer_memory_object_kind_from_question(question),
            Some(expected),
            "question: {question}"
        );
    }
}

#[test]
fn reflection_inference_requires_answer_value() {
    assert!(infer_memory_object_from_reflection("Which claim changed?", "").is_none());
    assert!(infer_memory_object_from_reflection("", "The claim changed.").is_none());
}

#[test]
fn reflection_inference_preserves_question_and_value() {
    let Some(object) = infer_memory_object_from_reflection(
        "Which failure mode should future agents avoid?",
        "Do not add a redundant memory subcommand.",
    ) else {
        panic!("expected failure memory object");
    };

    assert_eq!(object.kind, InferredMemoryObjectKind::Failure);
    assert_eq!(
        object.kind.facet_label(),
        "memory-failure",
        "ranking facet label"
    );
    assert_eq!(object.kind.name(), "failure");
    assert_eq!(object.value, "Do not add a redundant memory subcommand.");
}

#[test]
fn property_keys_map_to_first_class_memory_kinds() {
    let cases = [
        ("OUTCOME", InferredMemoryObjectKind::Finality),
        ("TASK_OUTCOME", InferredMemoryObjectKind::Finality),
        ("CLAIM", InferredMemoryObjectKind::Claim),
        ("REUSABLE_KNOWLEDGE", InferredMemoryObjectKind::Claim),
        ("EVIDENCE_REF", InferredMemoryObjectKind::Evidence),
        ("REFERENCE", InferredMemoryObjectKind::Evidence),
        ("SYMPTOM", InferredMemoryObjectKind::Failure),
        ("FAILURE_NOTE", InferredMemoryObjectKind::Failure),
        ("REUSE_RULE", InferredMemoryObjectKind::Preference),
        ("PREFERENCE_SIGNAL", InferredMemoryObjectKind::Preference),
    ];

    for (key, expected) in cases {
        assert_eq!(
            infer_memory_object_kind_from_property_key(key),
            Some(expected),
            "property key: {key}"
        );
    }
}

#[test]
fn codex_reference_sample_fields_map_to_xiuxian_memory_object_kinds() {
    let objects = infer_memory_objects_from_properties(
        [
            ("TASK_OUTCOME", "success"),
            (
                "PREFERENCE_SIGNAL",
                "Prefer compact rendered snapshots over JSON-heavy agent output.",
            ),
            (
                "REUSABLE_KNOWLEDGE",
                "The serverless recall path uses Org, DuckDB, and compact session packets.",
            ),
            (
                "FAILURE_NOTE",
                "A runtime memory system must not read ~/.codex/memories directly.",
            ),
            (
                "REFERENCE",
                "rollout_summaries/2026-05-24T01-49-12-memory-reference.md",
            ),
        ]
        .iter()
        .copied(),
    );

    let kinds = objects.iter().map(|object| object.kind).collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            InferredMemoryObjectKind::Finality,
            InferredMemoryObjectKind::Preference,
            InferredMemoryObjectKind::Claim,
            InferredMemoryObjectKind::Failure,
            InferredMemoryObjectKind::Evidence,
        ]
    );
    assert_eq!(objects[0].question, "TASK_OUTCOME");
    assert_eq!(
        objects[3].value,
        "A runtime memory system must not read ~/.codex/memories directly."
    );
}

#[test]
fn property_inference_rejects_mismatched_evidence_values() {
    assert!(
        infer_memory_object_from_property("VALIDATION", "cargo test -p xiuxian-memory-engine")
            .is_none()
    );
    assert!(
        infer_memory_object_from_property("EVIDENCE_REF", "cargo test -p xiuxian-memory-engine")
            .is_none()
    );
    assert!(
        infer_memory_object_from_property(
            "EVIDENCE_REF",
            "packages/rust/crates/xiuxian-memory-engine/tests/unit/inference.rs",
        )
        .is_some()
    );
}

#[test]
fn property_inference_preserves_key_and_value() {
    let Some(object) = infer_memory_object_from_property(
        "CLAIM",
        "Org properties are graph-searchable memory evidence.",
    ) else {
        panic!("expected claim memory object");
    };

    assert_eq!(object.kind, InferredMemoryObjectKind::Claim);
    assert_eq!(object.question, "CLAIM");
    assert_eq!(
        object.value,
        "Org properties are graph-searchable memory evidence."
    );
}

#[test]
fn property_inference_ignores_non_memory_metadata() {
    let objects = infer_memory_objects_from_properties(
        [
            ("ID", "task-id"),
            ("ENTITY_REFS", "qianji-flowhub"),
            ("CLAIM", "Structured properties feed memory recall."),
            ("FIX", "Keep inference in xiuxian-memory-engine."),
        ]
        .iter()
        .copied(),
    );

    assert_eq!(objects.len(), 2);
    assert_eq!(objects[0].kind, InferredMemoryObjectKind::Claim);
    assert_eq!(objects[1].kind, InferredMemoryObjectKind::Failure);
}
