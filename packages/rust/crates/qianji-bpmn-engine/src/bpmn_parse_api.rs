use crate::dmn_model_api::DmnSourceFile;
use crate::error::Result;
use crate::ir_package_api::BpmnPackage;
use crate::parser::parse_bpmn_bundle_impl;

/// In-memory BPMN source input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BpmnSourceFile {
    /// Source identifier used for diagnostics.
    pub source_id: String,
    /// Raw XML or BPMN content.
    pub contents: String,
}

impl BpmnSourceFile {
    /// Creates a BPMN source input.
    #[must_use]
    pub fn new(source_id: impl Into<String>, contents: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            contents: contents.into(),
        }
    }
}

/// Immutable parser-owned source snapshot for one bounded BPMN bundle.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BpmnBundleSnapshot {
    /// BPMN source files owned by the bundle.
    pub bpmn_sources: Vec<BpmnSourceFile>,
    /// Optional DMN source files that should populate the package registry.
    pub dmn_sources: Vec<DmnSourceFile>,
}

impl BpmnBundleSnapshot {
    /// Creates one parser-owned BPMN bundle snapshot.
    #[must_use]
    pub fn new(bpmn_sources: Vec<BpmnSourceFile>) -> Self {
        Self {
            bpmn_sources,
            dmn_sources: Vec::new(),
        }
    }

    /// Attaches DMN source files to the bundle snapshot.
    #[must_use]
    pub fn with_dmn_sources(mut self, dmn_sources: Vec<DmnSourceFile>) -> Self {
        self.dmn_sources = dmn_sources;
        self
    }
}

/// Parse-time options for BPMN package construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BpmnParseOptions {
    /// Whether schema validation should be attempted when implemented.
    pub validate_schema: bool,
    /// Whether ids should be normalized into compact runtime indices.
    pub normalize_ids: bool,
}

impl Default for BpmnParseOptions {
    fn default() -> Self {
        Self {
            validate_schema: false,
            normalize_ids: true,
        }
    }
}

/// Parses BPMN source files into a package shell.
///
/// # Errors
///
/// Returns typed parse or validation errors when the XML payload is malformed
/// or when the BPMN document falls outside the bounded supported subset.
pub fn parse_bpmn_package(
    sources: &[BpmnSourceFile],
    options: &BpmnParseOptions,
) -> Result<BpmnPackage> {
    parse_bpmn_bundle(&BpmnBundleSnapshot::new(sources.to_vec()), options)
}

/// Parses one parser-owned BPMN+DMN bundle snapshot into a package shell.
///
/// # Errors
///
/// Returns typed BPMN parse or validation errors when the BPMN payload is
/// malformed, typed DMN parse errors when one executable bundled DMN source is
/// invalid, or [`BpmnEngineError::UnsupportedSourceBundle`] when the bundle
/// contains an unsupported BPMN source count. Bundled DMN sources with
/// top-level imports are preserved as metadata-only source/import registry
/// entries and are not added to executable DMN decision registries.
pub fn parse_bpmn_bundle(
    snapshot: &BpmnBundleSnapshot,
    options: &BpmnParseOptions,
) -> Result<BpmnPackage> {
    parse_bpmn_bundle_impl(snapshot, options)
}
