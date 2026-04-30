use serde_json::{Value, json};

#[derive(Debug, Clone, Copy)]
pub(super) struct DiIdentityScope<'a> {
    pub(super) diagram_id: Option<&'a str>,
    pub(super) plane_id: Option<&'a str>,
}

#[derive(Debug)]
pub(super) struct DiIdentityOccurrence {
    diagram_id: Option<String>,
    plane_id: Option<String>,
    element: &'static str,
    element_id: String,
    owner_element: Option<&'static str>,
    owner_id: Option<String>,
}

#[derive(Debug)]
pub(super) struct DiIdentityViolation {
    duplicate_id: String,
    occurrences: Vec<DiIdentityOccurrence>,
}

impl DiIdentityOccurrence {
    pub(super) fn new(
        scope: DiIdentityScope<'_>,
        element: &'static str,
        element_id: &str,
        owner_element: Option<&'static str>,
        owner_id: Option<&str>,
    ) -> Self {
        Self {
            diagram_id: scope.diagram_id.map(str::to_string),
            plane_id: scope.plane_id.map(str::to_string),
            element,
            element_id: element_id.to_string(),
            owner_element,
            owner_id: owner_id.map(str::to_string),
        }
    }

    pub(super) fn evidence(&self) -> Value {
        json!({
            "diagram_id": self.diagram_id.as_deref(),
            "plane_id": self.plane_id.as_deref(),
            "element": self.element,
            "element_id": self.element_id,
            "owner_element": self.owner_element,
            "owner_id": self.owner_id.as_deref(),
        })
    }
}

impl DiIdentityViolation {
    pub(super) fn new(duplicate_id: String, occurrences: Vec<DiIdentityOccurrence>) -> Self {
        Self {
            duplicate_id,
            occurrences,
        }
    }

    pub(super) fn evidence(&self) -> Value {
        json!({
            "duplicate_id": self.duplicate_id,
            "occurrence_count": self.occurrences.len(),
            "occurrences": self
                .occurrences
                .iter()
                .map(DiIdentityOccurrence::evidence)
                .collect::<Vec<_>>(),
        })
    }
}
