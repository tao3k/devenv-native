use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};
use xiuxian_qianji_bpmn_engine::snapshot_bpmn_source;

#[test]
fn bpmn_linter_preserves_choreography_metadata_surface() {
    let source = bpmn_fixture_source("metadata-choreography.bpmn");
    let report = lint_bpmn_source(&source);

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "choreography metadata should lint cleanly as passive metadata: {report:?}"
    );
    assert!(report.issues.is_empty());

    let snapshot = snapshot_bpmn_source(&source)
        .unwrap_or_else(|error| panic!("choreography fixture should snapshot: {error}"));
    assert_eq!(snapshot.root.collaboration_count, 2);
    assert_eq!(
        snapshot
            .collaborations
            .iter()
            .map(|collaboration| collaboration.participants.len())
            .sum::<usize>(),
        2
    );
    assert_eq!(
        snapshot
            .collaborations
            .iter()
            .map(|collaboration| collaboration.message_flows.len())
            .sum::<usize>(),
        2
    );
    assert_eq!(
        snapshot
            .collaborations
            .iter()
            .map(|collaboration| collaboration.choreography_activities.len())
            .sum::<usize>(),
        3
    );
    assert_eq!(
        snapshot.collaborations[0].choreography_activities[0]
            .activity_id
            .as_deref(),
        Some("ChoreographyTask_Order")
    );
    assert_eq!(
        snapshot.collaborations[0].choreography_activities[0].message_flow_refs[0].as_str(),
        "MessageFlow_Request"
    );
    assert_eq!(
        snapshot.collaborations[0].choreography_activities[2]
            .called_choreography_ref
            .as_deref(),
        Some("GlobalChoreography_Escalation")
    );
}
