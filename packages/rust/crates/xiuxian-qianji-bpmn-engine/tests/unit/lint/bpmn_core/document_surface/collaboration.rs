use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};
use serde_json::json;

#[test]
fn bpmn_linter_reports_collaboration_metadata_surface_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-collaboration-participant.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_collaboration_surface");
    assert!(issue.why_it_failed.contains("pool"));
    assert!(issue.llm_fix_prompt.contains("host-level routing metadata"));
    assert_eq!(issue.evidence["snapshot_available"], true);
    assert_eq!(issue.evidence["snapshot"]["participant_count"], 2);
    assert_eq!(issue.evidence["snapshot"]["message_flow_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["conversation_node_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["conversation_link_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["correlation_key_count"], 1);
    assert_eq!(
        issue.evidence["snapshot"]["routing_boundary"]["status"],
        "metadata_only"
    );
    assert_eq!(
        issue.evidence["snapshot"]["routing_boundary"]["execution_policy"],
        "deferred"
    );
    assert_eq!(
        issue.evidence["snapshot"]["routing_boundary"]["runtime_scope"],
        "single_process_graph"
    );
    let Some(deferred_semantics) =
        issue.evidence["snapshot"]["routing_boundary"]["deferred_semantics"].as_array()
    else {
        panic!("routing boundary deferred semantics evidence");
    };
    assert!(deferred_semantics.contains(&json!("message_flow_routing")));
    assert!(deferred_semantics.contains(&json!("conversation_routing")));
    assert!(deferred_semantics.contains(&json!("choreography_execution")));
    assert!(deferred_semantics.contains(&json!("correlation_matching")));
    assert_eq!(issue.evidence["snapshot"]["item_definition_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["message_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["correlation_property_count"], 1);
    assert_eq!(
        issue.evidence["snapshot"]["item_definitions"][0]["item_definition_id"],
        "order_item"
    );
    assert_eq!(
        issue.evidence["snapshot"]["item_definitions"][0]["structure_ref"],
        "tns:Order"
    );
    assert_eq!(
        issue.evidence["snapshot"]["messages"][0]["message_id"],
        "order_message"
    );
    assert_eq!(
        issue.evidence["snapshot"]["messages"][0]["item_ref"],
        "order_item"
    );
    assert_eq!(
        issue.evidence["snapshot"]["correlation_properties"][0]["type_ref"],
        "tns:OrderId"
    );
    assert_eq!(
        issue.evidence["snapshot"]["correlation_properties"][0]["retrieval_expression_count"],
        1
    );
    assert_eq!(
        issue.evidence["snapshot"]["correlation_properties"][0]["retrieval_expressions"][0]["message_ref"],
        "order_message"
    );
    assert_eq!(
        issue.evidence["snapshot"]["correlation_properties"][0]["retrieval_expressions"][0]["message_path"],
        "payload.orderId"
    );
    assert_eq!(
        issue.evidence["snapshot"]["collaborations"][0]["participants"][0]["process_ref"],
        "order_flow"
    );
    assert_eq!(
        issue.evidence["snapshot"]["collaborations"][0]["message_flows"][0]["message_ref"],
        "order_message"
    );
    assert_eq!(
        issue.evidence["snapshot"]["collaborations"][0]["conversation_nodes"][0]["node_id"],
        "conversation_order"
    );
    assert_eq!(
        issue.evidence["snapshot"]["collaborations"][0]["conversation_nodes"][0]["participant_refs"]
            [0],
        "participant_customer"
    );
    assert_eq!(
        issue.evidence["snapshot"]["collaborations"][0]["conversation_links"][0]["target_ref"],
        "conversation_order"
    );
}
