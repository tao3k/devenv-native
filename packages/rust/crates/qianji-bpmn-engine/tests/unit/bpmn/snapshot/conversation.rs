use super::snapshot_fixture;

#[test]
fn bpmn_snapshot_preserves_conversation_metadata() {
    let snapshot = snapshot_fixture("metadata-collaboration-conversation.bpmn");

    assert_eq!(snapshot.root.collaboration_count, 1);
    let collaboration = &snapshot.collaborations[0];
    assert_eq!(collaboration.collaboration_kind, "collaboration");
    assert_eq!(
        collaboration.collaboration_id.as_deref(),
        Some("Collaboration_Conversation")
    );
    assert_eq!(collaboration.is_closed.map(|flag| flag.get()), Some(false));
    assert_eq!(collaboration.participants.len(), 2);
    assert_eq!(collaboration.message_flows.len(), 1);
    assert_eq!(collaboration.conversation_nodes.len(), 3);
    assert_eq!(collaboration.conversation_associations.len(), 1);
    assert_eq!(collaboration.participant_associations.len(), 1);
    assert_eq!(collaboration.message_flow_associations.len(), 1);
    assert_eq!(collaboration.correlation_keys.len(), 1);
    assert_eq!(collaboration.choreography_refs, ["Choreography_Order"]);
    assert_eq!(collaboration.conversation_links.len(), 1);

    let conversation = &collaboration.conversation_nodes[0];
    assert_eq!(conversation.node_kind, "conversation");
    assert_eq!(conversation.node_id.as_deref(), Some("Conversation_Order"));
    assert_eq!(
        conversation.participant_refs,
        ["Participant_Requester", "Participant_Approver"]
    );
    assert_eq!(conversation.message_flow_refs, ["MessageFlow_Order"]);
    assert_eq!(
        conversation.correlation_keys[0].correlation_property_refs,
        ["Correlation_Order"]
    );

    let sub_conversation = &collaboration.conversation_nodes[1];
    assert_eq!(sub_conversation.node_kind, "subConversation");
    assert_eq!(sub_conversation.child_nodes.len(), 1);
    assert_eq!(
        sub_conversation.child_nodes[0].node_id.as_deref(),
        Some("Conversation_Return_Ack")
    );

    let call_conversation = &collaboration.conversation_nodes[2];
    assert_eq!(call_conversation.node_kind, "callConversation");
    assert_eq!(
        call_conversation.called_collaboration_ref.as_deref(),
        Some("Collaboration_Reusable")
    );
    assert_eq!(call_conversation.participant_associations.len(), 1);
    assert_eq!(
        call_conversation.participant_associations[0]
            .inner_participant_ref
            .as_deref(),
        Some("Participant_Approver")
    );
    assert_eq!(
        collaboration.conversation_associations[0]
            .inner_conversation_node_ref
            .as_deref(),
        Some("Conversation_Return_Ack")
    );
    assert_eq!(
        collaboration.participant_associations[0]
            .outer_participant_ref
            .as_deref(),
        Some("Participant_Requester")
    );
    assert_eq!(
        collaboration.message_flow_associations[0]
            .inner_message_flow_ref
            .as_deref(),
        Some("MessageFlow_Order")
    );
    assert_eq!(
        collaboration.correlation_keys[0].correlation_property_refs,
        ["Correlation_Order"]
    );
    assert_eq!(
        collaboration.conversation_links[0].target_ref.as_deref(),
        Some("Conversation_Order")
    );
}
