use super::snapshot_fixture;

#[test]
fn bpmn_snapshot_preserves_import_metadata_catalogs() {
    let snapshot = snapshot_fixture("metadata-import-catalog.bpmn");

    assert_eq!(snapshot.root.import_count, 1);
    assert_eq!(snapshot.root.imports.len(), 1);
    let import = &snapshot.root.imports[0];
    assert_eq!(
        import.namespace.as_deref(),
        Some("https://example.com/bpmn/shared")
    );
    assert_eq!(import.location.as_deref(), Some("shared-processes.bpmn"));
    assert_eq!(
        import.import_type.as_deref(),
        Some("http://www.omg.org/spec/BPMN/20100524/MODEL")
    );
}

#[test]
fn bpmn_snapshot_preserves_extension_metadata_catalogs() {
    let snapshot = snapshot_fixture("metadata-extension-catalog.bpmn");

    assert_eq!(snapshot.root.extension_count, 2);
    assert_eq!(snapshot.root.extensions.len(), 2);

    let required_extension = &snapshot.root.extensions[0];
    assert_eq!(
        required_extension.definition.as_deref(),
        Some("ext:analytics")
    );
    assert!(required_extension.must_understand);
    assert_eq!(
        required_extension.documentation,
        ["Host extension declaration"]
    );

    let passive_extension = &snapshot.root.extensions[1];
    assert_eq!(passive_extension.definition.as_deref(), Some("ext:passive"));
    assert!(!passive_extension.must_understand);
    assert_eq!(
        passive_extension.documentation,
        ["Passive extension declaration"]
    );
}

#[test]
fn bpmn_snapshot_preserves_relationship_metadata_catalogs() {
    let snapshot = snapshot_fixture("metadata-relationship-catalog.bpmn");

    assert_eq!(snapshot.root.relationship_count, 1);
    assert_eq!(snapshot.root.relationships.len(), 1);
    let relationship = &snapshot.root.relationships[0];
    assert_eq!(
        relationship.relationship_id.as_deref(),
        Some("Relationship_RequestLineage")
    );
    assert_eq!(relationship.relationship_type.as_deref(), Some("lineage"));
    assert_eq!(relationship.direction.as_deref(), Some("Forward"));
    assert_eq!(
        relationship.source_refs,
        ["Message_Request", "Item_Request"]
    );
    assert_eq!(relationship.target_refs, ["review"]);
}

#[test]
fn bpmn_snapshot_preserves_event_definition_metadata_catalogs() {
    let snapshot = snapshot_fixture("metadata-event-definition-catalog.bpmn");

    assert_eq!(snapshot.root.error_count, 1);
    assert_eq!(
        snapshot.root.errors[0].error_id.as_deref(),
        Some("fatal_review_error")
    );
    assert_eq!(
        snapshot.root.errors[0].error_code.as_deref(),
        Some("fatal_review")
    );
    assert_eq!(
        snapshot.root.errors[0].structure_ref.as_deref(),
        Some("tns:ReviewError")
    );
    assert_eq!(snapshot.root.escalation_count, 1);
    assert_eq!(
        snapshot.root.escalations[0].escalation_id.as_deref(),
        Some("review_escalated")
    );
    assert_eq!(
        snapshot.root.escalations[0].escalation_code.as_deref(),
        Some("review_escalated")
    );
    assert_eq!(
        snapshot.root.escalations[0].structure_ref.as_deref(),
        Some("tns:ReviewEscalation")
    );
    assert_eq!(snapshot.root.signal_count, 1);
    assert_eq!(
        snapshot.root.signals[0].signal_id.as_deref(),
        Some("alert_signal")
    );
    assert_eq!(
        snapshot.root.signals[0].structure_ref.as_deref(),
        Some("tns:AlertSignal")
    );
}
