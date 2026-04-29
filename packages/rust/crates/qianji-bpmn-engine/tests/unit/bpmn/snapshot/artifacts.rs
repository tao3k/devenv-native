use super::snapshot_fixture;

#[test]
fn bpmn_snapshot_preserves_artifact_metadata() {
    let snapshot = snapshot_fixture("metadata-artifacts.bpmn");

    let collaboration = &snapshot.collaborations[0];
    assert_eq!(collaboration.associations.len(), 1);
    assert_eq!(collaboration.groups.len(), 1);
    assert_eq!(collaboration.text_annotations.len(), 1);
    assert_eq!(
        collaboration.associations[0].association_id.as_deref(),
        Some("Association_Collaboration")
    );
    assert_eq!(
        collaboration.associations[0].source_ref.as_deref(),
        Some("TextAnnotation_Collaboration")
    );
    assert_eq!(
        collaboration.associations[0].target_ref.as_deref(),
        Some("MessageFlow_Request")
    );
    assert_eq!(
        collaboration.associations[0]
            .association_direction
            .as_deref(),
        Some("One")
    );
    assert_eq!(
        collaboration.groups[0].category_value_ref.as_deref(),
        Some("CategoryValue_ManualReview")
    );
    assert_eq!(
        collaboration.text_annotations[0].text_format.as_deref(),
        Some("text/markdown")
    );
    assert_eq!(
        collaboration.text_annotations[0].text.as_deref(),
        Some("Review note from collaboration scope")
    );

    let Some(process) = snapshot.process("Process_Artifacts") else {
        panic!("artifact process should be indexed");
    };
    assert_eq!(process.association_count, 1);
    assert_eq!(process.group_count, 1);
    assert_eq!(process.text_annotation_count, 1);
    assert_eq!(
        process.associations[0].association_id.as_deref(),
        Some("Association_Process")
    );
    assert_eq!(
        process.associations[0].association_direction.as_deref(),
        Some("Both")
    );
    assert_eq!(
        process.groups[0].category_value_ref.as_deref(),
        Some("CategoryValue_ManualReview")
    );
    assert_eq!(
        process.text_annotations[0].annotation_id.as_deref(),
        Some("TextAnnotation_Process")
    );
    assert_eq!(
        process.text_annotations[0].text.as_deref(),
        Some("Process owner note")
    );
}
