use super::snapshot_fixture;

#[test]
fn bpmn_snapshot_preserves_partner_participant_metadata() {
    let snapshot = snapshot_fixture("metadata-partner-participant.bpmn");

    assert_eq!(snapshot.root.end_point_count, 1);
    assert_eq!(
        snapshot.root.end_points[0].end_point_id.as_deref(),
        Some("Endpoint_ReviewApi")
    );
    assert_eq!(snapshot.root.partner_entity_count, 1);
    assert_eq!(
        snapshot.root.partner_entities[0]
            .partner_entity_id
            .as_deref(),
        Some("PartnerEntity_RequestNetwork")
    );
    assert_eq!(
        snapshot.root.partner_entities[0].participant_refs,
        ["Participant_Requester", "Participant_Approver"]
    );
    assert_eq!(snapshot.root.partner_role_count, 1);
    assert_eq!(
        snapshot.root.partner_roles[0].partner_role_id.as_deref(),
        Some("PartnerRole_Approver")
    );
    assert_eq!(
        snapshot.root.partner_roles[0].participant_refs,
        ["Participant_Approver"]
    );

    let collaboration = &snapshot.collaborations[0];
    let participant = &collaboration.participants[0];
    assert_eq!(
        participant.participant_id.as_deref(),
        Some("Participant_Requester")
    );
    assert_eq!(participant.interface_refs, ["Interface_Review"]);
    assert_eq!(participant.end_point_refs, ["Endpoint_ReviewApi"]);
    let Some(multiplicity) = participant.participant_multiplicity.as_ref() else {
        panic!("participant multiplicity should be preserved");
    };
    assert_eq!(
        multiplicity.multiplicity_id.as_deref(),
        Some("Multiplicity_Requester")
    );
    assert_eq!(multiplicity.minimum.as_deref(), Some("1"));
    assert_eq!(multiplicity.maximum.as_deref(), Some("3"));
}
