use super::snapshot_fixture;

#[test]
fn bpmn_snapshot_preserves_choreography_metadata() {
    let snapshot = snapshot_fixture("metadata-choreography.bpmn");

    assert_eq!(snapshot.root.collaboration_count, 2);
    let choreography = &snapshot.collaborations[0];
    assert_eq!(choreography.collaboration_kind, "choreography");
    assert_eq!(
        choreography.collaboration_id.as_deref(),
        Some("Choreography_Order")
    );
    assert_eq!(
        choreography
            .is_closed
            .map(qianji_bpmn_engine::bpmn_model_api::BpmnSnapshotFlag::get),
        Some(false)
    );
    assert_eq!(choreography.participants.len(), 2);
    assert_eq!(choreography.message_flows.len(), 2);
    assert_eq!(choreography.choreography_activities.len(), 3);

    let task = &choreography.choreography_activities[0];
    assert_eq!(task.activity_kind, "choreographyTask");
    assert_eq!(task.activity_id.as_deref(), Some("ChoreographyTask_Order"));
    assert_eq!(
        task.initiating_participant_ref.as_deref(),
        Some("Participant_Requester")
    );
    assert_eq!(task.loop_type.as_deref(), Some("Standard"));
    assert_eq!(
        task.participant_refs,
        ["Participant_Requester", "Participant_Approver"]
    );
    assert_eq!(
        task.message_flow_refs,
        ["MessageFlow_Request", "MessageFlow_Response"]
    );
    assert_eq!(
        task.correlation_keys[0].correlation_property_refs,
        ["Correlation_Order"]
    );

    let sub_choreography = &choreography.choreography_activities[1];
    assert_eq!(sub_choreography.activity_kind, "subChoreography");
    assert_eq!(sub_choreography.child_activities.len(), 1);
    assert_eq!(
        sub_choreography.child_activities[0].activity_id.as_deref(),
        Some("ChoreographyTask_Return_Ack")
    );
    assert_eq!(
        sub_choreography.child_activities[0].message_flow_refs,
        ["MessageFlow_Response"]
    );

    let call_choreography = &choreography.choreography_activities[2];
    assert_eq!(call_choreography.activity_kind, "callChoreography");
    assert_eq!(
        call_choreography.called_choreography_ref.as_deref(),
        Some("GlobalChoreography_Escalation")
    );
    assert_eq!(call_choreography.participant_associations.len(), 1);
    assert_eq!(
        call_choreography.participant_associations[0]
            .inner_participant_ref
            .as_deref(),
        Some("Participant_Approver")
    );

    let global_choreography = &snapshot.collaborations[1];
    assert_eq!(
        global_choreography.collaboration_kind,
        "globalChoreographyTask"
    );
    assert_eq!(
        global_choreography.initiating_participant_ref.as_deref(),
        Some("Participant_Requester")
    );
}
