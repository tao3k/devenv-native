use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DiRequiredAttributeViolation {
    element: String,
    element_id: Option<String>,
    path: String,
    missing_attribute: String,
    required_attributes: &'static [&'static str],
}

impl DiRequiredAttributeViolation {
    pub(super) fn new(
        element: &str,
        element_id: Option<String>,
        path: &str,
        missing_attribute: &str,
        required_attributes: &'static [&'static str],
    ) -> Self {
        Self {
            element: element.to_string(),
            element_id,
            path: path.to_string(),
            missing_attribute: missing_attribute.to_string(),
            required_attributes,
        }
    }

    pub(super) fn evidence(&self) -> Value {
        json!({
            "element": self.element,
            "element_id": self.element_id.as_deref(),
            "path": self.path,
            "missing_attribute": self.missing_attribute,
            "required_attributes": self.required_attributes,
        })
    }
}
