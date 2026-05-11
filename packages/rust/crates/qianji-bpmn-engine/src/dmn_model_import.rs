//! Public dmn model import contracts for BPMN/DMN engine integration.

use crate::dmn_model_document::DmnImportSnapshot;
use crate::dmn_model_source::DmnSourceDefinition;
use std::sync::Arc;

/// One bounded non-executable DMN import contract owned by a BPMN package.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnImportDefinition {
    /// Source identifier of the DMN document declaring the import.
    pub source_id: Arc<str>,
    /// Optional import alias used by QName-style references.
    pub name: Option<Arc<str>>,
    /// Optional imported model namespace.
    pub namespace: Option<Arc<str>>,
    /// Optional import location URI.
    pub location_uri: Option<Arc<str>>,
    /// Optional imported model type URI.
    pub import_type: Option<Arc<str>>,
}

/// Owned metadata-only binding between one DMN import and one bundled source root.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DmnImportSourceBinding {
    /// Import declaration being reported.
    pub dmn_import: DmnImportDefinition,
    /// Bundled source root matched by the import namespace, when present.
    pub source_definition: Option<DmnSourceDefinition>,
}

/// Named construction payload for one DMN import definition.
pub struct DmnImportDefinitionInput<'a> {
    /// Source identifier of the DMN document declaring the import.
    pub source_id: &'a str,
    /// Optional import alias used by QName-style references.
    pub name: Option<&'a str>,
    /// Optional imported model namespace.
    pub namespace: Option<&'a str>,
    /// Optional import location URI.
    pub location_uri: Option<&'a str>,
    /// Optional imported model type URI.
    pub import_type: Option<&'a str>,
}

impl DmnImportSourceBinding {
    /// Creates one metadata-only import binding report.
    #[must_use]
    pub fn new(
        dmn_import: DmnImportDefinition,
        source_definition: Option<DmnSourceDefinition>,
    ) -> Self {
        Self {
            dmn_import,
            source_definition,
        }
    }

    /// Returns whether this report resolved to a bundled source root.
    #[must_use]
    pub fn is_bound(&self) -> bool {
        self.source_definition.is_some()
    }
}

impl DmnImportDefinition {
    /// Creates one bounded import definition.
    #[must_use]
    pub fn new(input: DmnImportDefinitionInput<'_>) -> Self {
        Self {
            source_id: (Arc::<str>::from(input.source_id)).into(),
            name: input.name.map(Arc::<str>::from),
            namespace: input.namespace.map(Arc::<str>::from),
            location_uri: input.location_uri.map(Arc::<str>::from),
            import_type: input.import_type.map(Arc::<str>::from),
        }
    }

    /// Builds one bounded import definition from one snapshot entry.
    #[must_use]
    pub fn from_snapshot(source_id: impl AsRef<str>, snapshot: &DmnImportSnapshot) -> Self {
        Self::new(DmnImportDefinitionInput {
            source_id: (source_id.as_ref()).into(),
            name: snapshot.name.as_deref(),
            namespace: snapshot.namespace.as_deref(),
            location_uri: snapshot.location_uri.as_deref(),
            import_type: snapshot.import_type.as_deref(),
        })
    }

    /// Returns whether this import was declared by the provided source.
    #[must_use]
    pub fn is_declared_by(&self, source_id: impl AsRef<str>) -> bool {
        let source_id = source_id.as_ref();
        self.source_id.as_ref() == source_id
    }

    /// Returns whether this import uses the provided alias.
    #[must_use]
    pub fn has_name(&self, name: &str) -> bool {
        self.name.as_deref() == Some(name)
    }

    /// Returns whether this import targets the provided namespace.
    #[must_use]
    pub fn has_namespace(&self, namespace: &str) -> bool {
        self.namespace.as_deref() == Some(namespace)
    }

    /// Returns whether this import uses the provided location URI.
    #[must_use]
    pub fn has_location_uri(&self, location_uri: &str) -> bool {
        self.location_uri.as_deref() == Some(location_uri)
    }
}
