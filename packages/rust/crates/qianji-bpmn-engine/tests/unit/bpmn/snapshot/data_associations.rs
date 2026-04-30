use super::snapshot_fixture;
use crate::test_support::MustExt as _;

#[test]
fn bpmn_snapshot_preserves_data_association_expression_metadata() {
    let snapshot = snapshot_fixture("metadata-data-association-expressions.bpmn");

    let process = snapshot
        .process("Process_DataAssociationExpressions")
        .must("data-association process should be indexed by id");
    assert_eq!(process.data_input_association_count, 1);
    assert_eq!(process.data_output_association_count, 1);

    let input_association = &process.data_input_associations[0];
    assert_eq!(
        input_association.association_id.as_deref(),
        Some("DataInputAssociation_MapOrder")
    );
    assert_eq!(input_association.source_refs, ["DataObject_Order"]);
    assert_eq!(
        input_association.target_ref.as_deref(),
        Some("DataInput_Order")
    );
    let transformation = input_association
        .transformation
        .as_ref()
        .must("input association transformation should be preserved");
    assert_eq!(
        transformation.expression_id.as_deref(),
        Some("Transformation_OrderPayload")
    );
    assert_eq!(transformation.body.as_deref(), Some("order.payload"));
    assert_eq!(transformation.language.as_deref(), Some("text/plain"));
    assert_eq!(
        transformation.evaluates_to_type_ref.as_deref(),
        Some("Item_Order")
    );

    let input_assignment = &input_association.assignments[0];
    assert_eq!(
        input_assignment.assignment_id.as_deref(),
        Some("Assignment_InputStatus")
    );
    assert_eq!(
        input_assignment
            .from
            .as_ref()
            .and_then(|expression| expression.expression_id.as_deref()),
        Some("Expression_InputStatusFrom")
    );
    assert_eq!(
        input_assignment
            .from
            .as_ref()
            .and_then(|expression| expression.body.as_deref()),
        Some("{\"status\":\"draft\"}")
    );
    assert_eq!(
        input_assignment
            .to
            .as_ref()
            .and_then(|expression| expression.body.as_deref()),
        Some("DataInput_Order.status")
    );

    let output_association = &process.data_output_associations[0];
    assert_eq!(
        output_association.association_id.as_deref(),
        Some("DataOutputAssociation_MapOrder")
    );
    assert_eq!(
        output_association.assignments[0]
            .from
            .as_ref()
            .and_then(|expression| expression.body.as_deref()),
        Some("DataOutput_Decision.approved")
    );
    assert_eq!(
        output_association.assignments[0]
            .to
            .as_ref()
            .and_then(|expression| expression.body.as_deref()),
        Some("decision.approved")
    );
}
