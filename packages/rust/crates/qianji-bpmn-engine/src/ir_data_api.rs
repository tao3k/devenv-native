//! Public ir data api contracts for BPMN/DMN engine integration.

use std::sync::Arc;

/// Bounded executable BPMN data-object binding owned by one process.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BpmnDataObjectBindingSpec {
    /// Canonical BPMN `dataObject` identifier.
    pub object_id: Arc<str>,
    /// Optional BPMN `dataObjectReference` identifier that points at the object.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference_id: Option<Arc<str>>,
    /// Workflow variable path used by bounded data associations.
    pub variable_ref: Arc<str>,
}

impl BpmnDataObjectBindingSpec {
    /// Creates a direct `dataObject` binding.
    #[must_use]
    pub fn object(object_id: impl AsRef<str>) -> Self {
        let object_id = Arc::<str>::from(object_id.as_ref());
        Self {
            object_id: Arc::clone(&object_id),
            reference_id: None,
            variable_ref: object_id,
        }
    }

    /// Creates a `dataObjectReference` binding to a canonical `dataObject`.
    #[must_use]
    pub fn reference(reference_id: impl AsRef<str>, object_id: impl AsRef<str>) -> Self {
        let object_id = Arc::<str>::from(object_id.as_ref());
        Self {
            object_id: Arc::clone(&object_id),
            reference_id: Some(Arc::<str>::from(reference_id.as_ref())),
            variable_ref: object_id,
        }
    }

    /// Returns true when this binding is addressed by the supplied BPMN id.
    #[must_use]
    pub fn matches_bpmn_ref(&self, bpmn_ref: &str) -> bool {
        self.object_id.as_ref() == bpmn_ref
            || self
                .reference_id
                .as_ref()
                .is_some_and(|reference_id| reference_id.as_ref() == bpmn_ref)
    }
}
