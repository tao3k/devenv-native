use super::snapshot_fixture;
use crate::test_support::MustExt as _;

#[test]
fn bpmn_snapshot_preserves_flow_element_metadata() {
    let snapshot = snapshot_fixture("metadata-flow-element.bpmn");

    let process = snapshot
        .process("Process_FlowElementMetadata")
        .must("flow element metadata process should be indexed by id");
    assert_eq!(process.flow_element_metadata_count, 3);

    let start = &process.flow_element_metadata[0];
    assert_eq!(start.element_kind, "startEvent");
    assert_eq!(start.element_id.as_deref(), Some("Start_Flow"));
    assert_eq!(start.name.as_deref(), Some("Start"));
    assert!(start.has_auditing);
    assert_eq!(start.auditing_id.as_deref(), Some("Audit_Start"));
    assert!(!start.has_monitoring);
    assert_eq!(start.category_value_refs, ["CategoryValue_Audit"]);

    let task = &process.flow_element_metadata[1];
    assert_eq!(task.element_kind, "userTask");
    assert_eq!(task.element_id.as_deref(), Some("Task_Review"));
    assert_eq!(task.name.as_deref(), Some("Review"));
    assert!(task.has_auditing);
    assert_eq!(task.auditing_id.as_deref(), Some("Audit_Review"));
    assert!(task.has_monitoring);
    assert_eq!(task.monitoring_id.as_deref(), Some("Monitor_Review"));
    assert_eq!(
        task.category_value_refs,
        ["CategoryValue_Audit", "CategoryValue_Monitoring"]
    );

    let sequence_flow = &process.flow_element_metadata[2];
    assert_eq!(sequence_flow.element_kind, "sequenceFlow");
    assert_eq!(
        sequence_flow.element_id.as_deref(),
        Some("Flow_Start_Review")
    );
    assert!(!sequence_flow.has_auditing);
    assert!(sequence_flow.has_monitoring);
    assert_eq!(sequence_flow.monitoring_id.as_deref(), Some("Monitor_Flow"));
    assert!(sequence_flow.category_value_refs.is_empty());
}
