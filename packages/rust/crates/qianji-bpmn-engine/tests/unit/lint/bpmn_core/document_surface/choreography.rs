use super::*;

#[test]
fn bpmn_linter_reports_choreography_metadata_surface_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("metadata-choreography.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_collaboration_surface");
    assert!(issue.title.contains("choreography"));
    assert_eq!(issue.evidence["snapshot_available"], true);
    assert_eq!(issue.evidence["snapshot"]["collaboration_count"], 2);
    assert_eq!(issue.evidence["snapshot"]["participant_count"], 2);
    assert_eq!(issue.evidence["snapshot"]["message_flow_count"], 2);
    assert_eq!(issue.evidence["snapshot"]["choreography_activity_count"], 4);
    assert_eq!(
        issue.evidence["snapshot"]["collaborations"][0]["choreography_activities"][0]["activity_id"],
        "ChoreographyTask_Order"
    );
    assert_eq!(
        issue.evidence["snapshot"]["collaborations"][0]["choreography_activities"][0]["message_flow_refs"]
            [0],
        "MessageFlow_Request"
    );
    assert_eq!(
        issue.evidence["snapshot"]["collaborations"][0]["choreography_activities"][2]["called_choreography_ref"],
        "GlobalChoreography_Escalation"
    );
}
