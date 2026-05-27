use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};
use xiuxian_qianji_bpmn_engine::{
    BpmnCollaborationExecutionPolicy, BpmnCollaborationRuntimeScope, BpmnCollaborationSnapshot,
    BpmnDocumentSnapshot, BpmnParseOptions, BpmnSourceFile, parse_bpmn_package,
    snapshot_bpmn_source,
};

#[test]
fn bpmn_linter_preserves_collaboration_metadata_surface() {
    let source = bpmn_fixture_source("invalid-collaboration-participant.bpmn");
    let report = lint_bpmn_source(&source);

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "collaboration metadata should lint cleanly as passive metadata: {report:?}"
    );
    assert!(report.issues.is_empty());

    let snapshot = snapshot_bpmn_source(&source)
        .unwrap_or_else(|error| panic!("collaboration fixture should snapshot: {error}"));
    assert_collaboration_snapshot_metadata(&snapshot);
    assert_collaboration_host_boundary(&source);
}

fn assert_collaboration_host_boundary(source: &BpmnSourceFile) {
    let package = parse_bpmn_package(std::slice::from_ref(source), &BpmnParseOptions::default())
        .unwrap_or_else(|error| panic!("collaboration fixture should parse: {error}"));
    let boundary = &package.collaboration_host_envelope().boundary;
    assert_eq!(
        boundary.execution_policy,
        BpmnCollaborationExecutionPolicy::MetadataOnly
    );
    assert_eq!(
        boundary.runtime_scope,
        BpmnCollaborationRuntimeScope::SingleProcessGraph
    );
    assert!(
        boundary
            .deferred_semantics
            .iter()
            .any(|semantic| semantic.as_ref() == "message_flow_routing")
    );
    assert!(
        boundary
            .deferred_semantics
            .iter()
            .any(|semantic| semantic.as_ref() == "conversation_routing")
    );
    assert!(
        boundary
            .deferred_semantics
            .iter()
            .any(|semantic| semantic.as_ref() == "choreography_execution")
    );
    assert!(
        boundary
            .deferred_semantics
            .iter()
            .any(|semantic| semantic.as_ref() == "correlation_matching")
    );
}

fn assert_collaboration_snapshot_metadata(snapshot: &BpmnDocumentSnapshot) {
    assert_eq!(snapshot.root.collaboration_count, 1);
    assert_eq!(snapshot.root.item_definition_count, 1);
    assert_eq!(snapshot.root.message_count, 1);
    assert_eq!(snapshot.root.correlation_property_count, 1);

    let collaboration = &snapshot.collaborations[0];
    assert_eq!(collaboration.participants.len(), 2);
    assert_eq!(collaboration.message_flows.len(), 1);
    assert_eq!(collaboration.conversation_nodes.len(), 1);
    assert_eq!(collaboration.conversation_links.len(), 1);
    assert_eq!(
        collaboration.conversation_nodes[0].correlation_keys.len(),
        1
    );
    assert_eq!(
        snapshot.root.item_definitions[0]
            .item_definition_id
            .as_deref(),
        Some("order_item")
    );
    assert_eq!(
        snapshot.root.item_definitions[0].structure_ref.as_deref(),
        Some("tns:Order")
    );
    assert_eq!(
        snapshot.root.messages[0].message_id.as_deref(),
        Some("order_message")
    );
    assert_eq!(
        snapshot.root.messages[0].item_ref.as_deref(),
        Some("order_item")
    );
    assert_eq!(
        snapshot.root.correlation_properties[0].type_ref.as_deref(),
        Some("tns:OrderId")
    );
    assert_eq!(
        snapshot.root.correlation_properties[0]
            .retrieval_expressions
            .len(),
        1
    );
    assert_eq!(
        snapshot.root.correlation_properties[0].retrieval_expressions[0]
            .message_ref
            .as_deref(),
        Some("order_message")
    );
    assert_eq!(
        snapshot.root.correlation_properties[0].retrieval_expressions[0]
            .message_path
            .as_deref(),
        Some("payload.orderId")
    );
    assert_collaboration_routing_metadata(collaboration);
}

fn assert_collaboration_routing_metadata(collaboration: &BpmnCollaborationSnapshot) {
    assert_eq!(
        collaboration.participants[0].process_ref.as_deref(),
        Some("order_flow")
    );
    assert_eq!(
        collaboration.message_flows[0].message_ref.as_deref(),
        Some("order_message")
    );
    assert_eq!(
        collaboration.conversation_nodes[0].node_id.as_deref(),
        Some("conversation_order")
    );
    assert_eq!(
        collaboration.conversation_nodes[0].participant_refs[0].as_str(),
        "participant_customer"
    );
    assert_eq!(
        collaboration.conversation_links[0].target_ref.as_deref(),
        Some("conversation_order")
    );
}

#[test]
fn bpmn_linter_accepts_collaboration_runtime_route_constraints_as_metadata() {
    let source = BpmnSourceFile::new(
        "collaboration-task-routing-metadata.bpmn",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" id="pkg_collaboration_route_metadata">
  <bpmn:collaboration id="collaboration_route_metadata">
    <bpmn:participant id="participant_route_metadata" processRef="route_metadata" />
  </bpmn:collaboration>
  <bpmn:process id="route_metadata" isExecutable="true">
    <bpmn:startEvent id="start" />
    <bpmn:serviceTask id="branching_task" />
    <bpmn:endEvent id="left_done" />
    <bpmn:endEvent id="right_done" />
    <bpmn:sequenceFlow id="flow_start" sourceRef="start" targetRef="branching_task" />
    <bpmn:sequenceFlow id="flow_left" sourceRef="branching_task" targetRef="left_done" />
    <bpmn:sequenceFlow id="flow_right" sourceRef="branching_task" targetRef="right_done" />
  </bpmn:process>
</bpmn:definitions>"#,
    );
    let report = lint_bpmn_source(&source);

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "collaboration documents preserve runtime routing constraints as metadata: {report:?}"
    );
    assert!(report.issues.is_empty());
}
