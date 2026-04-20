use super::{parse_fixture_error, parse_fixture_package};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEngineError, BpmnEventKind, BpmnGatewayKind, BpmnNodeKind};

#[test]
fn parser_supported_parallel_gateway_materializes_gateway_kinds() {
    let package = parse_fixture_package("parallel-gateway-join.bpmn");
    let process = package
        .find_process("parallel_flow")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::Gateway);
    assert_eq!(
        process.nodes[1].gateway_kind,
        Some(BpmnGatewayKind::Parallel)
    );
    assert_eq!(process.nodes[2].kind, BpmnNodeKind::Gateway);
    assert_eq!(
        process.nodes[2].gateway_kind,
        Some(BpmnGatewayKind::Exclusive)
    );
    assert_eq!(process.nodes[3].kind, BpmnNodeKind::Gateway);
    assert_eq!(
        process.nodes[3].gateway_kind,
        Some(BpmnGatewayKind::Exclusive)
    );
    assert_eq!(process.nodes[4].kind, BpmnNodeKind::Gateway);
    assert_eq!(
        process.nodes[4].gateway_kind,
        Some(BpmnGatewayKind::Parallel)
    );
}

#[test]
fn parser_event_based_gateway_materializes_gateway_kind_and_wait_targets() {
    let package = parse_fixture_package("event-based-gateway-basic.bpmn");
    let process = package
        .find_process("event_race")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::Gateway);
    assert_eq!(
        process.nodes[1].gateway_kind,
        Some(BpmnGatewayKind::EventBased)
    );
    assert_eq!(process.outgoing_edge_indices(1), [1, 2]);
    assert_eq!(process.nodes[2].kind, BpmnNodeKind::IntermediateCatchEvent);
    assert_eq!(process.nodes[3].kind, BpmnNodeKind::IntermediateCatchEvent);
    assert_eq!(
        process
            .event_for_node(2)
            .must("message wait should exist")
            .kind,
        BpmnEventKind::Message
    );
    assert_eq!(
        process
            .event_for_node(3)
            .must("timer wait should exist")
            .kind,
        BpmnEventKind::Timer
    );
}

#[test]
fn parser_event_based_gateway_requires_wait_targets() {
    let error = parse_fixture_error(
        "invalid-event-based-gateway-task-target.bpmn",
        "event-based gateway should only target wait events in the bounded slice",
    );

    assert_eq!(
        error,
        BpmnEngineError::UnsupportedEventBasedGatewayConfiguration {
            process_id: "event_race_invalid".to_string(),
            node_id: "wait_race".to_string(),
            detail: "unsupported_wait_target_kind",
        }
    );
}

#[test]
fn parser_unsupported_inclusive_gateway_is_rejected() {
    let error = parse_fixture_error(
        "invalid-unsupported-gateway.bpmn",
        "unsupported BPMN elements should fail explicitly",
    );
    assert_eq!(
        error,
        BpmnEngineError::UnsupportedElement {
            source_id: "invalid-unsupported-gateway.bpmn".to_string(),
            process_id: "gateway_flow".to_string(),
            element: "inclusiveGateway".to_string(),
        }
    );
}
