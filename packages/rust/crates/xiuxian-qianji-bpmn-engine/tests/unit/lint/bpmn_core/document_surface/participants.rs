use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};
use xiuxian_qianji_bpmn_engine::snapshot_bpmn_source;

#[test]
fn bpmn_linter_preserves_partner_participant_metadata_surface() {
    let source = bpmn_fixture_source("metadata-partner-participant.bpmn");
    let report = lint_bpmn_source(&source);

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "partner participant metadata should lint cleanly as passive metadata: {report:?}"
    );
    assert!(report.issues.is_empty());

    let snapshot = snapshot_bpmn_source(&source)
        .unwrap_or_else(|error| panic!("partner participant fixture should snapshot: {error}"));
    assert_eq!(snapshot.root.partner_entity_count, 1);
    assert_eq!(snapshot.root.partner_role_count, 1);
    assert_eq!(snapshot.root.end_point_count, 1);
    assert_eq!(
        snapshot.root.partner_entities[0].participant_refs[0].as_str(),
        "Participant_Requester"
    );
    assert_eq!(
        snapshot.root.partner_roles[0].participant_refs[0].as_str(),
        "Participant_Approver"
    );
    let participant = &snapshot.collaborations[0].participants[0];
    assert_eq!(participant.interface_refs[0].as_str(), "Interface_Review");
    assert_eq!(
        participant
            .participant_multiplicity
            .as_ref()
            .and_then(|multiplicity| multiplicity.maximum.as_deref()),
        Some("3")
    );
}

#[test]
fn bpmn_linter_accepts_passive_lane_metadata_surface() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-lane-set.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}
