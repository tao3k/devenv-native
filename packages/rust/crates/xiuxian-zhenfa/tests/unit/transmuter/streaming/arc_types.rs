use super::{ArcStreamingEvent, ArcStreamingOutcome, EventBuffer};

#[test]
fn arc_event_creates_thought() {
    let event = ArcStreamingEvent::thought("Test thought");
    assert!(matches!(event, ArcStreamingEvent::Thought(_)));
    assert_eq!(event.text_content(), Some("Test thought"));
}

#[test]
fn arc_event_estimates_size() {
    let event = ArcStreamingEvent::thought("Hello world");
    assert!(event.estimated_size() > 11);
}

#[test]
fn arc_event_helpers_cover_text_and_status_paths() {
    let text_delta = ArcStreamingEvent::text_delta("delta");
    assert!(matches!(text_delta, ArcStreamingEvent::TextDelta(_)));

    let status = ArcStreamingEvent::status("ready");
    assert_eq!(status.text_content(), Some("ready"));
}

#[test]
fn arc_event_helpers_detect_terminal_and_tool_states() {
    let terminal = ArcStreamingEvent::Error {
        code: std::sync::Arc::from("err"),
        message: std::sync::Arc::from("failed"),
    };
    assert!(terminal.is_terminal());

    let tool_event = ArcStreamingEvent::ToolCall {
        id: std::sync::Arc::from("tool-1"),
        name: std::sync::Arc::from("grep"),
        input: serde_json::Value::Null,
    };
    assert!(tool_event.is_tool_event());
}

#[test]
fn event_buffer_pushes_and_drains() {
    let mut buffer = EventBuffer::with_capacity(10);

    buffer.push(ArcStreamingEvent::thought("Event 1"));
    buffer.push(ArcStreamingEvent::thought("Event 2"));

    assert_eq!(buffer.len(), 2);
    assert!(!buffer.is_empty());

    let drained: Vec<_> = buffer.drain().collect();
    assert_eq!(drained.len(), 2);
    assert!(buffer.is_empty());
}

#[test]
fn event_buffer_flush_threshold() {
    let mut buffer = EventBuffer::new();
    buffer.set_max_size(100);

    buffer.push(ArcStreamingEvent::thought("Hi"));
    buffer.push(ArcStreamingEvent::thought(
        "This is a longer text that should push us over the threshold",
    ));

    assert!(
        buffer.total_size() >= 100,
        "Total size should be at least 100"
    );
    assert!(
        buffer.should_flush(),
        "Buffer should flush when size threshold exceeded"
    );

    buffer.clear();
    assert!(buffer.is_empty());
}

#[test]
fn arc_streaming_outcome_helpers_cover_success_and_failure() {
    let success = ArcStreamingOutcome::success("done");
    assert!(success.success);
    assert_eq!(success.final_text.as_ref(), "done");

    let failure = ArcStreamingOutcome::failure("boom");
    assert!(!failure.success);
    assert_eq!(failure.final_text.as_ref(), "boom");
}

#[test]
fn converts_from_standard_event() {
    let standard =
        crate::transmuter::streaming::ZhenfaStreamingEvent::Thought(std::sync::Arc::from("test"));
    let arc: ArcStreamingEvent = standard.into();

    assert_eq!(arc.text_content(), Some("test"));
}
