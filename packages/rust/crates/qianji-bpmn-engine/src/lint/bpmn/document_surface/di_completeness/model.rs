use serde_json::{Value, json};

#[derive(Debug)]
pub(super) struct DiCompletenessViolation {
    diagram_id: Option<String>,
    plane_id: Option<String>,
    element: &'static str,
    element_id: Option<String>,
    missing: &'static str,
    observed_count: Option<usize>,
}

impl DiCompletenessViolation {
    pub(super) fn shape_bounds(
        diagram_id: Option<&str>,
        plane_id: Option<&str>,
        shape_id: Option<&str>,
    ) -> Self {
        Self {
            diagram_id: diagram_id.map(str::to_string),
            plane_id: plane_id.map(str::to_string),
            element: "BPMNShape",
            element_id: shape_id.map(str::to_string),
            missing: "dc:Bounds",
            observed_count: None,
        }
    }

    pub(super) fn edge_waypoints(
        diagram_id: Option<&str>,
        plane_id: Option<&str>,
        edge_id: Option<&str>,
        observed_count: usize,
    ) -> Self {
        Self {
            diagram_id: diagram_id.map(str::to_string),
            plane_id: plane_id.map(str::to_string),
            element: "BPMNEdge",
            element_id: edge_id.map(str::to_string),
            missing: "di:waypoint[2]",
            observed_count: Some(observed_count),
        }
    }

    pub(super) fn label_style_font(diagram_id: Option<&str>, style_id: Option<&str>) -> Self {
        Self {
            diagram_id: diagram_id.map(str::to_string),
            plane_id: None,
            element: "BPMNLabelStyle",
            element_id: style_id.map(str::to_string),
            missing: "dc:Font",
            observed_count: None,
        }
    }

    pub(super) fn evidence(&self) -> Value {
        json!({
            "diagram_id": self.diagram_id.as_deref(),
            "plane_id": self.plane_id.as_deref(),
            "element": self.element,
            "element_id": self.element_id.as_deref(),
            "missing": self.missing,
            "observed_count": self.observed_count,
        })
    }
}
