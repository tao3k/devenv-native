use serde_json::{Value, json};

#[derive(Debug)]
pub(super) struct DiAnchorKindViolation {
    diagram_id: Option<String>,
    plane_id: Option<String>,
    element: &'static str,
    element_id: Option<String>,
    reference: String,
    actual_semantic_tag: String,
    expected_anchor_kind: &'static str,
}

impl DiAnchorKindViolation {
    pub(super) fn plane(
        diagram_id: Option<&str>,
        plane_id: Option<&str>,
        reference: &str,
        actual_semantic_tag: &str,
    ) -> Self {
        Self::new(
            diagram_id,
            plane_id,
            "BPMNPlane",
            plane_id,
            reference,
            actual_semantic_tag,
            "diagram_root",
        )
    }

    pub(super) fn shape(
        diagram_id: Option<&str>,
        plane_id: Option<&str>,
        shape_id: Option<&str>,
        reference: &str,
        actual_semantic_tag: &str,
    ) -> Self {
        Self::new(
            diagram_id,
            plane_id,
            "BPMNShape",
            shape_id,
            reference,
            actual_semantic_tag,
            "node_or_artifact",
        )
    }

    pub(super) fn edge(
        diagram_id: Option<&str>,
        plane_id: Option<&str>,
        edge_id: Option<&str>,
        reference: &str,
        actual_semantic_tag: &str,
    ) -> Self {
        Self::new(
            diagram_id,
            plane_id,
            "BPMNEdge",
            edge_id,
            reference,
            actual_semantic_tag,
            "flow_or_association",
        )
    }

    fn new(
        diagram_id: Option<&str>,
        plane_id: Option<&str>,
        element: &'static str,
        element_id: Option<&str>,
        reference: &str,
        actual_semantic_tag: &str,
        expected_anchor_kind: &'static str,
    ) -> Self {
        Self {
            diagram_id: diagram_id.map(str::to_string),
            plane_id: plane_id.map(str::to_string),
            element,
            element_id: element_id.map(str::to_string),
            reference: reference.to_string(),
            actual_semantic_tag: actual_semantic_tag.to_string(),
            expected_anchor_kind,
        }
    }

    pub(super) fn evidence(&self) -> Value {
        json!({
            "diagram_id": self.diagram_id.as_deref(),
            "plane_id": self.plane_id.as_deref(),
            "element": self.element,
            "element_id": self.element_id.as_deref(),
            "attribute": "bpmnElement",
            "reference": self.reference,
            "actual_semantic_tag": self.actual_semantic_tag,
            "expected_anchor_kind": self.expected_anchor_kind,
        })
    }
}
