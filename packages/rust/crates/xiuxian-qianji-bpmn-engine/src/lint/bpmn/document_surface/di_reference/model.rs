use serde_json::{Value, json};

#[derive(Debug)]
pub(super) struct DiReferenceViolation {
    diagram_id: Option<String>,
    plane_id: Option<String>,
    element: &'static str,
    element_id: Option<String>,
    attribute: &'static str,
    reference: String,
    expected_scope: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DiReferenceScope<'a> {
    pub(super) diagram_id: Option<&'a str>,
    pub(super) plane_id: Option<&'a str>,
}

#[derive(Debug)]
pub(super) struct DiReferenceTarget {
    pub(super) element: &'static str,
    pub(super) element_id: Option<String>,
    pub(super) attribute: &'static str,
}

impl DiReferenceViolation {
    pub(super) fn semantic(
        scope: DiReferenceScope<'_>,
        target: DiReferenceTarget,
        reference: &str,
    ) -> Self {
        Self {
            diagram_id: scope.diagram_id.map(str::to_string),
            plane_id: scope.plane_id.map(str::to_string),
            element: target.element,
            element_id: target.element_id,
            attribute: target.attribute,
            reference: reference.to_string(),
            expected_scope: "semantic_bpmn_id",
        }
    }

    pub(super) fn local(
        scope: DiReferenceScope<'_>,
        target: DiReferenceTarget,
        reference: &str,
    ) -> Self {
        Self {
            diagram_id: scope.diagram_id.map(str::to_string),
            plane_id: scope.plane_id.map(str::to_string),
            element: target.element,
            element_id: target.element_id,
            attribute: target.attribute,
            reference: reference.to_string(),
            expected_scope: "diagram_interchange_id",
        }
    }

    pub(super) fn evidence(&self) -> Value {
        json!({
            "diagram_id": self.diagram_id.as_deref(),
            "plane_id": self.plane_id.as_deref(),
            "element": self.element,
            "element_id": self.element_id.as_deref(),
            "attribute": self.attribute,
            "reference": self.reference,
            "expected_scope": self.expected_scope,
        })
    }
}
