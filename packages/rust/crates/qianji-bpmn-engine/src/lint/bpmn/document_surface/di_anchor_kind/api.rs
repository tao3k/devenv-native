use serde_json::json;

use crate::bpmn_model_api::{BpmnDocumentSnapshot, BpmnPlaneSnapshot};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::bpmn_snapshot_api::snapshot_bpmn_source;
use crate::lint::bpmn::document_surface::SNAPSHOT_EVIDENCE_LIMIT;
use crate::lint::bpmn::document_surface::di_semantic::SemanticElementIndex;
use crate::lint::bpmn::document_surface::summary::diagram_snapshot_summary;
use crate::lint_api::LintIssue;

use super::model::DiAnchorKindViolation;

pub(in crate::lint::bpmn::document_surface) fn diagram_anchor_kind_issue(
    source: &BpmnSourceFile,
) -> Option<LintIssue> {
    let snapshot = snapshot_bpmn_source(source).ok()?;
    let semantic_index = SemanticElementIndex::from_source(source)?;
    let violations = invalid_anchor_kinds(&snapshot, &semantic_index);
    if violations.is_empty() {
        return None;
    }

    let source_id = &source.source_id;
    let evidence = json!({
        "source_id": source_id,
        "element": "BPMNDiagram",
        "deferred_family": "diagram",
        "snapshot_available": true,
        "semantic_id_count": semantic_index.len(),
        "invalid_anchor_kind_count": violations.len(),
        "invalid_anchor_kinds": violations
            .iter()
            .take(SNAPSHOT_EVIDENCE_LIMIT)
            .map(DiAnchorKindViolation::evidence)
            .collect::<Vec<_>>(),
        "invalid_anchor_kinds_truncated": violations.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "snapshot": diagram_snapshot_summary(&snapshot),
    });

    Some(LintIssue::from_parts(
        "bpmn.invalid_di_anchor_kind",
        "BPMN diagram interchange metadata points at the wrong semantic element kind",
        format!(
            "Source '{source_id}' contains BPMN DI `bpmnElement` anchors whose semantic element kind does not match the DI element kind."
        ),
        "BPMN DI stays metadata-only in the bounded runtime, but diagram planes, shapes, and edges must point at compatible BPMN semantic elements so interchange tools can trace layout metadata without deriving execution behavior from coordinates.",
        vec![
            "Point each `bpmndi:BPMNPlane` at the owning process, collaboration, or choreography root.".to_string(),
            "Point each `bpmndi:BPMNShape` at a BPMN node, artifact, participant, lane, conversation node, choreography activity, or data reference it displays.".to_string(),
            "Point each `bpmndi:BPMNEdge` at a BPMN flow, association, conversation link, or data association it displays.".to_string(),
            "Do not use diagram anchors to encode runtime routing; keep executable behavior in BPMN flow, events, tasks, gateways, and data mappings.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' so every BPMN DI `bpmnElement` anchor points at a semantic BPMN element kind compatible with its DI element. Preserve valid diagram metadata, but retarget shape and edge anchors that were accidentally swapped."
        ),
        evidence,
    ))
}

fn invalid_anchor_kinds(
    snapshot: &BpmnDocumentSnapshot,
    semantic_index: &SemanticElementIndex,
) -> Vec<DiAnchorKindViolation> {
    let mut violations = Vec::new();
    for diagram in &snapshot.root.diagrams {
        let diagram_id = diagram.diagram_id.as_deref();
        let Some(plane) = diagram.plane.as_ref() else {
            continue;
        };
        collect_plane_anchor_kind_violations(&mut violations, diagram_id, plane, semantic_index);
    }
    violations
}

fn collect_plane_anchor_kind_violations(
    violations: &mut Vec<DiAnchorKindViolation>,
    diagram_id: Option<&str>,
    plane: &BpmnPlaneSnapshot,
    semantic_index: &SemanticElementIndex,
) {
    let plane_id = plane.plane_id.as_deref();
    if let Some(reference) = plane.bpmn_element.as_deref()
        && let Some(actual_tag) = semantic_index.tag_for(reference)
        && is_obvious_non_plane_root_tag(actual_tag)
    {
        violations.push(DiAnchorKindViolation::plane(
            diagram_id, plane_id, reference, actual_tag,
        ));
    }

    for shape in &plane.shapes {
        if let Some(reference) = shape.bpmn_element.as_deref()
            && let Some(actual_tag) = semantic_index.tag_for(reference)
            && is_obvious_non_shape_tag(actual_tag)
        {
            violations.push(DiAnchorKindViolation::shape(
                diagram_id,
                plane_id,
                shape.shape_id.as_deref(),
                reference,
                actual_tag,
            ));
        }
    }

    for edge in &plane.edges {
        if let Some(reference) = edge.bpmn_element.as_deref()
            && let Some(actual_tag) = semantic_index.tag_for(reference)
            && is_obvious_non_edge_tag(actual_tag)
        {
            violations.push(DiAnchorKindViolation::edge(
                diagram_id,
                plane_id,
                edge.edge_id.as_deref(),
                reference,
                actual_tag,
            ));
        }
    }
}

fn is_obvious_non_plane_root_tag(tag: &str) -> bool {
    is_shape_anchor_tag(tag) || is_edge_anchor_tag(tag)
}

fn is_obvious_non_shape_tag(tag: &str) -> bool {
    is_plane_root_tag(tag) || is_edge_anchor_tag(tag)
}

fn is_obvious_non_edge_tag(tag: &str) -> bool {
    is_plane_root_tag(tag) || is_shape_anchor_tag(tag)
}

fn is_plane_root_tag(tag: &str) -> bool {
    matches!(tag, "process" | "collaboration" | "choreography")
}

fn is_shape_anchor_tag(tag: &str) -> bool {
    matches!(
        tag,
        "adHocSubProcess"
            | "boundaryEvent"
            | "businessRuleTask"
            | "callActivity"
            | "callChoreography"
            | "callConversation"
            | "choreographyTask"
            | "complexGateway"
            | "conversation"
            | "dataObject"
            | "dataObjectReference"
            | "dataStoreReference"
            | "endEvent"
            | "eventBasedGateway"
            | "exclusiveGateway"
            | "globalBusinessRuleTask"
            | "globalManualTask"
            | "globalScriptTask"
            | "globalTask"
            | "globalUserTask"
            | "group"
            | "inclusiveGateway"
            | "intermediateCatchEvent"
            | "intermediateThrowEvent"
            | "lane"
            | "manualTask"
            | "parallelGateway"
            | "participant"
            | "receiveTask"
            | "scriptTask"
            | "sendTask"
            | "serviceTask"
            | "startEvent"
            | "subChoreography"
            | "subConversation"
            | "subProcess"
            | "task"
            | "textAnnotation"
            | "transaction"
            | "userTask"
    )
}

fn is_edge_anchor_tag(tag: &str) -> bool {
    matches!(
        tag,
        "association"
            | "conversationAssociation"
            | "conversationLink"
            | "dataInputAssociation"
            | "dataOutputAssociation"
            | "messageFlow"
            | "messageFlowAssociation"
            | "participantAssociation"
            | "sequenceFlow"
    )
}
