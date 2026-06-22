//! Typed workflow authoring-source HTTP fields.

use serde::{Deserialize, Serialize};

/// Workflow authoring-source media type carried over the public HTTP JSON API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QianjiControlWorkflowSourceAuthoringMediaType(String);

impl QianjiControlWorkflowSourceAuthoringMediaType {
    pub(in crate::bpmn::http_transport) fn from_text_markdown() -> Self {
        Self("text/markdown".to_owned())
    }

    pub(in crate::bpmn::http_transport) fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
