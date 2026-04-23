use super::{parse_fixture_error, parse_fixture_package};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{BpmnEngineError, BpmnNodeKind};

#[test]
fn parser_linear_service_task_materializes_dense_ir() {
    let package = parse_fixture_package("linear-service-task.bpmn");
    assert_eq!(package.package_id.as_ref(), "pkg_linear");

    let process = package
        .find_process("approve")
        .must("process should be discoverable by BPMN id");
    assert_eq!(process.nodes.len(), 3);
    assert_eq!(process.edges.len(), 2);
    assert_eq!(process.nodes[0].kind, BpmnNodeKind::StartEvent);
    assert_eq!(process.nodes[1].kind, BpmnNodeKind::ServiceTask);
    assert_eq!(process.nodes[2].kind, BpmnNodeKind::EndEvent);
    assert_eq!(process.outgoing_edge_indices(0), [0]);
    assert_eq!(process.outgoing_edge_indices(1), [1]);
    assert!(process.outgoing_edge_indices(2).is_empty());
    assert!(process.incoming_edge_indices(0).is_empty());
    assert_eq!(process.incoming_edge_indices(1), [0]);
    assert_eq!(process.incoming_edge_indices(2), [1]);
}

#[test]
fn parser_business_rule_task_keeps_dmn_placeholder() {
    let package = parse_fixture_package("linear-business-rule-placeholder.bpmn");
    let process = package
        .find_process("loan_review")
        .must("process should be present");
    assert_eq!(process.nodes[1].kind, BpmnNodeKind::BusinessRuleTask);
    let decision = process.nodes[1]
        .decision
        .as_ref()
        .must("DMN placeholder should be preserved");
    assert_eq!(decision.decision_id.as_ref(), "loan-decision");
}

#[test]
fn parser_duplicate_node_id_is_rejected() {
    let error = parse_fixture_error(
        "invalid-duplicate-node-id.bpmn",
        "duplicate node ids should fail validation",
    );
    assert_eq!(
        error,
        BpmnEngineError::DuplicateNodeId {
            process_id: "duplicate_nodes".to_string(),
            node_id: "task".to_string(),
        }
    );
}

#[test]
fn parser_missing_flow_target_is_rejected() {
    let error = parse_fixture_error(
        "invalid-missing-flow-target.bpmn",
        "missing sequence-flow endpoint should fail validation",
    );
    assert_eq!(
        error,
        BpmnEngineError::UnknownSequenceFlowEndpoint {
            process_id: "missing_target".to_string(),
            flow_id: "flow_1".to_string(),
            endpoint: "target",
            node_id: "missing_end".to_string(),
        }
    );
}
