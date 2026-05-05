//! Public dmn model decision requirement contracts for BPMN/DMN engine integration.

use super::Arc;

/// One bounded executable DMN information-requirement reference.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnInformationRequirementReference {
    /// Direct reference element kind, such as `requiredInput`.
    pub reference_kind: Arc<str>,
    /// Direct href placeholder preserved from source.
    pub href: Option<Arc<str>>,
}

impl DmnInformationRequirementReference {
    /// Creates one bounded information-requirement reference.
    #[must_use]
    pub fn new(reference_kind: impl AsRef<str>, href: Option<impl AsRef<str>>) -> Self {
        Self {
            reference_kind: Arc::<str>::from(reference_kind.as_ref()),
            href: href.map(|value| Arc::<str>::from(value.as_ref())),
        }
    }
}

/// One bounded executable DMN knowledge-requirement reference.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnKnowledgeRequirementReference {
    /// Direct reference element kind, such as `requiredKnowledge`.
    pub reference_kind: Arc<str>,
    /// Direct href placeholder preserved from source.
    pub href: Option<Arc<str>>,
}

impl DmnKnowledgeRequirementReference {
    /// Creates one bounded knowledge-requirement reference.
    #[must_use]
    pub fn new(reference_kind: impl AsRef<str>, href: Option<impl AsRef<str>>) -> Self {
        Self {
            reference_kind: Arc::<str>::from(reference_kind.as_ref()),
            href: href.map(|value| Arc::<str>::from(value.as_ref())),
        }
    }
}
