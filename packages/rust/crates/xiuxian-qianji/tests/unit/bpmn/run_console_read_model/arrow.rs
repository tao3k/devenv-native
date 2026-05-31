use super::support::{
    assert_element_state, assert_float64_eq, float64_value, int32_value, record, run_id, step_id,
    string_value,
};
use crate::bpmn::qianji_run_console_arrow_read_model;
use xiuxian_qianji_control::{ControlEvent, ControlEventKind};

#[test]
fn qianji_run_console_arrow_read_model_matches_js_contract() {
    let run_id = run_id();
    let events = vec![
        record(
            1,
            ControlEvent::run(
                run_id.clone(),
                10,
                ControlEventKind::RunCreated {
                    intent: "operator start".to_owned(),
                    budget: None,
                    metadata: serde_json::Value::Null,
                },
            ),
        ),
        record(
            2,
            ControlEvent::step(
                run_id.clone(),
                step_id("start"),
                11,
                ControlEventKind::StepStarted,
            ),
        ),
        record(
            3,
            ControlEvent::step(
                run_id.clone(),
                step_id("start"),
                12,
                ControlEventKind::StepSucceeded,
            ),
        ),
        record(
            4,
            ControlEvent::step(
                run_id.clone(),
                step_id("resolve_project"),
                13,
                ControlEventKind::StepStarted,
            ),
        ),
    ];

    let read_model = qianji_run_console_arrow_read_model(&run_id, &events)
        .unwrap_or_else(|error| panic!("run-console read model should build: {error}"));

    assert_eq!(read_model.events.schema().field(0).name(), "runId");
    assert_eq!(read_model.events.schema().field(1).name(), "eventId");
    assert_eq!(read_model.events.schema().field(2).name(), "sequence");
    assert_eq!(read_model.events.schema().field(5).name(), "stepId");
    assert_eq!(
        read_model
            .events
            .schema()
            .metadata()
            .get("wendao.table")
            .map(String::as_str),
        Some("qianji.run_console.event.v1")
    );
    assert_eq!(read_model.events.num_rows(), 4);
    assert_eq!(
        string_value(&read_model.events, "runId", 0),
        run_id.as_str()
    );
    assert_eq!(string_value(&read_model.events, "eventId", 0), "1");
    assert_eq!(int32_value(&read_model.events, "sequence", 2), 3);
    assert_eq!(
        string_value(&read_model.events, "kind", 2),
        "step_succeeded"
    );
    assert_float64_eq(float64_value(&read_model.events, "occurredAtMs", 3), 13.0);

    assert_eq!(read_model.element_states.schema().field(0).name(), "runId");
    assert_eq!(
        read_model
            .element_states
            .schema()
            .metadata()
            .get("wendao.table")
            .map(String::as_str),
        Some("qianji.run_console.element_state.v1")
    );
    assert_element_state(&read_model.element_states, "start", "completed", "3");
    assert_element_state(&read_model.element_states, "resolve_project", "active", "4");
}

#[test]
fn qianji_run_console_element_state_rows_keep_latest_state_per_element() {
    let run_id = run_id();
    let events = vec![
        record(
            1,
            ControlEvent::step(
                run_id.clone(),
                step_id("review"),
                10,
                ControlEventKind::StepStarted,
            ),
        ),
        record(
            2,
            ControlEvent::step(
                run_id.clone(),
                step_id("review"),
                11,
                ControlEventKind::StepFailed {
                    error_code: "agent_failed".to_owned(),
                    message: "review failed".to_owned(),
                    retryable: true,
                },
            ),
        ),
    ];

    let read_model = qianji_run_console_arrow_read_model(&run_id, &events)
        .unwrap_or_else(|error| panic!("run-console read model should build: {error}"));

    assert_eq!(read_model.element_states.num_rows(), 1);
    assert_eq!(
        string_value(&read_model.element_states, "runId", 0),
        run_id.as_str()
    );
    assert_eq!(
        string_value(&read_model.element_states, "elementId", 0),
        "review"
    );
    assert_eq!(
        string_value(&read_model.element_states, "state", 0),
        "failed"
    );
    assert_eq!(
        string_value(&read_model.element_states, "sourceEventId", 0),
        "2"
    );
    assert_eq!(
        string_value(&read_model.element_states, "message", 0),
        "review failed"
    );
}

#[test]
fn qianji_run_console_element_state_rows_keep_terminal_state_over_replay_markers() {
    let run_id = run_id();
    let events = vec![
        record(
            1,
            ControlEvent::step(
                run_id.clone(),
                step_id("resolve_project"),
                10,
                ControlEventKind::StepStarted,
            ),
        ),
        record(
            2,
            ControlEvent::step(
                run_id.clone(),
                step_id("resolve_project"),
                11,
                ControlEventKind::StepSucceeded,
            ),
        ),
        record(
            3,
            ControlEvent::step(
                run_id.clone(),
                step_id("resolve_project"),
                12,
                ControlEventKind::StepQueued,
            ),
        ),
        record(
            4,
            ControlEvent::step(
                run_id.clone(),
                step_id("resolve_project"),
                13,
                ControlEventKind::StepStarted,
            ),
        ),
    ];

    let read_model = qianji_run_console_arrow_read_model(&run_id, &events)
        .unwrap_or_else(|error| panic!("run-console read model should build: {error}"));

    assert_element_state(
        &read_model.element_states,
        "resolve_project",
        "completed",
        "2",
    );
}
