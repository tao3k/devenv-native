use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DiNumericViolation {
    element: String,
    element_id: Option<String>,
    path: String,
    attribute: String,
    value: String,
}

impl DiNumericViolation {
    pub(super) fn new(
        element: &str,
        element_id: Option<String>,
        path: &str,
        attribute: &str,
        value: &str,
    ) -> Self {
        Self {
            element: element.to_string(),
            element_id,
            path: path.to_string(),
            attribute: attribute.to_string(),
            value: value.to_string(),
        }
    }

    pub(super) fn evidence(&self) -> Value {
        json!({
            "element": self.element,
            "element_id": self.element_id.as_deref(),
            "path": self.path,
            "attribute": self.attribute,
            "value": self.value,
            "expected": "finite_xsd_double",
        })
    }
}
