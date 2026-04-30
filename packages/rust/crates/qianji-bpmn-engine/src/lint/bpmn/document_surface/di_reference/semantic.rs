use std::collections::BTreeSet;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::bpmn_model_api::BpmnPlaneSnapshot;
use crate::bpmn_parse_api::BpmnSourceFile;

use super::model::{DiReferenceScope, DiReferenceTarget, DiReferenceViolation};
use crate::lint::bpmn::document_surface::xml::local_name;

pub(super) fn collect_semantic_reference_violations(
    violations: &mut Vec<DiReferenceViolation>,
    diagram_id: Option<&str>,
    plane_id: Option<&str>,
    plane: &BpmnPlaneSnapshot,
    semantic_ids: &BTreeSet<String>,
) {
    let scope = DiReferenceScope {
        diagram_id,
        plane_id,
    };
    collect_reference(
        violations,
        scope,
        DiReferenceTarget {
            element: "BPMNPlane",
            element_id: plane.plane_id.clone(),
            attribute: "bpmnElement",
        },
        plane.bpmn_element.as_deref(),
        semantic_ids,
    );
    for shape in &plane.shapes {
        collect_reference(
            violations,
            scope,
            DiReferenceTarget {
                element: "BPMNShape",
                element_id: shape.shape_id.clone(),
                attribute: "bpmnElement",
            },
            shape.bpmn_element.as_deref(),
            semantic_ids,
        );
    }
    for edge in &plane.edges {
        collect_reference(
            violations,
            scope,
            DiReferenceTarget {
                element: "BPMNEdge",
                element_id: edge.edge_id.clone(),
                attribute: "bpmnElement",
            },
            edge.bpmn_element.as_deref(),
            semantic_ids,
        );
    }
}

pub(super) fn semantic_ids_from_source(source: &BpmnSourceFile) -> Option<BTreeSet<String>> {
    let mut reader = Reader::from_str(&source.contents);
    reader.config_mut().trim_text(true);
    let mut ids = BTreeSet::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event) | Event::Empty(event)) => {
                collect_semantic_id(&reader, &event, &mut ids);
            }
            Ok(Event::Eof) => return Some(ids),
            Err(_) => return None,
            Ok(_) => {}
        }
    }
}

fn collect_reference(
    violations: &mut Vec<DiReferenceViolation>,
    scope: DiReferenceScope<'_>,
    target: DiReferenceTarget,
    reference: Option<&str>,
    semantic_ids: &BTreeSet<String>,
) {
    let Some(reference) = reference else {
        return;
    };
    if semantic_ids.contains(reference) {
        return;
    }
    violations.push(DiReferenceViolation::semantic(scope, target, reference));
}

fn collect_semantic_id(reader: &Reader<&[u8]>, event: &BytesStart<'_>, ids: &mut BTreeSet<String>) {
    let event_name = event.name();
    let Some(tag) = local_name(event_name.as_ref()) else {
        return;
    };
    if is_di_metadata_tag(tag) {
        return;
    }
    let Some(id) = attribute_value(reader, event, "id") else {
        return;
    };
    ids.insert(id);
}

fn attribute_value(reader: &Reader<&[u8]>, event: &BytesStart<'_>, name: &str) -> Option<String> {
    for attribute in event.attributes().flatten() {
        if local_name(attribute.key.as_ref()) == Some(name) {
            return attribute
                .decode_and_unescape_value(reader.decoder())
                .ok()
                .map(std::borrow::Cow::into_owned);
        }
    }
    None
}

fn is_di_metadata_tag(tag: &str) -> bool {
    matches!(
        tag,
        "BPMNDiagram"
            | "BPMNPlane"
            | "BPMNShape"
            | "BPMNEdge"
            | "BPMNLabel"
            | "BPMNLabelStyle"
            | "Bounds"
            | "waypoint"
            | "Font"
    )
}
