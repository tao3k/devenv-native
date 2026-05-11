//! Public dmn model source contracts for BPMN/DMN engine integration.

use crate::dmn_model_document::DmnRootSnapshot;
use std::sync::Arc;

/// DMN source identifier used for package-local lookup.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DmnSourceId(String);

impl DmnSourceId {
    /// Borrows the serialized source id.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for DmnSourceId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// DMN root `definitions@id` identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DmnDefinitionsId(String);

impl DmnDefinitionsId {
    /// Borrows the serialized definitions id.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for DmnDefinitionsId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// DMN model namespace URI discovered from XML namespace declarations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DmnModelNamespaceUri(String);

impl DmnModelNamespaceUri {
    /// Borrows the serialized namespace URI.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for DmnModelNamespaceUri {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

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

/// Named construction payload for one DMN source-root definition.
pub struct DmnSourceDefinitionInput<'a> {
    /// Source identifier used for diagnostics and package-local lookup.
    pub source_id: DmnSourceId,
    /// Optional root `definitions@id`.
    pub definitions_id: Option<DmnDefinitionsId>,
    /// Optional root `definitions@name`.
    pub name: Option<&'a str>,
    /// Optional DMN business namespace declared on `definitions`.
    pub namespace: Option<&'a str>,
    /// Optional DMN model namespace URI discovered from XML namespace declarations.
    pub model_namespace_uri: Option<DmnModelNamespaceUri>,
    /// Optional model-version hint derived from the model namespace URI.
    pub model_version_hint: Option<&'a str>,
}

impl DmnSourceDefinition {
    /// Creates one bounded DMN source-root definition.
    #[must_use]
    pub fn new(input: DmnSourceDefinitionInput<'_>) -> Self {
        Self {
            source_id: (Arc::<str>::from(input.source_id.as_str())).into(),
            definitions_id: input
                .definitions_id
                .as_ref()
                .map(|value| Arc::<str>::from(value.as_str())),
            name: input.name.map(Arc::<str>::from),
            namespace: input.namespace.map(Arc::<str>::from),
            model_namespace_uri: input
                .model_namespace_uri
                .as_ref()
                .map(|value| Arc::<str>::from(value.as_str())),
            model_version_hint: input.model_version_hint.map(Arc::<str>::from),
        }
    }

    /// Builds one bounded source-root definition from a document snapshot root.
    #[must_use]
    pub fn from_root_snapshot(source_id: impl AsRef<str>, root: &DmnRootSnapshot) -> Self {
        Self::new(DmnSourceDefinitionInput {
            source_id: source_id.as_ref().into(),
            definitions_id: root.definitions_id.as_deref().map(Into::into),
            name: root.name.as_deref(),
            namespace: root.namespace.as_deref(),
            model_namespace_uri: root.model_namespace_uri.as_deref().map(Into::into),
            model_version_hint: root.model_version_hint.as_deref(),
        })
    }

    /// Returns whether this source root has the requested source id.
    #[must_use]
    pub fn has_source_id(&self, source_id: impl AsRef<str>) -> bool {
        let source_id = source_id.as_ref();
        self.source_id.as_ref() == source_id
    }

    /// Returns whether this source root declares the requested namespace.
    #[must_use]
    pub fn has_namespace(&self, namespace: &str) -> bool {
        self.namespace.as_deref() == Some(namespace)
    }
}
