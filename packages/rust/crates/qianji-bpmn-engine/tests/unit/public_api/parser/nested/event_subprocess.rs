use super::super::super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEngineError, BpmnParseOptions, parse_bpmn_package};

#[test]
fn parser_event_subprocess_is_rejected() {
    let error = parse_bpmn_package(
        &[fixture_source("invalid-compensation-event-subprocess.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must_err("event subprocesses should stay deferred");

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedSubProcessConfiguration {
            process_id: "main_process".to_string(),
            node_id: "comp_handler".to_string(),
            detail: "event_subprocess",
        }
    );
}
