use qianji_bpmn_engine::{
    BpmnCollaborationExecutionPolicy, BpmnCollaborationRuntimeScope, BpmnCorrelationKeyScope,
    BpmnEventDeduplicationPolicy, BpmnParseOptions, parse_bpmn_package,
};

use super::fixture_source;
use crate::test_support::MustExt as _;

#[test]
fn collaboration_host_envelope_materializes_participants_message_flows_and_correlation_intent() {
    let package = parse_bpmn_package(
        &[fixture_source("invalid-collaboration-participant.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("collaboration metadata fixture should parse");

    let envelope = package.collaboration_host_envelope();
    assert!(!envelope.is_empty());
    assert_eq!(
        envelope.boundary.execution_policy,
        BpmnCollaborationExecutionPolicy::MetadataOnly
    );
    assert_eq!(
        envelope.boundary.runtime_scope,
        BpmnCollaborationRuntimeScope::SingleProcessGraph
    );
    assert_eq!(
        envelope.boundary.event_deduplication_policy,
        BpmnEventDeduplicationPolicy::ExplicitEventReferenceOnly
    );
    assert!(
        envelope
            .boundary
            .deferred_semantics
            .iter()
            .any(|semantic| semantic.as_ref() == "correlation_matching")
    );

    let collaboration = envelope
        .collaborations
        .iter()
        .find(|collaboration| {
            collaboration.collaboration_id.as_deref() == Some("collaboration_order")
        })
        .must("collaboration shell should be exposed");
    assert_eq!(collaboration.name.as_deref(), None);

    let requester = envelope
        .participants
        .iter()
        .find(|participant| participant.participant_id.as_deref() == Some("participant_customer"))
        .must("participant should be exposed");
    assert_eq!(requester.process_ref.as_deref(), Some("order_flow"));
    assert_eq!(
        requester.collaboration_id.as_deref(),
        Some("collaboration_order")
    );

    let message_flow = envelope
        .message_flows
        .iter()
        .find(|flow| flow.message_flow_id.as_deref() == Some("order_message_flow"))
        .must("message flow should be exposed");
    assert_eq!(
        message_flow.source_ref.as_deref(),
        Some("participant_customer")
    );
    assert_eq!(
        message_flow.target_ref.as_deref(),
        Some("participant_fulfillment")
    );
    assert_eq!(message_flow.message_ref.as_deref(), Some("order_message"));

    let property = envelope
        .correlation_properties
        .iter()
        .find(|property| property.correlation_property_id.as_deref() == Some("order_correlation"))
        .must("correlation property should be exposed");
    assert_eq!(property.retrieval_expressions.len(), 1);
    assert_eq!(
        property.retrieval_expressions[0].message_ref.as_deref(),
        Some("order_message")
    );
    assert_eq!(
        property.retrieval_expressions[0].message_path.as_deref(),
        Some("payload.orderId")
    );

    let key = envelope
        .correlation_keys
        .iter()
        .find(|key| key.correlation_key_id.as_deref() == Some("order_key"))
        .must("conversation correlation key should be exposed");
    assert_eq!(key.scope, BpmnCorrelationKeyScope::Conversation);
    assert_eq!(key.scope_id.as_deref(), Some("conversation_order"));
    assert_eq!(
        key.correlation_property_refs[0].as_ref(),
        "order_correlation"
    );
}

#[test]
fn collaboration_host_envelope_preserves_process_correlation_subscriptions_as_metadata() {
    let package = parse_bpmn_package(
        &[fixture_source("metadata-process-callable.bpmn")],
        &BpmnParseOptions::default(),
    )
    .must("process correlation fixture should parse");

    let envelope = package.collaboration_host_envelope();
    let key = envelope
        .correlation_keys
        .iter()
        .find(|key| key.correlation_key_id.as_deref() == Some("CorrelationKey_Order"))
        .must("collaboration correlation key should be exposed");
    assert_eq!(key.scope, BpmnCorrelationKeyScope::Collaboration);
    assert_eq!(
        key.scope_id.as_deref(),
        Some("Collaboration_CallableMetadata")
    );
    assert_eq!(
        key.correlation_property_refs[0].as_ref(),
        "Correlation_Order"
    );

    let subscription = envelope
        .process_correlation_subscriptions
        .iter()
        .find(|subscription| subscription.subscription_id.as_deref() == Some("Subscription_Order"))
        .must("process correlation subscription should be exposed");
    assert_eq!(subscription.process_id.as_ref(), "Process_CallableMetadata");
    assert_eq!(
        subscription.correlation_key_ref.as_deref(),
        Some("CorrelationKey_Order")
    );
    assert_eq!(subscription.bindings.len(), 1);
    assert_eq!(
        subscription.bindings[0].correlation_property_ref.as_deref(),
        Some("Correlation_Order")
    );
    assert_eq!(
        subscription.bindings[0].data_path.as_deref(),
        Some("order.id")
    );
}
