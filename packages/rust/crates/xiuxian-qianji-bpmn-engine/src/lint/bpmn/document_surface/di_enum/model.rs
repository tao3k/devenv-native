use serde_json::{Value, json};

#[derive(Debug)]
pub(super) struct DiEnumViolation {
    diagram_id: Option<String>,
    plane_id: Option<String>,
    element: &'static str,
    element_id: Option<String>,
    attribute: &'static str,
    value: String,
    allowed_values: &'static [&'static str],
}

impl DiEnumViolation {
    pub(super) fn shape(
        diagram_id: Option<&str>,
        plane_id: Option<&str>,
        shape_id: Option<&str>,
        attribute: &'static str,
        value: &str,
        allowed_values: &'static [&'static str],
    ) -> Self {
        Self::new(
            diagram_id,
            plane_id,
            "BPMNShape",
            shape_id,
            attribute,
            value,
            allowed_values,
        )
    }

    pub(super) fn edge(
        diagram_id: Option<&str>,
        plane_id: Option<&str>,
        edge_id: Option<&str>,
        attribute: &'static str,
        value: &str,
        allowed_values: &'static [&'static str],
    ) -> Self {
        Self::new(
            diagram_id,
            plane_id,
            "BPMNEdge",
            edge_id,
            attribute,
            value,
            allowed_values,
        )
    }

    fn new(
        diagram_id: Option<&str>,
        plane_id: Option<&str>,
        element: &'static str,
        element_id: Option<&str>,
        attribute: &'static str,
        value: &str,
        allowed_values: &'static [&'static str],
    ) -> Self {
        Self {
            diagram_id: diagram_id.map(str::to_string),
            plane_id: plane_id.map(str::to_string),
            element,
            element_id: element_id.map(str::to_string),
            attribute,
            value: value.to_string(),
            allowed_values,
        }
    }

    pub(super) fn evidence(&self) -> Value {
        json!({
            "diagram_id": self.diagram_id.as_deref(),
            "plane_id": self.plane_id.as_deref(),
            "element": self.element,
            "element_id": self.element_id.as_deref(),
            "attribute": self.attribute,
            "value": self.value,
            "allowed_values": self.allowed_values,
        })
    }
}
