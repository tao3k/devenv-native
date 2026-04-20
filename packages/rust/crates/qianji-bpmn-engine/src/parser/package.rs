//! Public parser-shell types.

use super::import::import_bpmn_source;
use super::normalize::normalize_package;
use super::validate::validate_raw_package;
use crate::dmn::{DmnSourceFile, parse_dmn_decision};
use crate::error::{BpmnEngineError, Result};
use crate::ir::BpmnPackage;

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
/// malformed, typed DMN parse errors when one bundled DMN source is invalid,
/// or [`BpmnEngineError::UnsupportedSourceBundle`] when the bundle contains an
/// unsupported BPMN source count.
pub fn parse_bpmn_bundle(
    snapshot: &BpmnBundleSnapshot,
    options: &BpmnParseOptions,
) -> Result<BpmnPackage> {
    if snapshot.bpmn_sources.len() != 1 {
        return Err(BpmnEngineError::UnsupportedSourceBundle {
            count: snapshot.bpmn_sources.len(),
        });
    }
    if options.validate_schema {
        return Err(BpmnEngineError::UnsupportedOperation {
            operation: "bpmn_schema_validation",
        });
    }

    let raw = import_bpmn_source(&snapshot.bpmn_sources[0])?;
    validate_raw_package(&raw)?;
    let package = normalize_package(raw)?;
    let dmn_decisions = snapshot
        .dmn_sources
        .iter()
        .map(parse_dmn_decision)
        .collect::<Result<Vec<_>>>()?;
    if dmn_decisions.is_empty() {
        Ok(package)
    } else {
        Ok(package.with_dmn_decisions(dmn_decisions))
    }
}
