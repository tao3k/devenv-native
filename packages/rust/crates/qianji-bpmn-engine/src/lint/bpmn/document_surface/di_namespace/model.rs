use serde_json::{Value, json};

#[derive(Debug)]
pub(super) struct DiNamespaceViolation {
    element: String,
    element_id: Option<String>,
    path: String,
    prefix: Option<String>,
    namespace_uri: Option<String>,
    expected_namespace_uri: &'static str,
}

impl DiNamespaceViolation {
    pub(super) fn new(
        element: &str,
        element_id: Option<String>,
        path: &str,
        prefix: Option<&str>,
        namespace_uri: Option<&str>,
        expected_namespace_uri: &'static str,
    ) -> Self {
        Self {
            element: element.to_string(),
            element_id,
            path: path.to_string(),
            prefix: prefix.map(str::to_string),
            namespace_uri: namespace_uri.map(str::to_string),
            expected_namespace_uri,
        }
    }

    pub(super) fn evidence(&self) -> Value {
        json!({
            "element": self.element,
            "element_id": self.element_id.as_deref(),
            "path": self.path,
            "prefix": self.prefix.as_deref(),
            "namespace_uri": self.namespace_uri.as_deref(),
            "expected_namespace_uri": self.expected_namespace_uri,
        })
    }
}
