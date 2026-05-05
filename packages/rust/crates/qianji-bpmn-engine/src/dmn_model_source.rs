//! Public dmn model source contracts for BPMN/DMN engine integration.

use crate::dmn_model_document::DmnRootSnapshot;
use std::sync::Arc;

/// One bounded non-executable DMN source-root contract owned by a BPMN package.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnSourceDefinition {
    /// Source identifier used for diagnostics and package-local lookup.
    pub source_id: Arc<str>,
    /// Optional root `definitions@id`.
    pub definitions_id: Option<Arc<str>>,
    /// Optional root `definitions@name`.
    pub name: Option<Arc<str>>,
    /// Optional DMN business namespace declared on `definitions`.
    pub namespace: Option<Arc<str>>,
    /// Optional DMN model namespace URI discovered from XML namespace declarations.
    pub model_namespace_uri: Option<Arc<str>>,
    /// Optional model-version hint derived from the model namespace URI.
    pub model_version_hint: Option<Arc<str>>,
}

impl DmnSourceDefinition {
    /// Creates one bounded DMN source-root definition.
    #[must_use]
    pub fn new(
        source_id: impl AsRef<str>,
        definitions_id: Option<impl AsRef<str>>,
        name: Option<impl AsRef<str>>,
        namespace: Option<impl AsRef<str>>,
        model_namespace_uri: Option<impl AsRef<str>>,
        model_version_hint: Option<impl AsRef<str>>,
    ) -> Self {
        Self {
            source_id: Arc::<str>::from(source_id.as_ref()),
            definitions_id: definitions_id.map(|value| Arc::<str>::from(value.as_ref())),
            name: name.map(|value| Arc::<str>::from(value.as_ref())),
            namespace: namespace.map(|value| Arc::<str>::from(value.as_ref())),
            model_namespace_uri: model_namespace_uri.map(|value| Arc::<str>::from(value.as_ref())),
            model_version_hint: model_version_hint.map(|value| Arc::<str>::from(value.as_ref())),
        }
    }

    /// Builds one bounded source-root definition from a document snapshot root.
    #[must_use]
    pub fn from_root_snapshot(source_id: impl AsRef<str>, root: &DmnRootSnapshot) -> Self {
        Self::new(
            source_id,
            root.definitions_id.as_deref(),
            root.name.as_deref(),
            root.namespace.as_deref(),
            root.model_namespace_uri.as_deref(),
            root.model_version_hint.as_deref(),
        )
    }

    /// Returns whether this source root has the requested source id.
    #[must_use]
    pub fn has_source_id(&self, source_id: &str) -> bool {
        self.source_id.as_ref() == source_id
    }

    /// Returns whether this source root declares the requested namespace.
    #[must_use]
    pub fn has_namespace(&self, namespace: &str) -> bool {
        self.namespace.as_deref() == Some(namespace)
    }
}
