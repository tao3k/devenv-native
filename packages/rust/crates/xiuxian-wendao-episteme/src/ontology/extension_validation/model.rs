use std::path::{Path, PathBuf};

/// Episteme extension-pack corpus validation depth.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum EpistemeExtensionValidationMode {
    /// Validate metadata, source paths, byte sizes, and contract cross-links.
    #[default]
    MetadataOnly,
    /// Also read source bytes and compare SHA-256 hashes.
    FullHash,
}

/// Request for Episteme extension-pack validation.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeExtensionValidationRequest {
    pub(super) episteme_root: PathBuf,
    pub(super) corpus_root: Option<PathBuf>,
    pub(super) validation_mode: EpistemeExtensionValidationMode,
}

impl EpistemeExtensionValidationRequest {
    /// Build a validation request for one Episteme extension repository root.
    #[must_use]
    pub fn new(episteme_root: impl Into<PathBuf>) -> Self {
        Self {
            episteme_root: episteme_root.into(),
            corpus_root: None,
            validation_mode: EpistemeExtensionValidationMode::MetadataOnly,
        }
    }

    /// Override the source corpus root instead of resolving it from environment
    /// or `episteme.toml`.
    #[must_use]
    pub fn with_corpus_root(mut self, corpus_root: impl Into<PathBuf>) -> Self {
        self.corpus_root = Some(corpus_root.into());
        self
    }

    /// Select metadata-only or full-hash validation.
    #[must_use]
    pub fn with_validation_mode(
        mut self,
        validation_mode: EpistemeExtensionValidationMode,
    ) -> Self {
        self.validation_mode = validation_mode;
        self
    }

    /// Episteme extension repository root.
    #[must_use]
    pub fn episteme_root(&self) -> &Path {
        self.episteme_root.as_path()
    }

    /// Optional source corpus root override.
    #[must_use]
    pub fn corpus_root(&self) -> Option<&Path> {
        self.corpus_root.as_deref()
    }

    /// Validation mode.
    #[must_use]
    pub fn validation_mode(&self) -> EpistemeExtensionValidationMode {
        self.validation_mode
    }
}

/// Successful Episteme extension-pack validation summary.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EpistemeExtensionValidationReport {
    /// Number of ontology domains.
    pub domains: usize,
    /// Number of RDF files.
    pub rdf_files: usize,
    /// Number of object-model contract files.
    pub object_model_contracts: usize,
    /// Number of source manifests.
    pub source_manifests: usize,
    /// Number of source files declared in `files.tsv`.
    pub source_files: usize,
    /// Number of extraction queue rows.
    pub extraction_queue_rows: usize,
    /// Number of RDF class terms.
    pub rdf_classes: usize,
    /// Number of RDF object-property terms.
    pub rdf_object_properties: usize,
    /// Number of object types.
    pub object_types: usize,
    /// Number of property types.
    pub property_types: usize,
    /// Number of link types.
    pub link_types: usize,
    /// Number of action types.
    pub action_types: usize,
    /// Number of query types.
    pub query_types: usize,
}
