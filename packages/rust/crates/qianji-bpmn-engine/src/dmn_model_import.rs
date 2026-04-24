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
    pub fn new(
        source_id: impl AsRef<str>,
        name: Option<impl AsRef<str>>,
        namespace: Option<impl AsRef<str>>,
        location_uri: Option<impl AsRef<str>>,
        import_type: Option<impl AsRef<str>>,
    ) -> Self {
        Self {
            source_id: Arc::<str>::from(source_id.as_ref()),
            name: name.map(|value| Arc::<str>::from(value.as_ref())),
            namespace: namespace.map(|value| Arc::<str>::from(value.as_ref())),
            location_uri: location_uri.map(|value| Arc::<str>::from(value.as_ref())),
            import_type: import_type.map(|value| Arc::<str>::from(value.as_ref())),
        }
    }

    /// Builds one bounded import definition from one snapshot entry.
    #[must_use]
    pub fn from_snapshot(source_id: impl AsRef<str>, snapshot: &DmnImportSnapshot) -> Self {
        Self::new(
            source_id,
            snapshot.name.as_deref(),
            snapshot.namespace.as_deref(),
            snapshot.location_uri.as_deref(),
            snapshot.import_type.as_deref(),
        )
    }

    /// Returns whether this import was declared by the provided source.
    #[must_use]
    pub fn is_declared_by(&self, source_id: &str) -> bool {
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
