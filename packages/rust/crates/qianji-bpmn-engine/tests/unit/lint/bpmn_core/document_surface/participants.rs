use super::*;

#[test]
fn bpmn_linter_reports_partner_participant_metadata_surface_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("metadata-partner-participant.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_collaboration_surface");
    assert_eq!(issue.evidence["snapshot_available"], true);
    assert_eq!(issue.evidence["snapshot"]["partner_entity_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["partner_role_count"], 1);
    assert_eq!(issue.evidence["snapshot"]["end_point_count"], 1);
    assert_eq!(
        issue.evidence["snapshot"]["participant_interface_ref_count"],
        1
    );
    assert_eq!(
        issue.evidence["snapshot"]["participant_end_point_ref_count"],
        1
    );
    assert_eq!(
        issue.evidence["snapshot"]["participant_multiplicity_count"],
        1
    );
    assert_eq!(
        issue.evidence["snapshot"]["partner_entities"][0]["participant_refs"][0],
        "Participant_Requester"
    );
    assert_eq!(
        issue.evidence["snapshot"]["partner_roles"][0]["participant_refs"][0],
        "Participant_Approver"
    );
    assert_eq!(
        issue.evidence["snapshot"]["collaborations"][0]["participants"][0]["interface_refs"][0],
        "Interface_Review"
    );
    assert_eq!(
        issue.evidence["snapshot"]["collaborations"][0]["participants"][0]["participant_multiplicity"]
            ["maximum"],
        "3"
    );
}

#[test]
fn bpmn_linter_accepts_passive_lane_metadata_surface() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-lane-set.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}
