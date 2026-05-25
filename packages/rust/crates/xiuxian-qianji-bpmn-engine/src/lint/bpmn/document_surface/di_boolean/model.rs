use serde_json::{Value, json};

#[derive(Debug)]
pub(super) struct DiBooleanViolation {
    element: String,
    element_id: Option<String>,
    path: String,
    attribute: String,
    value: String,
}

impl DiBooleanViolation {
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
            "allowed_values": ["true", "false", "1", "0"],
        })
    }
}
