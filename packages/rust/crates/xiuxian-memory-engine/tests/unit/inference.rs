//! Inferred memory object tests.

use xiuxian_memory_engine::{
    InferredMemoryObjectKind, infer_memory_object_from_reflection,
    infer_memory_object_kind_from_question,
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
