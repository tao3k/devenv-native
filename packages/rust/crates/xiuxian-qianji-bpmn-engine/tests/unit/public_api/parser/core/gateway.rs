use super::{parse_fixture_error, parse_fixture_package};
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{BpmnEngineError, BpmnEventKind, BpmnGatewayKind, BpmnNodeKind};

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
fn parser_event_based_gateway_accepts_conditional_wait_target() {
    let package = parse_fixture_package("event-based-gateway-conditional.bpmn");
    let process = package
        .find_process("event_race_conditional")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::Gateway);
    assert_eq!(
        process.nodes[1].gateway_kind,
        Some(BpmnGatewayKind::EventBased)
    );
    assert_eq!(process.outgoing_edge_indices(1), [1, 2]);
    assert_eq!(
        process
            .event_for_node(2)
            .must("message wait should exist")
            .kind,
        BpmnEventKind::Message
    );
    let conditional_event = process
        .event_for_node(3)
        .must("conditional wait should exist");
    assert_eq!(conditional_event.kind, BpmnEventKind::Conditional);
    assert_eq!(
        conditional_event.condition_expression.as_deref(),
        Some("approved")
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
            process_id: ("event_race_invalid".to_string()).into(),
            node_id: ("wait_race".to_string()).into(),
            detail: "unsupported_wait_target_kind",
        }
    );
}

#[test]
fn parser_complex_gateway_materializes_gateway_kind() {
    let package = parse_fixture_package("invalid-unsupported-gateway.bpmn");
    let process = package
        .find_process("gateway_flow")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::Gateway);
    assert_eq!(
        process.nodes[1].gateway_kind,
        Some(BpmnGatewayKind::Complex)
    );
    assert_eq!(process.outgoing_edge_indices(1), [1]);
}

#[test]
fn parser_structured_inclusive_gateway_materializes_join_metadata() {
    let package = parse_fixture_package("inclusive-gateway-structured.bpmn");
    let process = package
        .find_process("inclusive_gateway_structured")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::Gateway);
    assert_eq!(
        process.nodes[1].gateway_kind,
        Some(BpmnGatewayKind::Inclusive)
    );
    assert_eq!(process.nodes[1].default_outgoing_edge, Some(3));
    assert_eq!(process.nodes[1].inclusive_join_node, Some(5));
    assert_eq!(
        process.edges[1].condition_expression.as_deref(),
        Some("approved")
    );
    assert_eq!(
        process.edges[2].condition_expression.as_deref(),
        Some("vip")
    );
    assert_eq!(process.edges[3].condition_expression, None);
    assert_eq!(
        process.nodes[5].gateway_kind,
        Some(BpmnGatewayKind::Inclusive)
    );
}

#[test]
fn parser_invalid_structured_inclusive_gateway_is_rejected() {
    let error = parse_fixture_error(
        "invalid-inclusive-gateway-branch-end-before-join.bpmn",
        "inclusive branches that end before the structured join should stay explicit",
    );
    assert_eq!(
        error,
        BpmnEngineError::UnsupportedGatewayConfiguration {
            process_id: ("inclusive_gateway_invalid".to_string()).into(),
            node_id: ("decision".to_string()).into(),
            detail: "inclusive_split_branch_ends_before_join",
        }
    );
}

#[test]
fn parser_structured_inclusive_gateway_numeric_conditions_materialize() {
    let package = parse_fixture_package("inclusive-gateway-structured-numeric.bpmn");
    let process = package
        .find_process("inclusive_gateway_numeric")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::Gateway);
    assert_eq!(
        process.nodes[1].gateway_kind,
        Some(BpmnGatewayKind::Inclusive)
    );
    assert_eq!(process.nodes[1].default_outgoing_edge, Some(3));
    assert_eq!(process.nodes[1].inclusive_join_node, Some(5));
    assert_eq!(
        process.edges[1].condition_expression.as_deref(),
        Some("amount > 100")
    );
    assert_eq!(
        process.edges[2].condition_expression.as_deref(),
        Some("risk >= 7")
    );
    assert_eq!(process.edges[3].condition_expression, None);
}

#[test]
fn parser_exclusive_gateway_conditions_and_default_flow_materialize() {
    let package = parse_fixture_package("exclusive-gateway-conditions-default.bpmn");
    let process = package
        .find_process("exclusive_gateway_conditions")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::Gateway);
    assert_eq!(
        process.nodes[1].gateway_kind,
        Some(BpmnGatewayKind::Exclusive)
    );
    assert_eq!(process.nodes[1].default_outgoing_edge, Some(3));
    assert_eq!(
        process.edges[1].condition_expression.as_deref(),
        Some("approved")
    );
    assert_eq!(
        process.edges[2].condition_expression.as_deref(),
        Some("vip")
    );
    assert_eq!(process.edges[3].condition_expression, None);
}

#[test]
fn parser_exclusive_gateway_numeric_conditions_materialize() {
    let package = parse_fixture_package("exclusive-gateway-conditions-numeric.bpmn");
    let process = package
        .find_process("exclusive_gateway_numeric_conditions")
        .must("process should be present");

    assert_eq!(process.nodes[1].kind, BpmnNodeKind::Gateway);
    assert_eq!(
        process.nodes[1].gateway_kind,
        Some(BpmnGatewayKind::Exclusive)
    );
    assert_eq!(process.nodes[1].default_outgoing_edge, Some(3));
    assert_eq!(
        process.edges[1].condition_expression.as_deref(),
        Some("amount > 100")
    );
    assert_eq!(
        process.edges[2].condition_expression.as_deref(),
        Some("risk >= 7")
    );
    assert_eq!(process.edges[3].condition_expression, None);
}

#[test]
fn parser_exclusive_gateway_unsupported_condition_is_rejected() {
    let error = parse_fixture_error(
        "invalid-exclusive-gateway-unsupported-condition.bpmn",
        "unsupported exclusive-gateway conditions should stay explicit",
    );
    assert_eq!(
        error,
        BpmnEngineError::UnsupportedGatewayConfiguration {
            process_id: ("gateway_flow".to_string()).into(),
            node_id: ("decision".to_string()).into(),
            detail: "unsupported_condition_expression",
        }
    );
}
