use std::sync::Arc;

use crate::transmuter::streaming::ZhenfaStreamingEvent;
use crate::transmuter::streaming::supervisor::{
    CognitiveDimension, CognitiveSupervisor, MAX_HISTORY_SIZE, ThoughtSubcategory,
};

#[test]
fn supervisor_classifies_planning_as_meta() {
    let mut supervisor = CognitiveSupervisor::new();
    let event =
        ZhenfaStreamingEvent::Thought(Arc::from("Let me plan my approach to this problem."));

    let classified = supervisor.classify(event);
    assert_eq!(classified.dimension, CognitiveDimension::Meta);
    assert_eq!(classified.subcategory, Some(ThoughtSubcategory::Planning));
}

#[test]
fn supervisor_classifies_code_analysis_as_operational() {
    let mut supervisor = CognitiveSupervisor::new();
    let event =
        ZhenfaStreamingEvent::Thought(Arc::from("This function handles the validation logic."));

    let classified = supervisor.classify(event);
    assert_eq!(classified.dimension, CognitiveDimension::Operational);
    assert_eq!(
        classified.subcategory,
        Some(ThoughtSubcategory::CodeAnalysis)
    );
}

#[test]
fn supervisor_classifies_tool_call_as_instrumental() {
    let mut supervisor = CognitiveSupervisor::new();
    let event = ZhenfaStreamingEvent::ToolCall {
        id: Arc::from("call_1"),
        name: Arc::from("read_file"),
        input: serde_json::Value::Null,
    };

    let classified = supervisor.classify(event);
    assert_eq!(classified.dimension, CognitiveDimension::Instrumental);
}

#[test]
fn supervisor_tracks_cognitive_balance() {
    let mut supervisor = CognitiveSupervisor::new();

    let _ = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from("Let me plan...")));
    let _ = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from(
        "I should reconsider...",
    )));
    let _ = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from(
        "This code needs...",
    )));
    let _ = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from(
        "I'll add a function...",
    )));

    let balance = supervisor.cognitive_balance();
    assert!(balance > 0.0 && balance < 1.0);
}

#[test]
fn supervisor_context_updates_on_classification() {
    let mut supervisor = CognitiveSupervisor::new();
    let _ = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from(
        "I'll implement this feature.",
    )));

    assert!(supervisor.context().in_implementation);
    assert_eq!(supervisor.context().operational_streak, 1);
}

#[test]
fn supervisor_classifies_uncertainty_as_epistemic() {
    let mut supervisor = CognitiveSupervisor::new();
    let event =
        ZhenfaStreamingEvent::Thought(Arc::from("I'm not sure if this approach is correct."));

    let classified = supervisor.classify(event);
    assert_eq!(classified.dimension, CognitiveDimension::Epistemic);
    assert_eq!(
        classified.subcategory,
        Some(ThoughtSubcategory::Uncertainty)
    );
}

#[test]
fn supervisor_classifies_knowledge_gap_as_epistemic() {
    let mut supervisor = CognitiveSupervisor::new();
    let event = ZhenfaStreamingEvent::Thought(Arc::from("I need more context about this API."));

    let classified = supervisor.classify(event);
    assert_eq!(classified.dimension, CognitiveDimension::Epistemic);
    assert_eq!(
        classified.subcategory,
        Some(ThoughtSubcategory::KnowledgeGap)
    );
}

#[test]
fn supervisor_calculates_coherence_score() {
    let mut supervisor = CognitiveSupervisor::new();

    let event = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from(
        "I'll add a function here.",
    )));
    assert!(event.coherence > 0.7);

    let event = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from(
        "I'm not sure about this.",
    )));
    assert!(event.coherence < 0.8);
}

#[test]
fn supervisor_triggers_early_halt_on_low_coherence() {
    let mut supervisor = CognitiveSupervisor::with_early_halt_threshold(0.5);

    for _ in 0..5 {
        let _ = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from(
            "I'm not sure about this approach.",
        )));
    }

    assert!(supervisor.should_halt());
}

#[test]
fn supervisor_detects_oscillating_behavior() {
    let mut supervisor = CognitiveSupervisor::new();

    let _ = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from("Let me plan...")));
    let _ = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from("This function...")));
    let _ = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from(
        "I should reconsider...",
    )));
    let event = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from("I'll add code...")));

    assert!(event.coherence < 0.75);
}

#[test]
fn supervisor_history_respects_max_size() {
    let mut supervisor = CognitiveSupervisor::new();

    for i in 0..150 {
        let _ = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from(format!(
            "Thought {i}"
        ))));
    }

    assert!(supervisor.history_len() <= MAX_HISTORY_SIZE);
}

#[test]
fn supervisor_history_maintains_recent_entries() {
    let mut supervisor = CognitiveSupervisor::new();

    for i in 0..150 {
        let _ = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from(format!(
            "Thought {i}"
        ))));
    }

    let history = supervisor.history_slice(0, 10);
    assert!(history.len() <= 10);
}

#[test]
fn supervisor_reset_clears_vecdeque() {
    let mut supervisor = CognitiveSupervisor::new();

    for i in 0..50 {
        let _ = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from(format!(
            "Thought {i}"
        ))));
    }

    assert!(supervisor.history_len() > 0);

    supervisor.reset();

    assert_eq!(supervisor.history_len(), 0);
    assert!(!supervisor.coherence().early_halt_triggered);
}

#[test]
fn supervisor_early_halt_threshold_customizable() {
    let mut supervisor = CognitiveSupervisor::with_early_halt_threshold(0.5);

    for _ in 0..6 {
        let _ = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from(
            "I'm uncertain about this.",
        )));
    }

    assert!(supervisor.should_halt());
}

#[test]
fn supervisor_self_correction_increments_counter() {
    let mut supervisor = CognitiveSupervisor::new();

    let _ = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from("I'll do X first.")));
    let _ = supervisor.classify(ZhenfaStreamingEvent::Thought(Arc::from(
        "Wait, let me reconsider that.",
    )));

    assert!(supervisor.coherence().self_correction_count > 0);
}
