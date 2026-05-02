use super::{ThoughtAggregator, ZhenfaStreamingEvent};
use serde_json::json;
use std::sync::Arc;

#[test]
fn thought_aggregator_creates_trace_with_intent() {
    let aggregator = ThoughtAggregator::new(
        Some("session-123".to_string()),
        "AuditNode".to_string(),
        "Critique the agenda".to_string(),
    );

    let trace = aggregator.build();
    assert!(trace.trace_id.starts_with("trace-AuditNode-"));
    assert_eq!(trace.session_id, Some("session-123".to_string()));
    assert_eq!(trace.node_id, "AuditNode");
    assert_eq!(trace.intent, "Critique the agenda");
}

#[test]
fn thought_aggregator_processes_thought_events() {
    let mut aggregator =
        ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

    aggregator.process_event(&ZhenfaStreamingEvent::Thought(Arc::<str>::from(
        "Thinking...",
    )));
    aggregator.process_event(&ZhenfaStreamingEvent::TextDelta(Arc::<str>::from(
        "Output text",
    )));

    let trace = aggregator.build();
    assert!(trace.reasoning.contains("[THOUGHT] Thinking..."));
    assert!(trace.reasoning.contains("Output text"));
}

#[test]
fn thought_aggregator_processes_tool_calls() {
    let mut aggregator =
        ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

    aggregator.process_event(&ZhenfaStreamingEvent::ToolCall {
        id: Arc::<str>::from("call-1"),
        name: Arc::<str>::from("search"),
        input: json!({"query": "test"}),
    });
    aggregator.process_event(&ZhenfaStreamingEvent::ToolResult {
        id: Arc::<str>::from("call-1"),
        output: json!({"results": []}),
    });

    assert_eq!(aggregator.tool_calls.len(), 1);
    assert_eq!(aggregator.tool_calls[0].name, "search");
    assert_eq!(aggregator.tool_calls[0].input["query"], "test");
    assert!(aggregator.tool_calls[0].output.is_some());
}

#[test]
fn thought_aggregator_records_outcome() {
    let mut aggregator =
        ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

    aggregator.set_outcome("Task completed successfully".to_string());

    let trace = aggregator.build();
    assert_eq!(
        trace.outcome,
        Some(Arc::<str>::from("Task completed successfully"))
    );
}

#[test]
fn thought_aggregator_tracks_coherence() {
    let mut aggregator =
        ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

    aggregator.set_coherence_score(0.85);
    aggregator.set_early_halt();

    let trace = aggregator.build();
    assert_eq!(trace.coherence_score, Some(0.85));
    assert!(trace.early_halt_triggered);
}

#[test]
fn thought_aggregator_reasoning_length() {
    let mut aggregator =
        ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

    assert!(aggregator.is_empty());
    assert_eq!(aggregator.reasoning_length(), 0);

    aggregator.process_event(&ZhenfaStreamingEvent::TextDelta(Arc::<str>::from("Hello")));

    assert!(!aggregator.is_empty());
    assert_eq!(aggregator.reasoning_length(), 5);
}

#[test]
fn thought_aggregator_processes_status_events() {
    let mut aggregator =
        ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

    aggregator.process_event(&ZhenfaStreamingEvent::Status(Arc::<str>::from(
        "Scanning files...",
    )));

    let trace = aggregator.build();
    assert!(trace.reasoning.contains("[STATUS] Scanning files..."));
}

#[test]
fn thought_aggregator_processes_progress_events() {
    let mut aggregator =
        ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

    aggregator.process_event(&ZhenfaStreamingEvent::Progress {
        message: Arc::<str>::from("Processing"),
        percent: 50,
    });

    let trace = aggregator.build();
    assert!(trace.reasoning.contains("[PROGRESS 50%] Processing"));
}

#[test]
fn thought_aggregator_processes_error_events() {
    let mut aggregator =
        ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

    aggregator.process_event(&ZhenfaStreamingEvent::Error {
        code: Arc::<str>::from("E001"),
        message: Arc::<str>::from("Something went wrong"),
    });

    let trace = aggregator.build();
    assert!(
        trace
            .reasoning
            .contains("[ERROR E001] Something went wrong")
    );
}

#[test]
fn thought_aggregator_processes_finished_event() {
    let mut aggregator =
        ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

    let outcome = xiuxian_zhenfa::StreamingOutcome {
        success: true,
        tokens_used: Some(xiuxian_zhenfa::TokenUsage {
            input: 50,
            output: 50,
            total: 100,
        }),
        final_text: Arc::<str>::from("Final result text"),
        tool_calls: Vec::new(),
        exit_code: None,
    };
    aggregator.process_event(&ZhenfaStreamingEvent::Finished(outcome));

    let trace = aggregator.build();
    assert_eq!(trace.outcome, Some(Arc::<str>::from("Final result text")));
}

#[test]
fn thought_aggregator_multiple_tool_calls_match_results() {
    let mut aggregator =
        ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

    aggregator.process_event(&ZhenfaStreamingEvent::ToolCall {
        id: Arc::<str>::from("call-1"),
        name: Arc::<str>::from("search"),
        input: json!({"query": "test1"}),
    });
    aggregator.process_event(&ZhenfaStreamingEvent::ToolCall {
        id: Arc::<str>::from("call-2"),
        name: Arc::<str>::from("read"),
        input: json!({"path": "test.md"}),
    });
    aggregator.process_event(&ZhenfaStreamingEvent::ToolResult {
        id: Arc::<str>::from("call-2"),
        output: json!({"content": "file content"}),
    });
    aggregator.process_event(&ZhenfaStreamingEvent::ToolResult {
        id: Arc::<str>::from("call-1"),
        output: json!({"results": ["a", "b"]}),
    });

    assert_eq!(aggregator.tool_calls.len(), 2);
    assert_eq!(aggregator.tool_calls[0].name, "search");
    assert_eq!(aggregator.tool_calls[1].name, "read");
    assert!(aggregator.tool_calls[0].output.is_some());
    assert!(aggregator.tool_calls[1].output.is_some());
}

#[test]
fn thought_aggregator_builds_complete_trace() {
    let mut aggregator = ThoughtAggregator::new(
        Some("session-complete".to_string()),
        "CompleteNode".to_string(),
        "Complete workflow".to_string(),
    );

    aggregator.process_event(&ZhenfaStreamingEvent::Thought(Arc::<str>::from(
        "Planning...",
    )));
    aggregator.process_event(&ZhenfaStreamingEvent::TextDelta(Arc::<str>::from("Step 1")));
    aggregator.process_event(&ZhenfaStreamingEvent::ToolCall {
        id: Arc::<str>::from("call-1"),
        name: Arc::<str>::from("execute"),
        input: json!({"cmd": "test"}),
    });
    aggregator.process_event(&ZhenfaStreamingEvent::ToolResult {
        id: Arc::<str>::from("call-1"),
        output: json!({"success": true}),
    });
    aggregator.set_coherence_score(0.92);
    aggregator.set_outcome("Workflow completed".to_string());

    let trace = aggregator.build();

    assert!(trace.trace_id.starts_with("trace-CompleteNode-"));
    assert_eq!(trace.session_id, Some("session-complete".to_string()));
    assert_eq!(trace.node_id, "CompleteNode");
    assert_eq!(trace.intent, "Complete workflow");
    assert!(trace.reasoning.contains("[THOUGHT] Planning..."));
    assert!(trace.reasoning.contains("Step 1"));
    assert_eq!(trace.outcome, Some(Arc::<str>::from("Workflow completed")));
    assert_eq!(trace.coherence_score, Some(0.92));
    assert!(!trace.early_halt_triggered);
}

#[test]
fn thought_aggregator_empty_trace_still_builds() {
    let aggregator =
        ThoughtAggregator::new(None, "EmptyNode".to_string(), "Empty intent".to_string());

    let trace = aggregator.build();

    assert!(trace.trace_id.starts_with("trace-EmptyNode-"));
    assert!(trace.reasoning.is_empty());
    assert!(trace.outcome.is_none());
}

#[test]
fn thought_aggregator_clone_preserves_state() {
    let mut aggregator =
        ThoughtAggregator::new(None, "CloneNode".to_string(), "Clone intent".to_string());

    aggregator.process_event(&ZhenfaStreamingEvent::TextDelta(Arc::<str>::from(
        "Content",
    )));
    aggregator.set_coherence_score(0.75);

    let cloned = aggregator.clone();

    assert_eq!(cloned.node_id, "CloneNode");
    assert_eq!(cloned.intent, "Clone intent");
    assert_eq!(cloned.reasoning_length(), 7);
    assert_eq!(cloned.coherence_score, Some(0.75));
}
