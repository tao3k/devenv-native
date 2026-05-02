use crate::bpmn_model_api::{BpmnDataAssociationExpressionSnapshot, BpmnDataAssociationSnapshot};
use crate::lint::bpmn::document_surface::SNAPSHOT_EVIDENCE_LIMIT;
use serde_json::{Value, json};

pub(super) fn data_association_evidence(association: &BpmnDataAssociationSnapshot) -> Value {
    json!({
        "association_id": association.association_id,
        "source_refs": association.source_refs,
        "target_ref": association.target_ref,
        "transformation": data_association_expression_evidence(association.transformation.as_ref()),
        "assignment_count": association.assignments.len(),
        "assignments": association.assignments.iter().take(SNAPSHOT_EVIDENCE_LIMIT).map(|assignment| {
            json!({
                "assignment_id": assignment.assignment_id,
                "from": data_association_expression_evidence(assignment.from.as_ref()),
                "to": data_association_expression_evidence(assignment.to.as_ref()),
            })
        }).collect::<Vec<_>>(),
        "assignments_truncated": association.assignments.len() > SNAPSHOT_EVIDENCE_LIMIT,
    })
}

fn data_association_expression_evidence(
    expression: Option<&BpmnDataAssociationExpressionSnapshot>,
) -> Value {
    expression.map_or(Value::Null, |expression| {
        json!({
            "expression_id": expression.expression_id,
            "body": expression.body,
            "language": expression.language,
            "evaluates_to_type_ref": expression.evaluates_to_type_ref,
        })
    })
}
