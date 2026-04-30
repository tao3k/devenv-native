use serde_json::{Value, json};

#[derive(Debug)]
pub(super) struct DiAnchorViolation {
    diagram_id: Option<String>,
    plane_id: Option<String>,
    element: &'static str,
    element_id: Option<String>,
}

impl DiAnchorViolation {
    pub(super) fn plane(diagram_id: Option<&str>, plane_id: Option<&str>) -> Self {
        Self {
            diagram_id: diagram_id.map(str::to_string),
            plane_id: plane_id.map(str::to_string),
            element: "BPMNPlane",
            element_id: plane_id.map(str::to_string),
        }
    }

    pub(super) fn shape(
        diagram_id: Option<&str>,
        plane_id: Option<&str>,
        shape_id: Option<&str>,
    ) -> Self {
        Self {
            diagram_id: diagram_id.map(str::to_string),
            plane_id: plane_id.map(str::to_string),
            element: "BPMNShape",
            element_id: shape_id.map(str::to_string),
        }
    }

    pub(super) fn edge(
        diagram_id: Option<&str>,
        plane_id: Option<&str>,
        edge_id: Option<&str>,
    ) -> Self {
        Self {
            diagram_id: diagram_id.map(str::to_string),
            plane_id: plane_id.map(str::to_string),
            element: "BPMNEdge",
            element_id: edge_id.map(str::to_string),
        }
    }

    pub(super) fn evidence(&self) -> Value {
        json!({
            "diagram_id": self.diagram_id.as_deref(),
            "plane_id": self.plane_id.as_deref(),
            "element": self.element,
            "element_id": self.element_id.as_deref(),
            "missing": "bpmnElement",
            "expected_scope": "semantic_bpmn_id",
        })
    }
}
